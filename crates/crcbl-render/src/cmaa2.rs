//! `docs/plan/49-antialiasing.md`'s antialiasing ladder, second rung: Intel's
//! Conservative Morphological Anti-Aliasing 2, which resolves the tonemapped
//! frame into the target.
//!
//! ```text
//!  begin_frame ──▶ extent ──▶ params[frame]
//!
//!  add_passes ── compute "cmaa2-clear"                  ──▶ control
//!             ── compute "cmaa2-edges"      display     ──▶ edges, candidates, accum
//!             ── compute "cmaa2-shapes"     edges       ──▶ items
//!             ── compute "cmaa2-accumulate" items       ──▶ accum
//!             ── draw    "cmaa2-apply"      accum       ──▶ target
//! ```
//!
//! # It takes the resolve slot, it does not stand beside FXAA
//!
//! [`crate::fxaa`]'s header describes that slot: the pass that reads what the
//! tonemap wrote and writes what the UI is composited onto, which is why
//! switching antialiasing on changes the *shape* of the frame rather than
//! adding a pass to it. This module is the same slot filled by a better and
//! more expensive filter. **The two are never both recorded** — a tier that is
//! off is a frame with fewer passes, which is what
//! [`RenderEffects`](crate::RenderEffects) means by a toggle — and
//! [`crate::forward`] owns that choice because it owns both ends.
//!
//! # Four dispatches and one draw, where SMAA was three draws
//!
//! The tier this replaced ran three fullscreen passes over every pixel and read
//! two committed lookup tables. This one carries no table and does its middle
//! work per **candidate** rather than per pixel: the edge detect appends every
//! edge pixel to a list, the classification runs once per entry of that list,
//! and the accumulate runs once per blend item those classifications produced.
//! A frame with few edges therefore costs little in the three passes between
//! the two that are per-pixel, which is the cost model
//! `docs/plan/49-antialiasing.md` chose it for.
//!
//! `crcbl_shaders::cmaa2` holds the constants and the capacities; the three
//! `cmaa2_*.slang` sources carry the algorithm and say which parts of the
//! reference they transcribe.
//!
//! # Only the last pass writes the target, and only it is a draw
//!
//! A caller's target is a swapchain image, and a swapchain image cannot be
//! bound as a storage image: there is no storage-image view of an sRGB format
//! on Vulkan, and WebGPU's storage-texture format list excludes every sRGB
//! encoding for the same reason. So the four compute stages leave their result
//! in a buffer and a fullscreen triangle writes the target, which is the shape
//! every other resolve in this engine has.
//!
//! # The buffers are the graph's, and every one of them is transient
//!
//! Five buffers, all sized from the frame's extent and all created through
//! [`RenderGraph::create_buffer`], on [`crate::ssao::Ssao::add_passes`]'s terms
//! for its images: a frame's resources belong to the graph, and a pool that
//! aliases them against other frames' is what keeps the tier's memory off a
//! frame that does not resolve through it. Nothing survives a frame — the
//! counters are zeroed by the first dispatch, the edge words are written whole
//! by the second, the accumulation is zeroed per pixel by that same dispatch,
//! and the two lists are only ever read below the counters that filled them.

use crcbl_hal::{
    BindGroupEntry, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutEntry,
    BindGroupLayoutHandle, BindingFlags, BindingKind, BindingResource, BufferDesc, BufferHandle,
    BufferUsage, ClearValue, ColorTargetState, ComputePipelineHandle, Device, Format,
    GraphicsPipelineHandle, HalError, ImageViewHandle, ImageViewType, LoadOp, MemoryLocation,
    PipelineLayoutDesc, PipelineLayoutHandle, ResourceState, SampleType, ShaderStages, StoreOp,
    check_portable_storage_buffers,
};
use crcbl_shaders::cmaa2::{
    ACCUM_WORDS, CONTROL_WORDS, Cmaa2Params, ITEM_WORDS, PARAMS_SIZE, WORD_BYTES, WORKGROUP_SIZE,
};
use crcbl_shaders::{CMAA2_APPLY, CMAA2_EDGES, CMAA2_SHAPES};

