//! One camera's share of the forward frame.
//!
//! [`ForwardRenderer`] owns two kinds of state, and this module is the line
//! between them. The **scene** is what every camera draws: the geometry and
//! instance pools, the materials and their pages, the probes, the shadow atlas
//! and everything that decides what it holds. A **view** is what one camera
//! needs to turn that scene into a picture: its own cull and the survivor lists
//! it writes, its frame block, its light clustering, its bind groups and the
//! ring of per-frame blocks every screen-space pass keeps.
//!
//! # Why the line is here
//!
//! Everything on this side is either sized by what one camera sees or carries
//! one camera's history from a frame to the next. `docs/plan/25-lod.md`'s
//! hysteresis lives in [`DrawGen::group_state`], so two cameras sharing one
//! generator would each undo the other's cut. Auto-exposure reads the previous
//! slot's measurement, and [`View::previous_view_projection`] is where the
//! motion vectors come from. Every one of those is wrong the moment a second
//! camera writes it.
//!
//! Nothing on the other side is. An instance is where it is whoever looks at
//! it, and the shadow atlas is drawn once for the frame.

use super::*;

/// Everything [`View::build`] reads out of the scene it draws.
///
/// Handles and borrowed tables, and nothing a view owns: each is created and
/// destroyed by [`ForwardRenderer`], which outlives every view built against
/// it.
pub(super) struct ViewInputs<'a> {
    /// The format the caller's target has, which the resolves and the upscale
    /// write.
    pub(super) target_format: Format,
    /// Which call the forward pass records.
    pub(super) emit: EmitTail,
    /// The instance ring, one buffer per frame in flight.
    pub(super) instances: &'a [BufferHandle],
    /// The mesh table's buffer.
    pub(super) mesh_table: BufferHandle,
    /// The bucket and level tables every generator of the scene is built from
    /// — see [`DrawGenDesc`].
    pub(super) bucket_meshes: &'a [u32],
    pub(super) bucket_modes: &'a [u32],
    pub(super) bucket_clusters: &'a [u32],
    pub(super) mesh_levels: &'a [level_select::MeshLevels],
    pub(super) level_groups: &'a [level_select::LevelGroup],
    pub(super) level_meshes: &'a [u32],
    /// [`SceneDesc::capacities`]' instance and light counts.
    pub(super) instance_capacity: u32,
    pub(super) light_capacity: u32,
    /// §3.5's clusters, on the mesh path only.
    pub(super) clusters: Option<&'a ClusterPool>,
    /// The mesh layout every group below is built against.
    pub(super) mesh_layout: BindGroupLayoutHandle,
    /// The scene's half of every group — see [`SharedBindings`]. The page
    /// sampler is per slot, because [`ForwardRenderer::adopt_page_sampler`]
    /// moves each slot to a new one on that slot's own frame.
    pub(super) vertices: BufferHandle,
    pub(super) draw_constants: BufferHandle,
    pub(super) materials: BufferHandle,
    pub(super) page: ImageViewHandle,
    pub(super) normal_page: ImageViewHandle,
    pub(super) mro_page: ImageViewHandle,
    pub(super) emissive_page: ImageViewHandle,
    pub(super) page_samplers: &'a [SamplerHandle],
    pub(super) probes: &'a [BufferHandle],
    pub(super) specular_dfg: ImageViewHandle,
    pub(super) ltc_table: ImageViewHandle,
    /// The shadow atlas and the sampler that compares against it.
    pub(super) shadow_map: ImageViewHandle,
    pub(super) shadow_sampler: SamplerHandle,
    /// The stand-ins a group names until the forward pass rebuilds it against
    /// the frame's own images.
    pub(super) ambient_occlusion: ImageViewHandle,
    pub(super) contact_shadow: ImageViewHandle,
    pub(super) probe_visibility: ImageViewHandle,
}

