//! The constants and the uniform block the three `cmaa2_*.slang` sources read,
//! in the layouts those shaders declare.
//!
//! Same reason as [`crate::fxaa`]: the shaders fix numbers and a byte layout,
//! every producer of those has to agree with them exactly, and keeping the
//! mirror in the crate that owns the sources means there is one place to change
//! rather than one per consumer.
//!
//! # What the tier stores, and how wide each list is
//!
//! `docs/plan/49-antialiasing.md`'s CMAA2 rung is five passes over five
//! buffers: two counters, one edge word per pixel, a **candidate** list of edge
//! pixels, a **blend item** list, and a fixed-point accumulation of
//! [`ACCUM_WORDS`] per pixel. The two lists have capacities rather than a
//! bound — how many edges a frame has is a property of the picture — so
//! [`candidate_capacity`] and [`item_capacity`] set them as a fraction of the
//! frame, and a frame that exceeds one **drops** the entries past it.
//!
//! Dropping is the whole degradation: the pixel an unwritten item was for keeps
//! its unresolved colour, and `cmaa2_shapes.slang` clamps every count it reads
//! back to the same capacity, so nothing is ever read or written outside a
//! list. `crates/crcbl/tests/mesh_e2e/cmaa2.rs` drives a frame past a
//! deliberately tiny cap and holds that.

/// Invocations per workgroup, matching `[numthreads(64, 1, 1)]` in every
/// `cmaa2_*.slang` compute entry point.
pub const WORKGROUP_SIZE: u32 = 64;

/// Bytes of the uniform block: four `uint`s.
///
/// Sixteen bytes of value, which is already the multiple of 16 `std140`
/// requires of a uniform block's size, so there is no tail padding to write.
pub const PARAMS_SIZE: usize = 16;

/// Words in the control buffer: the candidate count and the blend-item count.
///
/// Both are atomic adds, so both start each frame at zero — `clearMain` is the
/// dispatch that writes it, on `clear_counters.slang`'s terms.
pub const CONTROL_WORDS: u32 = 2;

/// Words per blend item: the pixel being blended, the pixel whose colour it
/// takes a share of, and that share as a fixed-point weight.
pub const ITEM_WORDS: u32 = 3;

/// Words per pixel in the accumulation: three colour channels and the weight
/// they were premultiplied by.
pub const ACCUM_WORDS: u32 = 4;

/// Bytes in one word of any of this tier's buffers.
pub const WORD_BYTES: u64 = 4;

/// What a fixed-point weight of one is, matching
/// `BLEND_FIXED_POINT_SCALE` in `shaders/cmaa2_shapes.slang`.
///
/// Two to the twentieth, and the choice is bounded from both sides. **Below**:
/// one part in 1048576 is three orders of magnitude finer than the `1/255` step
/// of an eight-bit target, so no accumulated colour can be quantised into a
/// different written texel. **Above**: a `u32` holds 4096 of these, and a pixel
/// takes a handful of blend items whose weights are each at most
/// [`MAX_BLEND_WEIGHT`] and whose colours are saturated into `[0, 1]`, so the
/// sum has no path to a wrap.
///
/// It is a power of two, which is what makes the conversion back exact — see
/// `INV_BLEND_FIXED_POINT_SCALE` in `shaders/cmaa2_apply.slang`.
pub const BLEND_FIXED_POINT_SCALE: u32 = 1 << 20;

/// The longest run `cmaa2_shapes.slang` will classify, in pixels.
///
/// A bound on a loop rather than a quality knob; that shader's constant carries
/// the argument for leaving a longer run unclassified rather than truncating
/// it.
pub const MAX_LINE_LENGTH: u32 = 64;

/// The luma delta across a pixel boundary, in display space, below which
/// `cmaa2_edges.slang` marks no edge.
///
/// **This tree's number rather than Intel's.** It is the threshold the retired
/// SMAA tier used on the same fixture, kept because it is the one that has been
/// measured here; `crates/crcbl/tests/mesh_e2e/cmaa2.rs` is what holds it and
/// `docs/backlog.md` records that the reference's own value is unverified.
pub const EDGE_THRESHOLD: f32 = 0.1;

/// How much stronger a neighbouring boundary has to be before this one is not
/// marked.
///
/// The local-contrast adaptation both CMAA2 and SMAA carry, in the form this
/// tree measured — see [`EDGE_THRESHOLD`] on where the number comes from.
pub const LOCAL_CONTRAST_ADAPTATION_FACTOR: f32 = 2.0;

/// How much of the `U`-shape reconstruction `cmaa2_shapes.slang` applies.
///
/// This transcription's number; that shader's constant argues it.
pub const U_SHAPE_WEIGHT: f32 = 0.5;