use std::cell::Cell;
use std::rc::Rc;

use crate::draw_gen::{bound, compute_pipeline_entry, storage, uniform};
use crate::graph::{ImageId, RenderGraph};
use crate::transient::TransientBufferDesc;

/// Vertices in the over-sized full-screen triangle `cmaa2_apply.slang`
/// generates from `SV_VertexID`. No geometry is bound anywhere.
const FULLSCREEN_VERTICES: u32 = 3;

/// What a bind group this module caches was built against.
///
/// The source view **and** the five buffers, because every one of them is a
/// graph transient: a pool that handed out a different physical buffer — after
/// a resize, or after the pool was destroyed and rebuilt — would leave a cached
/// group naming memory this frame does not own.
///
/// [`crate::ssao::cached_group`] is the same idea keyed on views alone, which
/// is all a pass whose buffers it owns itself needs. This tier owns none of
/// them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GroupKey {
    /// The tonemapped frame the tier reads.
    source: ImageViewHandle,
    /// `control`, `edges`, `candidates`, `accum`, `items` — in binding order.
    buffers: [BufferHandle; 5],
}

/// Everything the five passes own.
///
/// Built once by [`Cmaa2::new`] and released by [`Cmaa2::destroy`], which is the
/// shape every other resource group in this crate has — see [`crate::ssao`].
#[derive(Debug)]
pub(crate) struct Cmaa2 {
    /// The layout all four compute entry points bind, and the reason
    /// `cmaa2_edges.slang` and `cmaa2_shapes.slang` declare the same seven
    /// resources in the same order — see `crcbl_shaders::declaration_order`,
    /// which is what makes declaration order load-bearing on Metal.
    working_layout: BindGroupLayoutHandle,
    working_pipeline_layout: PipelineLayoutHandle,
    clear: ComputePipelineHandle,
    edges: ComputePipelineHandle,
    shapes: ComputePipelineHandle,
    accumulate: ComputePipelineHandle,
    /// The resolve's own layout: a fragment stage may not be handed the four
    /// writable storage buffers above, so it takes the accumulation read-only
    /// and nothing else.
    apply_layout: BindGroupLayoutHandle,
    apply_pipeline_layout: PipelineLayoutHandle,
    apply_pipeline: GraphicsPipelineHandle,
    /// `[frame]`: the block every pass reads. One buffer per frame in flight,
    /// and one *block* for the five passes — they run at one extent, and
    /// `crcbl_shaders::cmaa2::Cmaa2Params` says why that is one block rather
    /// than five that agree.
    uniforms: Vec<BufferHandle>,
    /// `[frame]`: the four dispatches' group, cached against everything it
    /// names.
    working_groups: Vec<Option<(GroupKey, BindGroupHandle)>>,
    /// `[frame]`: the resolve's group, cached the same way.
    apply_groups: Vec<Option<(GroupKey, BindGroupHandle)>>,
    /// What [`Cmaa2::begin_frame`] last wrote.
    ///
    /// Read by [`Cmaa2::add_passes`] so the buffers it creates and the
    /// dispatches it sizes are built from the **same** capacities the shaders
    /// read out of the block. Deriving them a second time from the extent would
    /// be two answers to one question, and
    /// [`capacity_cap`](Self::capacity_cap) is exactly the knob that would make
    /// them disagree.
    params: Cmaa2Params,
    /// An upper bound on both list capacities, or [`None`] for the frame's own.
    ///
    /// **A window for a test**, on [`crate::exposure::ExposureBuffers`]' terms:
    /// dropping entries past a full list is the one behaviour of this tier no
    /// picture shows, and the only way to reach it on a frame small enough to
    /// read back is to make the lists small.
    capacity_cap: Option<u32>,
}

/// The sampled-image entry the compute stages read the frame through.
fn sampled_compute(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        kind: BindingKind::SampledImage {
            view_type: ImageViewType::D2,
            sample_type: SampleType::Float,
        },
        count: 1,
        flags: BindingFlags::empty(),
    }
}