/// One camera's resources and history — see the [module docs](self).
#[derive(Debug)]
pub(super) struct View {
    /// The cull and draw-argument passes, and the indirect arguments they
    /// produce.
    pub(super) draws: DrawGen,
    /// One buffer per frame in flight holding the cut the descent chose, or
    /// empty where there is no amplification stage to choose one. See
    /// [`ForwardRenderer::cluster_selection`].
    pub(super) cluster_selection: Vec<BufferHandle>,
    /// What [`begin_frame`](ForwardRenderer::begin_frame) last handed
    /// [`DrawGen::begin_frame`], kept so a reader can compute the same cut
    /// host-side without re-deriving it from the camera.
    ///
    /// Pixels per unit, the budget a group starts expanding over, and the budget
    /// it is held down to — `docs/plan/25-lod.md`'s hysteresis, and
    /// [`LOD_HOLD_RATIO`] is what puts the third below the second.
    pub(super) lod_params: [f32; 3],
    /// Topic 18's light list and froxel grid, and the compute pass between them.
    pub(super) lights: LightGrid,
    /// This frame's froxel grid, as [`begin_frame`](ForwardRenderer::begin_frame)
    /// decided it from the viewport and the camera.
    ///
    /// Held rather than recomputed at [`ForwardRenderer::add_passes`], because
    /// the number of froxels the dispatch covers and the number the frame block
    /// tells the fragment stage about have to be the same one.
    pub(super) grid: Grid,
    /// `[frame]`: the frame block, one per frame in flight — see the
    /// [forward module docs](super) on why it is a ring.
    pub(super) uniforms: Vec<BufferHandle>,
    /// `[frame]`: the camera's group of the mesh layout, naming this view's
    /// frame block and survivors.
    pub(super) mesh_groups: Vec<BindGroupHandle>,
    /// `[frame]`: `tonemap.slang`'s exposure block, written by
    /// [`begin_frame`](ForwardRenderer::begin_frame).
    ///
    /// One per frame in flight for the frame uniforms' reason exactly — the
    /// previous frame may still be reading last frame's while this one is
    /// written.
    pub(super) tonemap_uniforms: Vec<BufferHandle>,
    /// `[frame]`: the tonemap group, cached against the scene target's view.
    ///
    /// Rebuilt only when that view changes, which is only on a resize. The graph
    /// hands the view to the pass body; caching against it is what keeps a
    /// steady-state frame free of descriptor writes.
    ///
    /// **One per frame in flight**, for [`crate::ssao`]'s reason: this group
    /// names [`View::tonemap_uniforms`] as well as the scene transient, and that
    /// is a ring — a single cache keyed on the view alone would hand the even
    /// frames' block to the odd frames.
    pub(super) tonemap_groups: Vec<Option<(Vec<ImageViewHandle>, BindGroupHandle)>>,
    /// `[frame]`: the entries [`View::mesh_groups`] was built from.
    ///
    /// Kept because the occlusion image is a graph transient: its view is known
    /// only at execute time, so the camera's group has to be rebuilt inside the
    /// forward pass, and re-deriving twenty bindings there would mean carrying
    /// half of `build`'s locals into the frame. Exactly two entries —
    /// [`AMBIENT_OCCLUSION_BINDING`]'s and [`CONTACT_SHADOW_BINDING`]'s — differ
    /// between the stored list and what the rebuild writes.
    pub(super) mesh_group_entries: Vec<Vec<BindGroupEntry>>,
    /// `[frame]`: the entries [`View::prepass_groups`] was built from.
    ///
    /// Kept for one rebuild only: the page sampler's, which
    /// [`ForwardRenderer::adopt_page_sampler`] performs on every group of the
    /// mesh layout a slot holds. Handles, so the cost is a few words a group.
    pub(super) prepass_group_entries: Vec<Vec<BindGroupEntry>>,
    /// `[frame]`: the camera's group rebuilt against the two screen-space
    /// channels the forward pass reads — the blurred occlusion and the contact
    /// shadow — cached against both views together.
    ///
    /// [`View::tonemap_groups`]' shape, one per frame in flight because the
    /// group it replaces is per frame in flight. Rebuilt only when a view
    /// changes, which is only on a resize or a toggle.
    ///
    /// **One cache for both channels rather than one each**, because they are
    /// two bindings of *one* group: rebuilding it twice a frame would be a
    /// descriptor write per pass per frame, which is what
    /// [`crate::ssao::cached_group`] exists to avoid. Its key is every view, so
    /// either one moving is a miss.
    ///
    /// [`View::mesh_groups`] is the fallback and is *not* dead weight: it is
    /// what the depth prepass binds, because that pass runs before there is any
    /// occlusion to name.
    pub(super) screen_channel_groups: Vec<Option<(Vec<ImageViewHandle>, BindGroupHandle)>>,
    /// `[frame]`: the depth prepass's group — the camera's, with the occlusion
    /// placeholder and **a culling-statistics buffer of its own**.
    ///
    /// The second half is the whole reason this is a group rather than
    /// [`View::mesh_groups`] reused. On the mesh-shader path the prepass runs
    /// the same amplification stage the forward pass does, and that stage counts
    /// every surviving cluster into the buffer bound at binding 14 — so sharing
    /// the camera's would make
    /// [`CullStats::clusters`](crate::cull_stats::CullStats::clusters) report
    /// every cluster of the frame twice, which is a plausible number and a wrong
    /// one.
    ///
    /// **Nothing reads what this counts and nothing clears it.** It is a sink: a
    /// wrapping `u32` whose value is never looked at, which is the honest price
    /// of a prepass that shares a pipeline with the pass it precedes.
    pub(super) prepass_groups: Vec<BindGroupHandle>,
    /// `[frame]`: the sink [`View::prepass_groups`] counts into.
    ///
    /// Held so the prepass can declare it and the graph can barrier it. A ring
    /// rather than one buffer for every other per-frame resource's reason: the
    /// previous frame's submission may still be writing last frame's.
    pub(super) prepass_stats: Vec<BufferHandle>,
    /// This frame's camera view-projection, as
    /// [`begin_frame`](ForwardRenderer::begin_frame) computed it.
    ///
    /// Kept because the ground grid's pass needs it and `add_passes` has no
    /// camera: recomputing it there would be a second `aspect` to get wrong, and
    /// a grid drawn through a camera the frame is not drawn with lands on the
    /// wrong pixels while still looking like a grid.
    pub(super) camera_view_proj: Mat4,
    /// The view-projection the **previous** frame was drawn with, or [`None`]
    /// before there was one.
    ///
    /// The camera-side twin of
    /// [`GpuInstance::previous_transform`](crcbl_shaders::mesh::GpuInstance::previous_transform):
    /// the pool says where each object was and this says where the viewer was,
    /// and `mesh.slang`'s `motion_vector` is what turns the pair into a screen
    /// offset. It reaches the shader as
    /// [`FrameUniforms::previous_view_proj`](crcbl_shaders::mesh::FrameUniforms::previous_view_proj).
    ///
    /// **Advanced once per frame, in
    /// [`begin_frame_body`](ForwardRenderer::begin_frame_body)** — which runs
    /// exactly once per [`InstancePool::rotate`], so the camera's history and
    /// the instances' settle on the same boundary. Advancing it anywhere a
    /// frame can reach twice would report a camera that moved half as far as it
    /// did, and a still scene would never come back to rest.
    ///
    /// [`None`] means the first frame, which is drawn with its own matrix in
    /// both slots: a camera that has not moved yet has not moved, and an
    /// identity or a zero here would put every pixel of the first frame in
    /// motion.
    pub(super) previous_view_projection: Option<Mat4>,
    /// `docs/plan/18-render-features.md`'s occlusion pair — see [`crate::ssao`].
    pub(super) ssao: Ssao,
    /// `docs/plan/45-shadows.md`'s contact-shadow march — see
    /// [`crate::contact_shadows`].
    pub(super) contact_shadows: ContactShadows,
    /// `docs/plan/18-render-features.md`'s depth pyramid, which the reflection
    /// march climbs — see [`crate::hiz`].
    pub(super) hiz: Hiz,
    /// `docs/plan/18-render-features.md`'s reflection march — see
    /// [`crate::ssr`].
    pub(super) ssr: Ssr,
    /// `docs/plan/51-volumetrics.md`'s froxel volume and its composite — see
    /// [`crate::volumetric`].
    pub(super) volumetric: Volumetric,
    /// `docs/plan/43-render-standards.md` §6's auto-exposure — see
    /// [`crate::exposure`]. Named for what it owns rather than for the value:
    /// [`ForwardRenderer::exposure`] is the number a caller set.
    pub(super) auto_exposure: Exposure,
    /// `docs/plan/18-render-features.md`'s bloom chain — see [`crate::bloom`].
    pub(super) bloom: Bloom,
    /// `docs/plan/49-antialiasing.md`'s cheap antialiasing tier — see
    /// [`crate::fxaa`].
    pub(super) fxaa: Fxaa,
    /// `docs/plan/49-antialiasing.md`'s higher antialiasing tier — see
    /// [`crate::cmaa2`]. It takes the resolve slot from [`View::fxaa`] on the
    /// frames [`RenderEffects::CMAA2`] is set for, and neither is built per
    /// frame: both exist, and at most one records.
    pub(super) cmaa2: Cmaa2,
    /// [`crate::upscale`], and it draws nothing at a
    /// [`render_scale`](ForwardRenderer::render_scale) of `1.0`.
    pub(super) upscale: Upscale,
    /// `docs/plan/43-render-standards.md` §8's background pass — see
    /// [`crate::sky_pass`]. It draws on no frame whose sky is [`Sky::NONE`],
    /// which is every frame until a caller calls
    /// [`set_sky`](ForwardRenderer::set_sky).
    pub(super) sky_pass: SkyPass,
}

