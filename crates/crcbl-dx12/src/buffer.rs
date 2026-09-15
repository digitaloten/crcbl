//! What D3D12 requires of a buffer's allocation, and of every view taken over
//! it.
//!
//! # Why this module exists
//!
//! A `Create*View` call returns `void`, so a descriptor D3D12 disagrees with is
//! not an error anywhere — it is a debug-layer message and, when the disagreement
//! is a view running past the end of its resource, `DXGI_ERROR_INVALID_CALL` and
//! a **removed device** reported at whatever call comes next. That is the
//! failure this module exists to make impossible: the arithmetic every buffer
//! view is built from lives here, checked, in one place, rather than inline at
//! every call site where nothing can test it.
//!
//! # Why it is not Windows-only
//!
//! It holds no `windows` type — the vocabulary is the seam's
//! [`BufferUsage`] and [`MemoryLocation`], and the D3D12 side is three integer
//! constants — so off Windows it exists in the test build alone and `cargo test`
//! on any host runs the rules. That is the same argument
//! [`crate::draw`], `crate::dxil`, `crate::present`,
//! `crate::root` and `crate::pin` make, and it matters here for their reason:
//! nothing in D3D12 reports a view whose size disagrees with its resource, so
//! the only place this can be caught is before the call.
//!
//! # A constant buffer's padding is bought at *creation*, not at the view
//!
//! D3D12 requires a constant buffer view's `SizeInBytes` to be a multiple of
//! [`CONSTANT_BUFFER_ALIGNMENT`], and a view may not extend past the end of the
//! resource it names. Those two rules together mean the rounding cannot happen
//! at the view: a 16-byte buffer viewed as 256 bytes is exactly the invalid call
//! above. Rounding *down* is not available either — 16 is not a legal
//! `SizeInBytes`.
//!
//! So [`allocation_size`] pads the **allocation** instead, and only for a buffer
//! whose usage says it may be bound as a constant buffer. Padding every buffer
//! would cost 240 bytes on every four-byte counter in the engine for a rule that
//! applies to none of them.
//!
//! **Nothing above the seam can observe the padding.** The device's table keeps
//! the size the caller asked for, so `write_buffer` still refuses a byte past
//! it, `BindingResource::WHOLE_BUFFER` still resolves to it, and every bounds
//! check a copy or a binding makes is against it. The padding is visible only to
//! D3D12, as slack the view is allowed to name — which is precisely what the
//! rule wants and what the buffer did not have.
//! [`Limits::max_uniform_buffer_range`](crcbl_hal::Limits::max_uniform_buffer_range)
//! is a limit on a *bindable range*, not on an allocation, so it is unaffected:
//! D3D12's own ceiling for it is already a multiple of the alignment.

use crcbl_hal::{BufferUsage, HalError, MemoryLocation};

/// A constant buffer view's `SizeInBytes` and its start must both be multiples
/// of this many bytes.
///
/// D3D12's `D3D12_CONSTANT_BUFFER_DATA_PLACEMENT_ALIGNMENT`, spelled out rather
/// than imported so this module stays free of `windows` types and its tests run
/// on the host this backend is written on.
/// `the_alignments_are_the_ones_d3d12_names` asserts the two agree, in the build
/// that has D3D12 to ask.
pub(crate) const CONSTANT_BUFFER_ALIGNMENT: u64 = 256;

/// The bytes a buffer of `size` has to allocate to satisfy every view its
/// `usage` allows.
///
/// See the module docs for why the padding is here and not at the view, and for
/// why nothing above the seam sees it.
pub(crate) fn allocation_size(size: u64, usage: BufferUsage) -> Result<u64, HalError> {
    if !usage.contains(BufferUsage::UNIFORM) {
        return Ok(size);
    }
    size.checked_next_multiple_of(CONSTANT_BUFFER_ALIGNMENT)
        .ok_or_else(|| {
            HalError::InvalidDescriptor(format!(
                "a {size}-byte uniform buffer cannot be padded to a multiple of \
                 {CONSTANT_BUFFER_ALIGNMENT} bytes without overflowing a 64-bit size, and D3D12 \
                 requires a constant buffer view to be a whole number of those blocks"
            ))
        })
}