/// The largest share one blend item may carry: half a pixel.
pub const MAX_BLEND_WEIGHT: f32 = 0.5;

/// What fraction of a frame's pixels the candidate list holds — one in
/// [`CANDIDATE_DIVISOR`].
///
/// An eighth. A frame whose edge pixels are more than an eighth of it is not a
/// picture with edges in it, it is noise, and the tier has nothing useful to do
/// with the rest — so this buys the list's memory back rather than sizing for a
/// frame nobody draws. Past it the candidates are dropped, which this module's
/// header describes.
pub const CANDIDATE_DIVISOR: u32 = 8;

/// The same for the blend-item list — one in [`ITEM_DIVISOR`].
///
/// A quarter, which is twice the candidate list: a pixel may be covered by a
/// horizontal run and a vertical one, and each covered pixel is one item.
pub const ITEM_DIVISOR: u32 = 4;

/// The floor under both capacities, in entries.
///
/// One workgroup. A frame small enough for a divided capacity to fall under
/// this is a frame where the memory saved is nothing and the arithmetic is all
/// that is left, and a capacity of zero would make the whole tier a no-op with
/// no tell.
pub const MIN_LIST_CAPACITY: u32 = WORKGROUP_SIZE;

/// How many candidates a frame of `width` by `height` may append before they
/// start being dropped.
#[must_use]
pub fn candidate_capacity(width: u32, height: u32) -> u32 {
    list_capacity(width, height, CANDIDATE_DIVISOR)
}

/// How many blend items it may append, likewise.
#[must_use]
pub fn item_capacity(width: u32, height: u32) -> u32 {
    list_capacity(width, height, ITEM_DIVISOR)
}

/// A frame's pixel count divided by `divisor`, floored at [`MIN_LIST_CAPACITY`].
///
/// Saturating rather than wrapping: an extent a caller got wrong is a capacity
/// that is merely large, and the buffer built from it is refused by the seam
/// where a wrapped one would be silently tiny.
fn list_capacity(width: u32, height: u32, divisor: u32) -> u32 {
    let pixels = width.max(1).saturating_mul(height.max(1));
    (pixels / divisor).max(MIN_LIST_CAPACITY)
}

/// The uniform block, matching `struct Cmaa2Params` in every `cmaa2_*.slang`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cmaa2Params {
    /// Width of the image being filtered, in texels. Every buffer this tier
    /// owns is indexed `y * viewport_x + x`.
    pub viewport_x: u32,
    /// Its height.
    pub viewport_y: u32,
    /// See [`candidate_capacity`].
    pub candidate_capacity: u32,
    /// See [`item_capacity`].
    pub item_capacity: u32,
}

impl Cmaa2Params {
    /// The block for a frame of `width` by `height` at the default capacities.
    ///
    /// The extent is floored at one texel: a zero extent would make the pixel
    /// count zero, and a dispatch of no groups is something Metal rejects
    /// outright rather than treating as a no-op.
    #[must_use]
    pub fn for_extent(width: u32, height: u32) -> Self {
        Self {
            viewport_x: width.max(1),
            viewport_y: height.max(1),
            candidate_capacity: candidate_capacity(width, height),
            item_capacity: item_capacity(width, height),
        }
    }

    /// The same block with both list capacities clamped to `cap`.
    ///
    /// **A window for a test**, on [`crate::exposure`]'s terms: the overflow
    /// path is the one behaviour of this tier that no picture shows, and the
    /// only way to reach it on a frame small enough to check is to make the
    /// lists small. `crates/crcbl/tests/mesh_e2e/cmaa2.rs` is the caller.
    ///
    /// A cap of zero is lifted to **one** entry rather than honoured: a buffer
    /// of no bytes is refused at the seam, so a frame built from it would fail
    /// before any device saw it, which is not the path a caller asking for a
    /// tiny list wants to test. The floor here is one and not
    /// [`MIN_LIST_CAPACITY`] on purpose — a cap that could not go below a whole
    /// workgroup would be a knob that cannot reach what it exists for.
    #[must_use]
    pub fn with_capacity_cap(self, cap: u32) -> Self {
        let cap = cap.max(1);
        Self {
            candidate_capacity: self.candidate_capacity.min(cap),
            item_capacity: self.item_capacity.min(cap),
            ..self
        }
    }