impl Cmaa2 {
    /// Passes [`Cmaa2::add_passes`] adds to a frame.
    pub(crate) const PASSES: u32 = 5;

    /// How many of those are full-screen **draws**, which is what
    /// [`crate::forward`]'s statistics count.
    ///
    /// One: the resolve. The four before it are compute dispatches and draw
    /// nothing, exactly as [`crate::exposure`]'s three do not.
    pub(crate) const FULLSCREEN_PASSES: u64 = 1;

    /// Builds the four compute pipelines, the resolve pipeline and the uniform
    /// ring.
    ///
    /// `build_fullscreen` is handed in rather than duplicated, on
    /// [`crate::fxaa::Fxaa::new`]'s terms exactly.
    ///
    /// `target_format` is what the resolve writes, which is the caller's
    /// target — and, because the tonemap writes an intermediate of the same
    /// description, also the format every pass here reads.
    ///
    /// # Errors
    ///
    /// [`HalError`] from any seam call. **Nothing is released on the failing
    /// path**, for the reason every other builder in this crate gives: the
    /// caller holds a rollback, and this is stored in it whole.
    pub(crate) fn new(
        device: &dyn Device,
        frames: usize,
        target_format: Format,
        build_fullscreen: impl Fn(
            &dyn Device,
            &str,
            &crcbl_shaders::Shader,
            PipelineLayoutHandle,
            &[ColorTargetState],
        ) -> Result<GraphicsPipelineHandle, HalError>,
    ) -> Result<Self, HalError> {
        // Declaration order is binding order in both of these, because the
        // sources declare their resources in the order they number them and
        // Slang's Metal target follows the declarations — see
        // `crcbl_shaders::declaration_order`.
        let working_entries = [
            uniform(0),
            sampled_compute(1),
            storage(2, false),
            storage(3, false),
            storage(4, false),
            storage(5, false),
            storage(6, false),
        ];
        let working_desc = BindGroupLayoutDesc {
            label: Some("cmaa2 working"),
            entries: &working_entries,
        };
        let apply_entries = [
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                kind: BindingKind::SampledImage {
                    view_type: ImageViewType::D2,
                    sample_type: SampleType::Float,
                },
                count: 1,
                flags: BindingFlags::empty(),
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                // **Read-only, and that is what makes it bindable here.** A
                // writable storage buffer may not be handed to a vertex stage
                // on WebGPU at all, and the resolve's layout serves both stages
                // of one pipeline — `crate::ssr` binds its light list on
                // exactly these terms.
                kind: BindingKind::StorageBuffer {
                    read_only: true,
                    dynamic: false,
                },
                count: 1,
                flags: BindingFlags::empty(),
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                kind: BindingKind::UniformBuffer { dynamic: false },
                count: 1,
                flags: BindingFlags::empty(),
            },
        ];
        let apply_desc = BindGroupLayoutDesc {
            label: Some("cmaa2 apply"),
            entries: &apply_entries,
        };
        // Five storage buffers in the widest of the two, against the eight a
        // WebGPU device guarantees per stage.
        check_portable_storage_buffers(Some("cmaa2"), &[&working_desc, &apply_desc])?;