/// A constant buffer view's `SizeInBytes` for `offset..offset + size` of a
/// buffer that allocated `allocation` bytes.
///
/// Every one of the three failures below is a call D3D12 answers with `void` and
/// a removed device, so each is an [`HalError::InvalidDescriptor`] here instead.
pub(crate) fn constant_view_size(
    offset: u64,
    size: u64,
    allocation: u64,
    binding: u32,
) -> Result<u32, HalError> {
    if !offset.is_multiple_of(CONSTANT_BUFFER_ALIGNMENT) {
        return Err(HalError::InvalidDescriptor(format!(
            "binding {binding} starts a constant buffer view at byte {offset}, and D3D12 requires \
             a multiple of {CONSTANT_BUFFER_ALIGNMENT} — the alignment this device reports as \
             Limits::min_uniform_buffer_offset_alignment"
        )));
    }
    let bytes = size
        .checked_next_multiple_of(CONSTANT_BUFFER_ALIGNMENT)
        .and_then(|bytes| u32::try_from(bytes).ok())
        .ok_or_else(|| {
            HalError::InvalidDescriptor(format!(
                "binding {binding} binds {size} bytes as a constant buffer, and D3D12's \
                 SizeInBytes is a 32-bit count of {CONSTANT_BUFFER_ALIGNMENT}-byte blocks"
            ))
        })?;
    // The check that would have caught the bug this module was written for: the
    // rounding is only legal because `allocation_size` bought the bytes it
    // reads, and this is what says so rather than trusting that it did.
    let end = offset.checked_add(u64::from(bytes));
    if end.is_none_or(|end| end > allocation) {
        return Err(HalError::InvalidDescriptor(format!(
            "binding {binding} would read {bytes} bytes from byte {offset} of a buffer that \
             allocated {allocation}: a constant buffer view is a whole number of \
             {CONSTANT_BUFFER_ALIGNMENT}-byte blocks, so a buffer bound as one must be created \
             with BufferUsage::UNIFORM to be padded to them"
        )));
    }
    Ok(bytes)
}

/// A structured buffer view's `FirstElement` and `NumElements` for
/// `offset..offset + size`, in elements of `stride` bytes.
///
/// **Structured, not raw.** Slang emits every storage buffer as
/// `StructuredBuffer<T>`, and a driver addresses such a buffer's elements by the
/// view's `StructureByteStride` — so the view carries the stride the layout
/// declared and counts in its elements. A raw view (`R32_TYPELESS`,
/// `_FLAG_RAW`, stride zero) is `ByteBuffer`'s shape, and bound under a
/// `StructuredBuffer<T>` it aliased every element onto the first on an RX 9060
/// XT while WARP read it the way the shader meant; that is how this function
/// came to replace `raw_view_range`.
///
/// Both ends are in whole elements: the start must be one, because
/// `FirstElement` has no way to name a byte inside an element, and a range that
/// is not a whole number of them names the elements it fully covers.
pub(crate) fn structured_view_range(
    offset: u64,
    size: u64,
    stride: u32,
    binding: u32,
) -> Result<(u64, u32), HalError> {
    if stride == 0 {
        // `check_entries` refuses this at layout creation; a record that got
        // here with one would divide by it.
        return Err(HalError::InvalidDescriptor(format!(
            "binding {binding} is a storage buffer with a stride of 0"
        )));
    }
    let element = u64::from(stride);
    if !offset.is_multiple_of(element) {
        return Err(HalError::InvalidDescriptor(format!(
            "binding {binding} starts a structured buffer view at byte {offset}, which is not a \
             whole number of its {stride}-byte elements; D3D12's FirstElement counts elements, \
             so the offset must be a multiple of the stride the layout declares"
        )));
    }
    let elements = size / element;
    if elements == 0 {
        return Err(HalError::InvalidDescriptor(format!(
            "binding {binding} binds {size} bytes as a storage buffer, which is less than one of \
             its {stride}-byte elements"
        )));
    }
    let elements = u32::try_from(elements).map_err(|_| {
        HalError::InvalidDescriptor(format!(
            "binding {binding} binds {size} bytes as a storage buffer, and D3D12's NumElements is \
             a 32-bit count of {stride}-byte elements"
        ))
    })?;
    Ok((offset / element, elements))
}