impl View {
    /// Creates one camera's resources against the scene `inputs` describes.
    ///
    /// **All or nothing**: a failure part-way releases what this call had
    /// created, so the caller has nothing of the view to clean up, and a success
    /// hands every handle to the returned [`View`] — none of them is left in a
    /// rollback the caller might run later.
    ///
    /// # Errors
    ///
    /// [`HalError`] if any buffer, group or pipeline could not be created.
    pub(super) fn build(
        device: &dyn Device,
        queue: QueueHandle,
        inputs: &ViewInputs<'_>,
    ) -> Result<Self, HalError> {
        let mut rollback = Rollback::default();
        let built = Self::build_into(device, queue, inputs, &mut rollback);
        if built.is_err() {
            rollback.run(device);
        }
        built
    }

    /// [`View::build`]'s body, with every handle it creates placed in
    /// `rollback` until the view is whole.
    fn build_into(
        device: &dyn Device,
        queue: QueueHandle,
        inputs: &ViewInputs<'_>,
        rollback: &mut Rollback,
    ) -> Result<Self, HalError> {
        let frames = inputs.instances.len();
        let draws = DrawGen::new(
            device,
            queue,
            &DrawGenDesc {
                label: Some("forward"),
                instances: inputs.instances,
                mesh_table: inputs.mesh_table,
                bucket_meshes: inputs.bucket_meshes,
                bucket_modes: inputs.bucket_modes,
                bucket_clusters: inputs.bucket_clusters,
                mesh_levels: inputs.mesh_levels,
                level_groups: inputs.level_groups,
                level_meshes: inputs.level_meshes,
                instance_capacity: inputs.instance_capacity,
            },
        )?;
        let runs: Vec<BufferHandle> = (0..frames).map(|frame| draws.runs(frame)).collect();
        let args: Vec<BufferHandle> = (0..frames).map(|frame| draws.args(frame)).collect();
        // What the amplification stage reads and writes, and nothing else does:
        // this frame's frustum, and the culling statistics its surviving
        // clusters are counted into.
        let cull_params: Vec<BufferHandle> =
            (0..frames).map(|frame| draws.cull_params(frame)).collect();
        let cull_stats: Vec<BufferHandle> = (0..frames)
            .map(|frame| draws.visible_count(frame))
            .collect();
        rollback.draws = Some(draws);

        // `docs/plan/25-lod.md`'s observable: one word per resident cluster,
        // holding the cut the descent chose. Empty where there is no
        // amplification stage, which is the same condition binding 18 exists
        // under — and the two cannot disagree, because this vector is what
        // decides whether the entry is written.
        //
        // **One buffer per frame in flight**, on `cull_stats`' terms exactly: a
        // frame still in flight is a frame still writing, and one buffer shared
        // across the ring would have the next frame's dispatch overwriting what
        // this one recorded. `TRANSFER_SRC` because reading it is the point.
        //
        // Allocated on the whole mesh path rather than only where there is an
        // amplification stage to write them, because the layout declares
        // binding 18 there — see the layout, which is where that is argued. On
        // a device with no task stage nothing writes them and
        // `ForwardRenderer::cluster_selection` still answers `None`, so the
        // cost is the allocation and nothing else.
        let mut cluster_selection: Vec<BufferHandle> = Vec::new();
        if inputs.emit.is_mesh() {
            let count = inputs
                .clusters
                .unwrap_or_else(|| unreachable!("the mesh path implies a cluster pool"))
                .count();
            for frame in 0..frames {
                let buffer = device.create_buffer(&BufferDesc {
                    label: Some(&format!("cluster selection {frame}")),
                    size: u64::from(count) * 4,
                    usage: BufferUsage::STORAGE.union(BufferUsage::TRANSFER_SRC),
                    memory: MemoryLocation::DeviceLocal,
                })?;
                rollback.buffers.push(buffer);
                cluster_selection.push(buffer);
            }
        }

        // Topic 18's light list and the froxel grid its compute pass fills.
        //
        // Built after the cull because it needs the culling-statistics ring:
        // its overflow counter is a word of that buffer, which is what keeps
        // topic 03 §3.6's readback at one.
        rollback.lights = Some(LightGrid::new(
            device,
            &LightGridDesc {
                label: Some("lights"),
                frames,
                lights: inputs.light_capacity,
                froxels: FROXEL_CAPACITY,
                stats: &cull_stats,
            },
        )?);
        let lights = rollback.lights.as_ref().expect("just stored");
        let draws = rollback.draws.as_ref().expect("stored above");

        let mut uniforms = Vec::with_capacity(frames);
        let mut mesh_groups = Vec::with_capacity(frames);
        let mut mesh_group_entries = Vec::with_capacity(frames);
        let mut prepass_groups = Vec::with_capacity(frames);
        let mut prepass_stats = Vec::with_capacity(frames);
        let mut prepass_group_entries = Vec::with_capacity(frames);
        for (frame, &slot_instances) in inputs.instances.iter().enumerate() {
            // Everything a group of this layout names that is the same in all of
            // this frame's. The per-group half is what `MeshGroup` below varies,
            // and the two exist so the colour pass's group and the shadow pass's
            // are one description rather than two that agree today.
            let shared = SharedBindings {
                vertices: inputs.vertices,
                draw_constants: inputs.draw_constants,
                mesh_table: inputs.mesh_table,
                materials: inputs.materials,
                page: inputs.page,
                normal_page: inputs.normal_page,
                mro_page: inputs.mro_page,
                emissive_page: inputs.emissive_page,
                page_sampler: inputs.page_samplers[frame],
                clusters: inputs.clusters,
                shadow_sampler: inputs.shadow_sampler,
                lights: lights.lights(frame),
                light_grid: lights.grid(frame),
                probes: inputs.probes[frame],
                tables: draws.tables(),
                specular_dfg: inputs.specular_dfg,
                ltc_table: inputs.ltc_table,
            };
            let buffer = device.create_buffer(&BufferDesc {
                label: Some("mesh frame uniforms"),
                size: mesh::FRAME_UNIFORMS_SIZE as u64,
                usage: BufferUsage::UNIFORM,
                memory: MemoryLocation::HostUpload,
            })?;
            rollback.buffers.push(buffer);
            let entries = MeshGroup {
                uniforms: buffer,
                instances: slot_instances,
                runs: runs[frame],
                args: args[frame],
                cull_params: cull_params[frame],
                cull_stats: cull_stats[frame],
                cluster_selection: cluster_selection.get(frame).copied(),
                group_state: inputs.emit.is_mesh().then(|| draws.group_state()),
                // The colour pass reads the finished atlas. Its own pass writes
                // nothing to it, so there is no conflict to avoid here.
                shadow_map: inputs.shadow_map,
                // The placeholder even for the camera's group: the occlusion
                // image is a graph transient and its view does not exist until
                // execute time. `add_passes` rebuilds this group against the real
                // one and caches it, and *this* group is what the depth prepass
                // binds — which runs before there is any occlusion to name.
                ambient_occlusion: inputs.ambient_occlusion,
                // The same, one binding along, and for the same reason.
                contact_shadow: inputs.contact_shadow,
                probe_visibility: inputs.probe_visibility,
            }
            .entries(&shared);
            let group = device.create_bind_group(&BindGroupDesc {
                label: Some("mesh frame"),
                layout: inputs.mesh_layout,
                entries: &entries,
                variable_count: None,
            })?;
            rollback.bind_groups.push(group);
            uniforms.push(buffer);
            mesh_groups.push(group);
            // Kept so the forward pass can rebuild this group against the
            // occlusion image the graph realised, without re-deriving twenty
            // bindings out of fields that no longer exist by then. Only the
            // screen-space channels and the probe visibility maps differ — see
            // [`View::screen_channel_groups`].
            mesh_group_entries.push(entries);

            // The depth prepass's group: this one again, counting its clusters
            // somewhere the camera's counter cannot see. See
            // [`View::prepass_groups`] for why that matters, and note that
            // binding 14 exists at all only where there is an amplification
            // stage — so on every other path this buffer is bound nowhere and the
            // group is the camera's under another handle.
            //
            // `DeviceLocal`, because a shader writes it: D3D12 has no unordered
            // access view of a host-visible resource, and `create_bind_group`
            // enforces it.
            let stats = device.create_buffer(&BufferDesc {
                label: Some("depth prepass cluster survivors"),
                size: u64::from(crcbl_shaders::cull::STATS_WORDS) * 4,
                usage: BufferUsage::STORAGE,
                memory: MemoryLocation::DeviceLocal,
            })?;
            rollback.buffers.push(stats);
            let entries = MeshGroup {
                uniforms: buffer,
                instances: slot_instances,
                runs: runs[frame],
                args: args[frame],
                cull_params: cull_params[frame],
                cull_stats: stats,
                cluster_selection: cluster_selection.get(frame).copied(),
                group_state: inputs.emit.is_mesh().then(|| draws.group_state()),
                shadow_map: inputs.shadow_map,
                ambient_occlusion: inputs.ambient_occlusion,
                contact_shadow: inputs.contact_shadow,
                probe_visibility: inputs.probe_visibility,
            }
            .entries(&shared);
            let group = device.create_bind_group(&BindGroupDesc {
                label: Some("depth prepass"),
                layout: inputs.mesh_layout,
                entries: &entries,
                variable_count: None,
            })?;
            rollback.bind_groups.push(group);
            prepass_groups.push(group);
            prepass_stats.push(stats);
            // For the page sampler's rebuild — see [`View::prepass_group_entries`].
            prepass_group_entries.push(entries);
        }

        // The exposure block, one per frame in flight for the frame uniforms'
        // reason exactly — the previous frame may still be reading last frame's
        // while this one is written. See the forward module docs on the ring.
        let mut tonemap_uniforms = Vec::with_capacity(frames);
        for _ in 0..frames {
            let buffer = device.create_buffer(&BufferDesc {
                label: Some("tonemap params"),
                size: tonemap::PARAMS_SIZE as u64,
                usage: BufferUsage::UNIFORM,
                memory: MemoryLocation::HostUpload,
            })?;
            rollback.buffers.push(buffer);
            tonemap_uniforms.push(buffer);
        }

        // --- the screen-space occlusion pair ---
        //
        // Stored in the rollback whole, like the light grid: it owns two
        // pipelines and a ring of buffers, and `Ssao::destroy` is the one place
        // their release order lives.
        rollback.ssao = Some(Ssao::new(
            device,
            frames,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the screen-space contact-shadow march ---
        //
        // Stored whole for the pair above's reason, and after them because
        // `Rollback::run` releases in the reverse order of construction.
        rollback.contact_shadows = Some(ContactShadows::new(
            device,
            frames,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the screen-space reflection march ---
        //
        // Stored whole for the pair above's reason, and after them because
        // `Rollback::run` releases in the reverse order of construction.
        rollback.ssr = Some(Ssr::new(
            device,
            queue,
            frames,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the Hi-Z pyramid the march climbs ---
        //
        // After the march it serves and before the chain below, on their reason:
        // `Rollback::run` releases in the reverse order of construction. Its
        // pipeline is the only one in this file with a depth attachment and no
        // colour one — see [`ForwardRenderer::build_depth_fullscreen`].
        rollback.hiz = Some(Hiz::new(
            device,
            frames,
            ForwardRenderer::build_depth_fullscreen,
        )?);

        // --- the froxel volume ---
        //
        // Stored whole for the three above's reason, and after them because
        // `Rollback::run` releases in the reverse order of construction. It sits
        // between the march and the chain in the frame as well: the medium is
        // scene content and the chain is a lens. Its volume holds the same
        // [`FROXEL_CAPACITY`] the clustering pass's grid does, because it is
        // subdivided by the same [`Grid`] — see [`crate::volumetric`].
        rollback.volumetric = Some(Volumetric::new(
            device,
            frames,
            FROXEL_CAPACITY,
            inputs.shadow_map,
            inputs.shadow_sampler,
            lights,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- auto-exposure ---
        //
        // Stored whole for the volume's reason, and after it because
        // `Rollback::run` releases in the reverse order of construction. It runs
        // between the chain and the tonemap in the frame as well: it bins the
        // picture the tonemap is about to read, which is the one with the lens
        // already on it — see [`crate::exposure`].
        rollback.exposure = Some(Exposure::new(device, queue, frames)?);

        // --- the bloom chain ---
        //
        // Stored whole for the pair above's reason, and after them because
        // `Rollback::run` releases in the reverse order of construction. It owns
        // a **linear** sampler of its own; the tonemap's sampler is `Nearest` on
        // purpose and `crate::bloom` says why the chain cannot share it.
        rollback.bloom = Some(Bloom::new(
            device,
            frames,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the antialiasing resolve ---
        //
        // Stored whole for the three above's reason, and after them because
        // `Rollback::run` releases in the reverse order of construction. It is
        // the one of the four that needs `target_format`: it writes the caller's
        // target where the others write `Rgba16Float` transients of their own
        // choosing — see [`crate::fxaa`]. Its sampler is **linear** for the
        // chain's reason and not the tonemap's.
        rollback.fxaa = Some(Fxaa::new(
            device,
            frames,
            inputs.target_format,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the higher antialiasing tier ---
        //
        // The same slot, filled by CMAA2's two dispatches and one draw
        // instead of FXAA's one draw — see [`crate::cmaa2`], which says why the
        // two are built together and at most one recorded. It carries no lookup
        // table, so unlike the tier it replaced it takes no `queue`: there is
        // nothing to upload.
        rollback.cmaa2 = Some(Cmaa2::new(
            device,
            frames,
            inputs.target_format,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the render-scale upscale ---
        //
        // The second pass that writes the caller's target rather than a
        // transient of its own. It draws on no frame at a render scale of
        // `1.0`, which is every frame until a caller moves it — see
        // [`crate::upscale`].
        rollback.upscale = Some(Upscale::new(
            device,
            frames,
            inputs.target_format,
            ForwardRenderer::build_fullscreen,
        )?);

        // --- the background ---
        //
        // Built last, so `Rollback::run` releases it first. It writes the scene
        // target rather than the caller's — it is scene content, drawn before
        // the operator, unlike the ground grid — and takes the depth format as
        // well, because it is the one full-screen pass in this frame that tests
        // against an attachment. See [`crate::sky_pass`].
        rollback.sky_pass = Some(SkyPass::new(
            device,
            frames,
            Format::Rgba16Float,
            Format::D32Float,
            ForwardRenderer::build_tested_fullscreen,
        )?);

        // Whole, so the rollback lets go of every handle: the view owns them
        // from here, and a rollback still naming one would release it twice.
        let view =
            Self {
                draws: rollback.draws.take().unwrap_or_else(|| {
                    unreachable!("draw generation was placed in the rollback above")
                }),
                cluster_selection,
                // Overwritten by the first `begin_frame`, which is the only thing
                // that can know the viewport. A zero scale with a budget of zero
                // selects nothing at all, and there is no frame yet to select for.
                lod_params: [0.0, 0.0, 0.0],
                lights: rollback.lights.take().unwrap_or_else(|| {
                    unreachable!("the light grid was placed in the rollback above")
                }),
                // Overwritten by the first `begin_frame` on `lod_params`' terms: a
                // one-froxel grid is the smallest legal one, and there is no
                // viewport yet to size a real one against.
                grid: Grid {
                    x: 1,
                    y: 1,
                    slices: 1,
                    tile_pixels: 1,
                },
                uniforms,
                mesh_groups,
                tonemap_uniforms,
                tonemap_groups: vec![None; frames],
                mesh_group_entries,
                prepass_group_entries,
                screen_channel_groups: vec![None; frames],
                prepass_groups,
                prepass_stats,
                // Replaced by every `begin_frame`, which `add_passes` documents as
                // having to run first.
                camera_view_proj: Mat4::IDENTITY,
                // No frame has been drawn, so there is no previous camera — see the
                // field, which says why that is not the identity.
                previous_view_projection: None,
                ssao: rollback.ssao.take().unwrap_or_else(|| {
                    unreachable!("the occlusion pair was placed in the rollback above")
                }),
                contact_shadows: rollback.contact_shadows.take().unwrap_or_else(|| {
                    unreachable!("the contact march was placed in the rollback above")
                }),
                hiz: rollback.hiz.take().unwrap_or_else(|| {
                    unreachable!("the pyramid was placed in the rollback above")
                }),
                ssr: rollback.ssr.take().unwrap_or_else(|| {
                    unreachable!("the reflection march was placed in the rollback above")
                }),
                volumetric: rollback.volumetric.take().unwrap_or_else(|| {
                    unreachable!("the froxel volume was placed in the rollback above")
                }),
                auto_exposure: rollback.exposure.take().unwrap_or_else(|| {
                    unreachable!("the histogram was placed in the rollback above")
                }),
                bloom: rollback.bloom.take().unwrap_or_else(|| {
                    unreachable!("the bloom chain was placed in the rollback above")
                }),
                fxaa: rollback.fxaa.take().unwrap_or_else(|| {
                    unreachable!("the resolve was placed in the rollback above")
                }),
                cmaa2: rollback.cmaa2.take().unwrap_or_else(|| {
                    unreachable!("the higher tier was placed in the rollback above")
                }),
                upscale: rollback.upscale.take().unwrap_or_else(|| {
                    unreachable!("the upscale was placed in the rollback above")
                }),
                sky_pass: rollback
                    .sky_pass
                    .take()
                    .unwrap_or_else(|| unreachable!("the sky was placed in the rollback above")),
            };
        rollback.buffers.clear();
        rollback.bind_groups.clear();
        Ok(view)
    }

    /// Releases every resource this view owns.
    ///
    /// The caller's to call once no frame in flight still names them — the
    /// same contract [`ForwardRenderer::destroy`] has, because these are the
    /// handles that frame's passes and groups bind.
    pub(super) fn destroy(self, device: &dyn Device) {
        for (_, group) in self.tonemap_groups.into_iter().flatten() {
            device.destroy_bind_group(group);
        }
        for buffer in self.tonemap_uniforms {
            device.destroy_buffer(buffer);
        }
        self.sky_pass.destroy(device);
        self.upscale.destroy(device);
        self.cmaa2.destroy(device);
        self.fxaa.destroy(device);
        self.bloom.destroy(device);
        self.auto_exposure.destroy(device);
        self.volumetric.destroy(device);
        self.ssr.destroy(device);
        self.hiz.destroy(device);
        self.contact_shadows.destroy(device);
        self.ssao.destroy(device);
        for group in self
            .mesh_groups
            .into_iter()
            .chain(self.prepass_groups)
            .chain(
                self.screen_channel_groups
                    .into_iter()
                    .flatten()
                    .map(|(_, group)| group),
            )
        {
            device.destroy_bind_group(group);
        }
        for buffer in self.prepass_stats {
            device.destroy_buffer(buffer);
        }
        for buffer in self.uniforms {
            device.destroy_buffer(buffer);
        }
        for buffer in self.cluster_selection {
            device.destroy_buffer(buffer);
        }
        self.lights.destroy(device);
        self.draws.destroy(device);
    }
}