    /// The block as the bytes a uniform buffer holds, in `std140` order.
    #[must_use]
    pub fn to_bytes(self) -> [u8; PARAMS_SIZE] {
        let mut bytes = [0u8; PARAMS_SIZE];
        for (slot, value) in [
            self.viewport_x,
            self.viewport_y,
            self.candidate_capacity,
            self.item_capacity,
        ]
        .into_iter()
        .enumerate()
        {
            let at = slot * 4;
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `cmaa2_*.slang`.
    const SOURCES: [(&str, &str); 3] = [
        (
            "cmaa2_edges.slang",
            include_str!("../shaders/cmaa2_edges.slang"),
        ),
        (
            "cmaa2_shapes.slang",
            include_str!("../shaders/cmaa2_shapes.slang"),
        ),
        (
            "cmaa2_apply.slang",
            include_str!("../shaders/cmaa2_apply.slang"),
        ),
    ];

    /// The block every source declares, member for member.
    ///
    /// Nothing else can catch a rename or a reorder: the shaders compile either
    /// way and the buffer is bound either way, and a block whose members moved
    /// would read a capacity as an extent and dispatch over the wrong frame.
    /// Reading the sources is the check, and they are hash-pinned by the
    /// manifest, so they are the files the committed artifacts were built from.
    #[test]
    fn every_cmaa2_source_declares_the_block_to_bytes_writes() {
        for (name, source) in SOURCES {
            for member in [
                "uint viewport_x;",
                "uint viewport_y;",
                "uint candidate_capacity;",
                "uint item_capacity;",
            ] {
                assert!(
                    source.contains(member),
                    "{name} does not declare `{member}`"
                );
            }
            assert!(
                source.contains("ConstantBuffer<Cmaa2Params> params;"),
                "{name} does not bind the block `to_bytes` writes"
            );
        }
    }

    /// The four compute entry points declare the workgroup size this crate
    /// sizes their dispatches with.
    ///
    /// A mismatch is this tier's quietest failure: a dispatch sized against the
    /// wrong number covers a *prefix* of the frame, so the picture is right at
    /// the top and unfiltered below a line nothing names.
    #[test]
    fn the_compute_sources_declare_the_workgroup_size_this_module_names() {
        let declaration = format!("[numthreads({WORKGROUP_SIZE}, 1, 1)]");
        for name in ["cmaa2_edges.slang", "cmaa2_shapes.slang"] {
            let source = SOURCES
                .into_iter()
                .find(|(source_name, _)| *source_name == name)
                .expect("a listed source")
                .1;
            assert!(
                source.contains(&declaration),
                "{name} does not declare `{declaration}`; WORKGROUP_SIZE has drifted from it"
            );
        }
    }

    /// Every constant a shader and this module both name is written the same in
    /// both.
    ///
    /// The shaders have no `#include`, so each of these is two copies of one
    /// number — and a drift in any of them is a filter that reads a list it did
    /// not fill or scales a weight nothing divides back.
    #[test]
    fn the_shared_constants_are_spelled_the_same_in_the_sources() {
        let edges = SOURCES[0].1;
        let shapes = SOURCES[1].1;
        let apply = SOURCES[2].1;
        for (name, source, declaration) in [
            (
                "cmaa2_edges.slang",
                edges,
                format!("static const uint CONTROL_WORDS = {CONTROL_WORDS};"),
            ),
            (
                "cmaa2_edges.slang",
                edges,
                format!("static const uint ACCUM_WORDS = {ACCUM_WORDS};"),
            ),
            (
                "cmaa2_edges.slang",
                edges,
                format!("static const float EDGE_THRESHOLD = {EDGE_THRESHOLD:?};"),
            ),
            (
                "cmaa2_edges.slang",
                edges,
                format!(
                    "static const float LOCAL_CONTRAST_ADAPTATION_FACTOR = \
                     {LOCAL_CONTRAST_ADAPTATION_FACTOR:?};"
                ),
            ),
            (
                "cmaa2_shapes.slang",
                shapes,
                format!("static const uint ACCUM_WORDS = {ACCUM_WORDS};"),
            ),
            (
                "cmaa2_shapes.slang",
                shapes,
                format!("static const uint ITEM_WORDS = {ITEM_WORDS};"),
            ),
            (
                "cmaa2_shapes.slang",
                shapes,
                format!("static const uint MAX_LINE_LENGTH = {MAX_LINE_LENGTH};"),
            ),
            (
                "cmaa2_shapes.slang",
                shapes,
                format!(
                    "static const float BLEND_FIXED_POINT_SCALE = {:?};",
                    BLEND_FIXED_POINT_SCALE as f32
                ),
            ),
            (
                "cmaa2_shapes.slang",
                shapes,
                format!("static const float U_SHAPE_WEIGHT = {U_SHAPE_WEIGHT:?};"),
            ),
            (
                "cmaa2_shapes.slang",
                shapes,
                format!("static const float MAX_BLEND_WEIGHT = {MAX_BLEND_WEIGHT:?};"),
            ),
            (
                "cmaa2_apply.slang",
                apply,
                format!("static const uint ACCUM_WORDS = {ACCUM_WORDS};"),
            ),
            (
                "cmaa2_apply.slang",
                apply,
                format!(
                    "static const float INV_BLEND_FIXED_POINT_SCALE = 1.0 / {:?};",
                    BLEND_FIXED_POINT_SCALE as f32
                ),
            ),
        ] {
            assert!(
                source.contains(&declaration),
                "{name} does not declare `{declaration}`"
            );
        }
    }

    /// Each member lands in the word the shader will read it from.
    #[test]
    fn every_member_is_written_at_the_offset_the_block_declares() {
        let bytes = Cmaa2Params {
            viewport_x: 1920,
            viewport_y: 1080,
            candidate_capacity: 7,
            item_capacity: 9,
        }
        .to_bytes();
        assert_eq!(bytes.len(), PARAMS_SIZE);
        assert_eq!(
            PARAMS_SIZE % 16,
            0,
            "std140 rounds a uniform block's size up to a multiple of 16"
        );
        let uint_at =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("4"));
        assert_eq!(uint_at(0), 1920, "viewport_x at offset 0");
        assert_eq!(uint_at(4), 1080, "viewport_y at offset 4");
        assert_eq!(uint_at(8), 7, "candidate_capacity at offset 8");
        assert_eq!(uint_at(12), 9, "item_capacity at offset 12");
    }

    /// The capacities are a fraction of the frame, and never zero.
    ///
    /// The floor is the half that matters: a capacity of zero is a tier that
    /// records five passes and filters nothing, which is indistinguishable from
    /// a correct frame with no edges in it.
    #[test]
    fn the_capacities_scale_with_the_frame_and_never_reach_zero() {
        assert_eq!(candidate_capacity(1920, 1080), 1920 * 1080 / 8);
        assert_eq!(item_capacity(1920, 1080), 1920 * 1080 / 4);
        for (width, height) in [(0, 0), (1, 1), (8, 8), (16, 16)] {
            assert!(candidate_capacity(width, height) >= MIN_LIST_CAPACITY);
            assert!(item_capacity(width, height) >= MIN_LIST_CAPACITY);
        }
    }

    /// An extent large enough to overflow a `u32` pixel count still produces a
    /// capacity rather than a wrapped one.
    #[test]
    fn a_capacity_is_saturated_rather_than_wrapped() {
        let capacity = candidate_capacity(u32::MAX, u32::MAX);
        assert_eq!(capacity, u32::MAX / CANDIDATE_DIVISOR);
    }

    /// The cap a test drives the overflow path with lowers both lists and
    /// touches nothing else.
    #[test]
    fn the_capacity_cap_lowers_both_lists_and_leaves_the_extent_alone() {
        let params = Cmaa2Params::for_extent(256, 192).with_capacity_cap(4);
        assert_eq!(params.viewport_x, 256);
        assert_eq!(params.viewport_y, 192);
        assert_eq!(params.candidate_capacity, 4);
        assert_eq!(params.item_capacity, 4);

        // A cap above what the frame asked for leaves the frame's own answer.
        let uncapped = Cmaa2Params::for_extent(256, 192).with_capacity_cap(u32::MAX);
        assert_eq!(uncapped, Cmaa2Params::for_extent(256, 192));

        // And a cap of zero is one entry, not none: a buffer of no bytes is
        // refused at the seam, and that is not the path this knob is for.
        let floored = Cmaa2Params::for_extent(256, 192).with_capacity_cap(0);
        assert_eq!(floored.candidate_capacity, 1);
        assert_eq!(floored.item_capacity, 1);
    }

    /// The fixed-point scale's two bounds, as arithmetic rather than as prose.
    ///
    /// Below: one step of the scale is finer than one step of an eight-bit
    /// channel. Above: a `u32` holds enough of them that a pixel taking every
    /// blend item a run of [`MAX_LINE_LENGTH`] can produce, in both axes, is
    /// nowhere near a wrap.
    #[test]
    fn the_fixed_point_scale_is_finer_than_the_target_and_wider_than_the_sum() {
        assert!(1.0 / f64::from(BLEND_FIXED_POINT_SCALE) < 1.0 / 255.0);
        let widest = f64::from(u32::MAX) / f64::from(BLEND_FIXED_POINT_SCALE);
        let reachable = f64::from(MAX_LINE_LENGTH) * 2.0 * f64::from(MAX_BLEND_WEIGHT);
        assert!(
            widest > reachable,
            "a u32 holds {widest} weights of one and a pixel can be claimed by {reachable}"
        );
    }
}