/// Whether an unordered access view may exist over a buffer in this memory
/// location.
///
/// **A host-visible buffer can never carry one.** D3D12 pins a resource on the
/// upload heap to `GENERIC_READ` and one on the readback heap to `COPY_DEST` for
/// its whole lifetime, and a UAV both requires the resource to have been created
/// with `D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS` — which those heaps reject
/// at creation — and to be in the `UNORDERED_ACCESS` state when the shader
/// writes, which they can never reach. So the combination is not a flag this
/// backend forgot to set; it is one D3D12 has no way to express, and the seam
/// permits it because Vulkan does.
///
/// Refusing here turns what was `CreateUnorderedAccessView` writing nothing —
/// followed by a device removed at the next call — into an error naming the
/// binding and the fix.
pub(crate) fn check_unordered_access(
    location: MemoryLocation,
    binding: u32,
) -> Result<(), HalError> {
    if matches!(location, MemoryLocation::DeviceLocal) {
        return Ok(());
    }
    Err(HalError::InvalidDescriptor(format!(
        "binding {binding} binds a {location:?} buffer for writing, and D3D12 has no unordered \
         access view of one: its upload and readback heaps refuse \
         D3D12_RESOURCE_FLAG_ALLOW_UNORDERED_ACCESS at creation and pin the resource to a state a \
         shader cannot write from. A buffer a shader writes must be MemoryLocation::DeviceLocal; \
         read-only storage bindings of a host-visible buffer are unaffected"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two buffers the D3D12 frame died on, by size: `forward params` is 16
    /// bytes and `forward cull params 0` is 112, and the view over each asked
    /// for 256.
    const FRAME_UNIFORMS: &[u64] = &[16, 112];

    /// `HalError` carries no `PartialEq`, so a refusal is read as its message.
    fn refusal<T: core::fmt::Debug>(result: Result<T, HalError>, what: &str) -> String {
        match result {
            Ok(value) => panic!("{what} was accepted as {value:?}"),
            Err(why) => format!("{why}"),
        }
    }

    /// The padding is a whole number of blocks, and it is bought for the sizes
    /// that actually removed a device.
    #[test]
    fn a_uniform_buffer_allocates_a_whole_number_of_constant_buffer_blocks() {
        for &size in FRAME_UNIFORMS {
            let allocation = allocation_size(size, BufferUsage::UNIFORM).expect("a small buffer");
            assert_eq!(allocation, CONSTANT_BUFFER_ALIGNMENT, "{size} bytes");
        }
        for (size, padded) in [
            (1, 256),
            (255, 256),
            (256, 256),
            (257, 512),
            (65_536, 65_536),
        ] {
            let allocation = allocation_size(size, BufferUsage::UNIFORM).expect("a small buffer");
            assert_eq!(allocation, padded, "{size} bytes");
        }
    }

    /// Only the usage that can reach a constant buffer view pays for it.
    #[test]
    fn a_buffer_that_is_never_a_constant_buffer_is_allocated_at_its_own_size() {
        for usage in [
            BufferUsage::STORAGE,
            BufferUsage::INDEX,
            BufferUsage::INDIRECT | BufferUsage::TRANSFER_SRC,
            BufferUsage::empty(),
        ] {
            let allocation = allocation_size(4, usage).expect("no padding to compute");
            assert_eq!(allocation, 4, "{usage:?}");
        }
        // A buffer that is both is padded, because the constant buffer view is
        // the binding with the rule.
        let both = allocation_size(4, BufferUsage::STORAGE | BufferUsage::UNIFORM)
            .expect("a small buffer");
        assert_eq!(both, CONSTANT_BUFFER_ALIGNMENT);
    }

    /// The property the frame violated: the view a binding builds never runs
    /// past the allocation the buffer bought.
    #[test]
    fn a_constant_buffer_view_never_outruns_the_allocation_its_padding_bought() {
        for size in (1..=1024).chain([4096, 65_536]) {
            let allocation = allocation_size(size, BufferUsage::UNIFORM).expect("a small buffer");
            let bytes = constant_view_size(0, size, allocation, 0)
                .unwrap_or_else(|why| panic!("{size} bytes: {why}"));
            assert!(
                u64::from(bytes) <= allocation,
                "{size} bytes: a view of {bytes} over an allocation of {allocation}"
            );
            assert!(
                u64::from(bytes).is_multiple_of(CONSTANT_BUFFER_ALIGNMENT),
                "{size} bytes: {bytes} is not a whole number of blocks"
            );
            assert!(
                u64::from(bytes) >= size,
                "{size} bytes: a view of {bytes} is short of what was bound"
            );
        }
    }

    /// A binding into the middle of a larger uniform buffer — the shape a
    /// per-object block takes — is bounded by the same arithmetic.
    #[test]
    fn a_binding_into_the_middle_of_a_uniform_buffer_is_bounded_by_the_allocation() {
        let allocation = allocation_size(512, BufferUsage::UNIFORM).expect("a small buffer");
        let block = u32::try_from(CONSTANT_BUFFER_ALIGNMENT).expect("256 fits in 32 bits");
        for size in [16, 256] {
            let bytes = constant_view_size(256, size, allocation, 0).expect("the second block");
            assert_eq!(bytes, block, "{size} bytes at offset 256");
        }
        // One block further along there is nothing left to round up into.
        let text = refusal(
            constant_view_size(512, 16, allocation, 7),
            "a view past the end",
        );
        assert!(text.contains("binding 7"), "{text}");
        assert!(text.contains("allocated 512"), "{text}");
    }

    /// The pre-fix state, stated as a test: an unpadded buffer bound as a
    /// constant buffer is refused rather than handed to D3D12.
    ///
    /// This is the call that removed the device — a 16-byte resource with a
    /// 256-byte view over it — and it is what goes red if the padding is ever
    /// dropped from [`allocation_size`].
    #[test]
    fn a_view_that_would_outrun_its_buffer_is_refused_rather_than_written() {
        for &size in FRAME_UNIFORMS {
            let text = refusal(
                constant_view_size(0, size, size, 3),
                "a 256-byte view over an unpadded buffer",
            );
            assert!(text.contains("binding 3"), "{text}");
            assert!(text.contains(&format!("allocated {size}")), "{text}");
            assert!(text.contains("BufferUsage::UNIFORM"), "{text}");
        }
    }

    /// D3D12 has no unaligned constant buffer view, and the offset is where a
    /// caller's own arithmetic arrives.
    #[test]
    fn a_constant_buffer_binding_offset_must_be_a_multiple_of_the_alignment() {
        for offset in [1, 4, 16, 128, 255, 257] {
            let text = refusal(
                constant_view_size(offset, 16, 65_536, 2),
                "an unaligned constant buffer view",
            );
            assert!(text.contains("binding 2"), "{offset}: {text}");
            assert!(text.contains("256"), "{offset}: {text}");
        }
        let aligned = constant_view_size(256, 16, 65_536, 2).expect("an aligned offset");
        assert_eq!(u64::from(aligned), CONSTANT_BUFFER_ALIGNMENT);
    }

    /// `SizeInBytes` is 32-bit, and a size that does not fit is a refusal rather
    /// than a truncation that would read the wrong bytes.
    #[test]
    fn a_constant_buffer_view_larger_than_a_u32_is_refused() {
        let text = refusal(
            constant_view_size(0, u64::from(u32::MAX) + 1, u64::MAX, 1),
            "a view past 32 bits",
        );
        assert!(text.contains("32-bit"), "{text}");
        // And the padding itself cannot overflow silently.
        let text = refusal(
            allocation_size(u64::MAX, BufferUsage::UNIFORM),
            "a size past 64 bits",
        );
        assert!(text.contains("overflow"), "{text}");
    }

    /// A structured view counts elements of the declared stride, from a start
    /// that is a whole element.
    ///
    /// Two strides, so a function that ignored its `stride` and counted words —
    /// the raw view this replaced — fails the 32-byte half.
    #[test]
    fn a_structured_view_counts_elements_of_its_stride_from_a_whole_element() {
        assert_eq!(structured_view_range(0, 64, 4, 0).expect("words"), (0, 16));
        assert_eq!(
            structured_view_range(0, 96, 32, 0).expect("vertices"),
            (0, 3)
        );
        assert_eq!(
            structured_view_range(32, 96, 32, 0).expect("one in"),
            (1, 3)
        );
        // A range that is not a whole number of elements names the elements it
        // covers and no partial one.
        assert_eq!(
            structured_view_range(0, 70, 32, 0).expect("70 bytes"),
            (0, 2)
        );
        for offset in [4, 16, 31] {
            let text = refusal(
                structured_view_range(offset, 96, 32, 5),
                "a mid-element start",
            );
            assert!(text.contains("binding 5"), "{offset}: {text}");
            assert!(text.contains("32-byte"), "{offset}: {text}");
        }
        let text = refusal(structured_view_range(0, 31, 32, 5), "less than one element");
        assert!(text.contains("less than one"), "{text}");
        let text = refusal(structured_view_range(0, 64, 0, 5), "a zero stride");
        assert!(text.contains("stride of 0"), "{text}");
    }

    /// A host-visible buffer is refused for writing, and only for writing.
    #[test]
    fn only_a_device_local_buffer_can_carry_an_unordered_access_view() {
        check_unordered_access(MemoryLocation::DeviceLocal, 0).expect("the default heap");
        for location in [MemoryLocation::HostUpload, MemoryLocation::HostReadback] {
            let text = refusal(
                check_unordered_access(location, 4),
                "an unordered access view of a host-visible buffer",
            );
            assert!(text.contains("binding 4"), "{location:?}: {text}");
            assert!(text.contains(&format!("{location:?}")), "{text}");
            assert!(text.contains("DeviceLocal"), "{text}");
        }
    }

    /// The constant-buffer alignment above is D3D12's own number, asserted against
    /// the header rather than against a comment — in the build that has D3D12 to
    /// ask.
    #[cfg(target_os = "windows")]
    #[test]
    fn the_alignments_are_the_ones_d3d12_names() {
        use windows::Win32::Graphics::Direct3D12::D3D12_CONSTANT_BUFFER_DATA_PLACEMENT_ALIGNMENT;

        assert_eq!(
            CONSTANT_BUFFER_ALIGNMENT,
            u64::from(D3D12_CONSTANT_BUFFER_DATA_PLACEMENT_ALIGNMENT)
        );
    }
}