        let working_layout = device.create_bind_group_layout(&working_desc)?;
        let working_set_layouts = [working_layout];
        let working_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDesc {
            label: Some("cmaa2 working"),
            bind_group_layouts: &working_set_layouts,
            push_constants: None,
        })?;
        // Two entry points per module, which is why these are
        // `compute_pipeline_entry` rather than `compute_pipeline`: each pair
        // shares its file's constants and shaders here have no `#include`.
        let clear = compute_pipeline_entry(
            device,
            "cmaa2 clear",
            &CMAA2_EDGES,
            "clearMain",
            working_pipeline_layout,
            WORKGROUP_SIZE,
        )?;
        let edges = compute_pipeline_entry(
            device,
            "cmaa2 edges",
            &CMAA2_EDGES,
            "edgesMain",
            working_pipeline_layout,
            WORKGROUP_SIZE,
        )?;
        let shapes = compute_pipeline_entry(
            device,
            "cmaa2 shapes",
            &CMAA2_SHAPES,
            "shapesMain",
            working_pipeline_layout,
            WORKGROUP_SIZE,
        )?;
        let accumulate = compute_pipeline_entry(
            device,
            "cmaa2 accumulate",
            &CMAA2_SHAPES,
            "accumulateMain",
            working_pipeline_layout,
            WORKGROUP_SIZE,
        )?;

        let apply_layout = device.create_bind_group_layout(&apply_desc)?;
        let apply_set_layouts = [apply_layout];
        let apply_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDesc {
            label: Some("cmaa2 apply"),
            bind_group_layouts: &apply_set_layouts,
            push_constants: None,
        })?;
        let apply_pipeline = build_fullscreen(
            device,
            "cmaa2-apply",
            &CMAA2_APPLY,
            apply_pipeline_layout,
            &[ColorTargetState::opaque(target_format)],
        )?;

        let mut uniforms = Vec::with_capacity(frames);
        for _ in 0..frames {
            uniforms.push(device.create_buffer(&BufferDesc {
                label: Some("cmaa2 params"),
                size: PARAMS_SIZE as u64,
                usage: BufferUsage::UNIFORM,
                memory: MemoryLocation::HostUpload,
            })?);
        }

        Ok(Self {
            working_layout,
            working_pipeline_layout,
            clear,
            edges,
            shapes,
            accumulate,
            apply_layout,
            apply_pipeline_layout,
            apply_pipeline,
            uniforms,
            working_groups: (0..frames).map(|_| None).collect(),
            apply_groups: (0..frames).map(|_| None).collect(),
            // Zero until a frame writes one, which is
            // `Cmaa2Params::default`'s tell rather than a plausible extent:
            // `add_passes` before `begin_frame` would create empty buffers and
            // the seam refuses those.
            params: Cmaa2Params::default(),
            capacity_cap: None,
        })
    }

    /// Caps both list capacities at `cap`, or restores the frame's own.
    ///
    /// See [`capacity_cap`](Self::capacity_cap) for why this exists.
    pub(crate) const fn set_capacity_cap(&mut self, cap: Option<u32>) {
        self.capacity_cap = cap;
    }

    /// Writes this frame's block and remembers the capacities in it.
    ///
    /// The extent is the frame's, and it is the one thing the shaders cannot
    /// derive — see [`Cmaa2Params::for_extent`], whose default leaves it zero
    /// precisely so an unwritten block draws a tell rather than something that
    /// nearly works.
    ///
    /// # Errors
    ///
    /// [`HalError`] from the mapped write.
    ///
    /// # Panics
    ///
    /// If `frame` is not a slot this was built with.
    pub(crate) fn begin_frame(
        &mut self,
        device: &dyn Device,
        frame: usize,
        extent: (u32, u32),
    ) -> Result<(), HalError> {
        let params = Cmaa2Params::for_extent(extent.0, extent.1);
        self.params = match self.capacity_cap {
            Some(cap) => params.with_capacity_cap(cap),
            None => params,
        };
        device.write_buffer(self.uniforms[frame], 0, &self.params.to_bytes())
    }

    /// Adds the four dispatches and the resolve: `source` through the edge
    /// detect, the classification and the accumulate into `target`.
    ///
    /// # Panics
    ///
    /// If `frame` is not a slot this was built with.
    pub(crate) fn add_passes<'a>(
        &'a mut self,
        graph: &mut RenderGraph<'a>,
        frame: usize,
        source: ImageId,
        target: ImageId,
    ) {
        let params = self.params;
        let pixels = u64::from(params.viewport_x) * u64::from(params.viewport_y);
        let control = graph.create_buffer(
            "cmaa2-control",
            TransientBufferDesc::storage(u64::from(CONTROL_WORDS) * WORD_BYTES),
        );
        let edge_words = graph.create_buffer(
            "cmaa2-edges",
            TransientBufferDesc::storage(pixels * WORD_BYTES),
        );
        let candidates = graph.create_buffer(
            "cmaa2-candidates",
            TransientBufferDesc::storage(u64::from(params.candidate_capacity) * WORD_BYTES),
        );
        let items = graph.create_buffer(
            "cmaa2-items",
            TransientBufferDesc::storage(
                u64::from(params.item_capacity) * u64::from(ITEM_WORDS) * WORD_BYTES,
            ),
        );
        let accum = graph.create_buffer(
            "cmaa2-accum",
            TransientBufferDesc::storage(pixels * u64::from(ACCUM_WORDS) * WORD_BYTES),
        );

        let uniforms = self.uniforms[frame];
        let working_layout = self.working_layout;
        let working_pipeline_layout = self.working_pipeline_layout;
        let apply_layout = self.apply_layout;
        let apply_pipeline_layout = self.apply_pipeline_layout;
        let apply_pipeline = self.apply_pipeline;
        let (clear, edges, shapes, accumulate) =
            (self.clear, self.edges, self.shapes, self.accumulate);
        // Split so the pass bodies below borrow different halves of `self`; one
        // `&mut self` shared between them is what the borrow checker refuses,
        // and it would be refusing something genuinely wrong — a pass body may
        // run at any point after it is declared. `crate::ssao` splits its two
        // the same way.
        let Self {
            working_groups,
            apply_groups,
            ..
        } = self;
        let working_cached = &mut working_groups[frame];
        let apply_cached = &mut apply_groups[frame];

        // One group for all four dispatches, built inside the first of them: it
        // names the tonemapped frame and five graph buffers, none of which has a
        // handle until the graph has realised one. A shared `Cell` carries it to
        // the three passes after this one, on [`crate::exposure`]'s terms — the
        // bodies run synchronously and in order on this thread, and a body that
        // finds no group records no dispatch rather than binding a stale one.
        let working: Rc<Cell<Option<BindGroupHandle>>> = Rc::new(Cell::new(None));
        let build = Rc::clone(&working);
        graph
            .add_compute_pass("cmaa2-clear")
            // `ShaderReadWrite` rather than a write-only state, on the light
            // grid's terms: a storage-buffer descriptor permits reads whatever
            // the shader does with it.
            .use_buffer(control, ResourceState::ShaderReadWrite)
            .use_buffer(edge_words, ResourceState::ShaderReadWrite)
            .use_buffer(candidates, ResourceState::ShaderReadWrite)
            .use_buffer(items, ResourceState::ShaderReadWrite)
            .use_buffer(accum, ResourceState::ShaderReadWrite)
            // The image the pass after this one reads. Declared here as well
            // because the group naming it is built in this pass, and a view is
            // only realised for a pass that declared the image.
            .read_image(source)
            .execute(move |ctx| {
                let view = ctx.image_view(source);
                let handles = [
                    ctx.buffer(control),
                    ctx.buffer(edge_words),
                    ctx.buffer(candidates),
                    ctx.buffer(accum),
                    ctx.buffer(items),
                ];
                let entries = vec![
                    bound(0, uniforms),
                    BindGroupEntry {
                        binding: 1,
                        array_index: 0,
                        resource: BindingResource::ImageView(view),
                    },
                    bound(2, handles[0]),
                    bound(3, handles[1]),
                    bound(4, handles[2]),
                    bound(5, handles[3]),
                    bound(6, handles[4]),
                ];
                let Some(group) = cached_group(
                    working_cached,
                    ctx.device(),
                    GroupKey {
                        source: view,
                        buffers: handles,
                    },
                    "cmaa2 working",
                    working_layout,
                    entries,
                ) else {
                    return;
                };
                build.set(Some(group));
                let encoder = ctx.encoder();
                encoder.bind_compute_pipeline(clear);
                encoder.bind_group(0, group, &[], working_pipeline_layout);
                // One group: the control buffer is `CONTROL_WORDS` long and the
                // shader guards the tail of it.
                encoder.dispatch(1, 1, 1);
            });

        // One invocation per texel, and never zero: `Cmaa2Params::for_extent`
        // floors the extent at one, and Metal rejects an empty dispatch outright
        // rather than treating it as a no-op.
        let pixel_groups = groups_for(params.viewport_x.saturating_mul(params.viewport_y));
        let bind = Rc::clone(&working);
        graph
            .add_compute_pass("cmaa2-edges")
            .use_buffer(control, ResourceState::ShaderReadWrite)
            .use_buffer(edge_words, ResourceState::ShaderReadWrite)
            .use_buffer(candidates, ResourceState::ShaderReadWrite)
            .use_buffer(accum, ResourceState::ShaderReadWrite)
            .read_image(source)
            .execute(move |ctx| {
                let Some(group) = bind.get() else {
                    return;
                };
                let encoder = ctx.encoder();
                encoder.bind_compute_pipeline(edges);
                encoder.bind_group(0, group, &[], working_pipeline_layout);
                encoder.dispatch(pixel_groups, 1, 1);
            });

        // One invocation per candidate **slot**, not per candidate: how many
        // there are is a number only the device knows, and reading it back to
        // size a dispatch is the stall this whole tier is built to avoid. The
        // shader exits immediately on a slot past the count.
        let candidate_groups = groups_for(params.candidate_capacity);
        let bind = Rc::clone(&working);
        graph
            .add_compute_pass("cmaa2-shapes")
            .use_buffer(control, ResourceState::ShaderReadWrite)
            .use_buffer(edge_words, ResourceState::ShaderReadWrite)
            .use_buffer(candidates, ResourceState::ShaderReadWrite)
            .use_buffer(items, ResourceState::ShaderReadWrite)
            .execute(move |ctx| {
                let Some(group) = bind.get() else {
                    return;
                };
                let encoder = ctx.encoder();
                encoder.bind_compute_pipeline(shapes);
                encoder.bind_group(0, group, &[], working_pipeline_layout);
                encoder.dispatch(candidate_groups, 1, 1);
            });

        // One invocation per item slot, on the pass above's terms.
        let item_groups = groups_for(params.item_capacity);
        let bind = working;
        graph
            .add_compute_pass("cmaa2-accumulate")
            .use_buffer(control, ResourceState::ShaderReadWrite)
            .use_buffer(items, ResourceState::ShaderReadWrite)
            .use_buffer(accum, ResourceState::ShaderReadWrite)
            .read_image(source)
            .execute(move |ctx| {
                let Some(group) = bind.get() else {
                    return;
                };
                let encoder = ctx.encoder();
                encoder.bind_compute_pipeline(accumulate);
                encoder.bind_group(0, group, &[], working_pipeline_layout);
                encoder.dispatch(item_groups, 1, 1);
            });

        graph
            .add_render_pass("cmaa2-apply")
            // `DontCare`, not `Clear`: the full-screen triangle writes every
            // pixel of the target — a pixel no blend item reached is written
            // with its own colour rather than skipped — so loading or clearing
            // it is pure bandwidth.
            .color(
                target,
                LoadOp::DontCare,
                StoreOp::Store,
                ClearValue::default(),
            )
            .read_image(source)
            // The accumulate's own output, read here — the graph's barrier
            // between the two comes from both declaring this one id.
            .read_buffer(accum)
            .execute(move |ctx| {
                let view = ctx.image_view(source);
                let accumulation = ctx.buffer(accum);
                let entries = vec![
                    BindGroupEntry {
                        binding: 0,
                        array_index: 0,
                        resource: BindingResource::ImageView(view),
                    },
                    bound(1, accumulation),
                    bound(2, uniforms),
                ];
                let Some(group) = cached_group(
                    apply_cached,
                    ctx.device(),
                    GroupKey {
                        source: view,
                        // Only the accumulation is this group's; the four
                        // repeats keep one key type across both caches, and
                        // comparing a handle against itself costs nothing.
                        buffers: [accumulation; 5],
                    },
                    "cmaa2 apply",
                    apply_layout,
                    entries,
                ) else {
                    return;
                };
                let encoder = ctx.encoder();
                encoder.bind_graphics_pipeline(apply_pipeline);
                encoder.bind_group(0, group, &[], apply_pipeline_layout);
                encoder.draw(0..FULLSCREEN_VERTICES, 0..1);
            });
    }

    /// Releases everything, in dependency order. The device must be idle.
    pub(crate) fn destroy(self, device: &dyn Device) {
        for (_, group) in self
            .working_groups
            .into_iter()
            .chain(self.apply_groups)
            .flatten()
        {
            device.destroy_bind_group(group);
        }
        device.destroy_graphics_pipeline(self.apply_pipeline);
        device.destroy_compute_pipeline(self.accumulate);
        device.destroy_compute_pipeline(self.shapes);
        device.destroy_compute_pipeline(self.edges);
        device.destroy_compute_pipeline(self.clear);
        device.destroy_pipeline_layout(self.apply_pipeline_layout);
        device.destroy_pipeline_layout(self.working_pipeline_layout);
        device.destroy_bind_group_layout(self.apply_layout);
        device.destroy_bind_group_layout(self.working_layout);
        for buffer in self.uniforms {
            device.destroy_buffer(buffer);
        }
    }
}

/// Workgroups covering `invocations`, never fewer than one.
///
/// The floor is not defensive rounding: Metal rejects a dispatch of no groups
/// outright rather than treating it as a no-op, and every capacity this module
/// sizes a dispatch from is floored well above zero anyway — see
/// `crcbl_shaders::cmaa2::MIN_LIST_CAPACITY`.
fn groups_for(invocations: u32) -> u32 {
    invocations.div_ceil(WORKGROUP_SIZE).max(1)
}

/// [`crate::ssao::cached_group`] keyed on a [`GroupKey`] rather than on views
/// alone.
///
/// A separate function rather than a parameter on that one, because the two
/// answer different questions: a pass that owns its buffers only has to notice
/// a resize, and this tier owns none of them — see [`GroupKey`].
fn cached_group(
    cache: &mut Option<(GroupKey, BindGroupHandle)>,
    device: &dyn Device,
    key: GroupKey,
    label: &str,
    layout: BindGroupLayoutHandle,
    entries: Vec<BindGroupEntry>,
) -> Option<BindGroupHandle> {
    if let Some((cached, group)) = cache
        && *cached == key
    {
        return Some(*group);
    }
    if let Some((_, stale)) = cache.take() {
        device.destroy_bind_group(stale);
    }
    match device.create_bind_group(&crcbl_hal::BindGroupDesc {
        label: Some(label),
        layout,
        entries: &entries,
        variable_count: None,
    }) {
        Ok(group) => {
            *cache = Some((key, group));
            Some(group)
        }
        Err(error) => {
            crcbl_core::log::error!("graph: {label} bind group failed: {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dispatch is never empty, whatever the capacity.
    #[test]
    fn a_dispatch_always_covers_at_least_one_group() {
        assert_eq!(groups_for(0), 1);
        assert_eq!(groups_for(1), 1);
        assert_eq!(groups_for(WORKGROUP_SIZE), 1);
        assert_eq!(groups_for(WORKGROUP_SIZE + 1), 2);
    }

    /// The five buffers this tier creates are sized from the block the shaders
    /// read, so a capacity the block carries and a buffer that holds fewer
    /// entries cannot come apart.
    ///
    /// The arithmetic rather than the buffers, because a `RenderGraph` needs a
    /// pool and a device to realise one — what can go wrong here is a factor,
    /// and that is what this pins.
    #[test]
    fn every_buffer_is_sized_from_the_block_the_shaders_read() {
        let params = Cmaa2Params::for_extent(320, 240);
        let pixels = u64::from(params.viewport_x) * u64::from(params.viewport_y);
        assert_eq!(pixels * WORD_BYTES, 320 * 240 * 4);
        assert_eq!(
            pixels * u64::from(ACCUM_WORDS) * WORD_BYTES,
            320 * 240 * 4 * 4
        );
        assert_eq!(
            u64::from(params.item_capacity) * u64::from(ITEM_WORDS) * WORD_BYTES,
            u64::from(320u32 * 240 / 4) * 3 * 4
        );
        assert_eq!(
            u64::from(params.candidate_capacity) * WORD_BYTES,
            u64::from(320u32 * 240 / 8) * 4
        );
    }
}
