//! `docs/plan/29-fp-rendering.md`'s second camera, on whichever backend
//! `CRCBL_GPU` names: a view of the scene the renderer already holds, drawn into
//! a target of its own in the same frame as the primary camera.
//!
//! Pixels are the observable. A view that recorded every pass and drew into the
//! wrong image, or drew nothing, or drew the primary camera's picture, compiles
//! and submits cleanly — and only a readback of both targets can tell those
//! apart from a view that works.

use crcbl::hal::{
    BufferDesc, BufferImageCopy, BufferUsage, CommandEncoderDesc, Extent3d, Features, ImageAspect,
    ImageDesc, ImageSubresourceLayers, ImageSubresourceRange, ImageType, ImageUsage, ImageViewDesc,
    ImageViewType, MemoryLocation, PresentInfo, ResourceState, SubmitInfo,
};
use crcbl::math::Vec3;
use crcbl::render::{
    Camera, DirectionalLight, ForwardRenderer, FrameTargets, ImportedImage, InitialClaim,
    InstanceHandle, RenderGraph, TransientPool, ViewDesc, ViewId, ViewMask, ViewTarget,
};

use crate::harness::{Headless, poisoned};
use crate::mesh_scene::{MESH_EXTENT, mesh_camera, place};

/// The view's target: square, and a width whose four-byte rows meet the 256-byte
/// copy pitch wgpu and D3D12 enforce.
const VIEW_EXTENT: (u32, u32) = (128, 128);

/// Reads `image`, `extent` in size, back out of a finished frame's encoder.
fn copy_back(
    headless: &Headless,
    encoder: &mut dyn crcbl::hal::CommandEncoder,
    image: crcbl::hal::ImageHandle,
    extent: (u32, u32),
) -> crcbl::hal::BufferHandle {
    let staging = headless
        .device
        .create_buffer(&BufferDesc {
            label: Some("view readback"),
            size: u64::from(extent.0) * u64::from(extent.1) * 4,
            usage: BufferUsage::TRANSFER_DST,
            memory: MemoryLocation::HostReadback,
        })
        .expect("a readback buffer");
    encoder.copy_image_to_buffer(&BufferImageCopy {
        buffer: staging,
        buffer_offset: 0,
        buffer_row_length: 0,
        buffer_image_height: 0,
        image,
        image_subresource: ImageSubresourceLayers {
            aspect: ImageAspect::COLOR,
            mip: 0,
            base_layer: 0,
            layer_count: 1,
        },
        image_offset: crcbl::hal::Offset3d::default(),
        image_extent: Extent3d::d2(extent.0, extent.1),
    });
    staging
}

/// One frame of the primary camera into the swapchain and of `view` into
/// `view_image`, both read back.
fn render_both(
    headless: &Headless,
    renderer: &mut ForwardRenderer,
    pool: &mut TransientPool,
    view: ViewId,
    view_image: (crcbl::hal::ImageHandle, crcbl::hal::ImageViewHandle),
) -> (crcbl_golden::Image, crcbl_golden::Image) {
    let device = headless.device.as_ref();
    let acquired = device
        .acquire_next_frame(headless.swapchain)
        .expect("the ring always has an image");
    let camera = mesh_camera(crcbl::render::Projection::default());
    // The view looks at the same cube from the other side, so its picture is a
    // camera of its own and not a copy of the primary camera's.
    let view_camera = Camera {
        eye: Vec3::new(-1.6, 1.2, -2.2),
        ..camera
    };
    renderer
        .begin_frame(
            device,
            &camera,
            &DirectionalLight::default(),
            acquired.extent,
        )
        .expect("the uniform buffers are writable");
    renderer
        .begin_view(device, view, &view_camera, VIEW_EXTENT)
        .expect("the view's uniform buffers are writable");

    let mut encoder = device.create_command_encoder(&CommandEncoderDesc {
        label: Some("view frame"),
        queue: headless.queue,
    });
    let compiled = {
        let mut graph = RenderGraph::new(headless.queue);
        let target = graph.import_image(
            "swapchain",
            ImportedImage {
                image: acquired.image,
                view: acquired.view,
                format: headless.format,
                extent: acquired.extent,
                initial: ResourceState::Undefined,
                claim: InitialClaim::Acquired,
                final_state: ResourceState::TransferSrc,
            },
        );
        let view_target = graph.import_image(
            "view target",
            ImportedImage {
                image: view_image.0,
                view: view_image.1,
                format: headless.format,
                extent: VIEW_EXTENT,
                // Where the previous frame left it, which the pool records —
                // the first frame finds nothing recorded and says `Undefined`.
                initial: pool
                    .imported_image_use(view_image.0)
                    .unwrap_or(ResourceState::Undefined),
                claim: InitialClaim::Tracked,
                final_state: ResourceState::TransferSrc,
            },
        );
        renderer.add_passes_with_views(
            &mut graph,
            &*pool,
            FrameTargets {
                target,
                extent: acquired.extent,
                skinning: None,
                views: &[ViewTarget {
                    view,
                    target: view_target,
                    extent: VIEW_EXTENT,
                }],
            },
            |_, _| {},
        );
        graph.compile(&*pool).expect("a legal frame")
    };
    compiled
        .execute(device, pool, encoder.as_mut(), None)
        .expect("the graph executed");
    let primary_staging = copy_back(headless, encoder.as_mut(), acquired.image, acquired.extent);
    let view_staging = copy_back(headless, encoder.as_mut(), view_image.0, VIEW_EXTENT);
    let commands = encoder.finish().expect("recording succeeded");
    device
        .submit(headless.queue, &SubmitInfo::new(&[commands]))
        .expect("submit");
    device
        .present(
            headless.queue,
            &PresentInfo {
                swapchain: headless.swapchain,
                waits: acquired.present_semaphore.as_slice(),
                present_id: None,
            },
        )
        .expect("present");

    let order = match headless.format {
        crcbl::hal::Format::Bgra8Unorm | crcbl::hal::Format::Bgra8UnormSrgb => {
            crcbl_golden::ChannelOrder::Bgra
        }
        _ => crcbl_golden::ChannelOrder::Rgba,
    };
    let read = |staging, extent: (u32, u32)| {
        let bytes = u64::from(extent.0) * u64::from(extent.1) * 4;
        let mut pixels = poisoned(bytes as usize);
        headless.readback(staging, bytes, &mut pixels);
        device.destroy_buffer(staging);
        crcbl_golden::Image::from_readback(extent.0, extent.1, &pixels, order)
            .expect("the readback is exactly one image")
    };
    let primary = read(primary_staging, acquired.extent);
    let view_picture = read(view_staging, VIEW_EXTENT);
    device.destroy_command_buffer(commands);
    (primary, view_picture)
}

/// **A view draws the scene into its own target, and an instance hidden from it
/// is gone from its picture and nobody else's.**
///
/// The cube at the origin fills the centre of both targets. Hiding it from the
/// view has to turn the view's centre into the background its corner shows,
/// while the primary camera's centre keeps the cube — which is what separates a
/// working mask from a view that stopped drawing, and both from a cull that
/// dropped the cube everywhere.
#[test]
#[ignore = "needs a real GPU and a backend pin; run tests/run-forward-e2e.sh"]
fn a_view_draws_its_own_picture_and_skips_what_is_hidden_from_it() {
    let headless = Headless::open_for_mesh_with(Features::GPU_DRIVEN);
    let device = headless.device.as_ref();
    let mut pool = TransientPool::new();
    let mut renderer =
        ForwardRenderer::new(device, headless.queue, headless.format).expect("a forward renderer");
    let cube: InstanceHandle = place(
        &mut renderer,
        crcbl::render::scene::DEMO_CUBE,
        crcbl::render::scene::DEMO_UNTINTED,
        crcbl::math::Mat4::IDENTITY,
    );
    let view = renderer
        .create_view(device, headless.queue, &ViewDesc::default())
        .expect("a view");
    let image = device
        .create_image(&ImageDesc {
            label: Some("view target"),
            image_type: ImageType::D2,
            extent: Extent3d::d2(VIEW_EXTENT.0, VIEW_EXTENT.1),
            format: headless.format,
            mip_levels: 1,
            samples: 1,
            usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSFER_SRC,
        })
        .expect("a view target");
    let image_view = device
        .create_image_view(&ImageViewDesc {
            label: Some("view target"),
            image,
            view_type: ImageViewType::D2,
            format: headless.format,
            range: ImageSubresourceRange::all(headless.format),
        })
        .expect("a view target view");

    let (primary, shown) = render_both(
        &headless,
        &mut renderer,
        &mut pool,
        view,
        (image, image_view),
    );
    let centre = |picture: &crcbl_golden::Image, extent: (u32, u32)| {
        picture
            .pixel(extent.0 / 2, extent.1 / 2)
            .expect("inside the frame")
    };
    let corner = |picture: &crcbl_golden::Image| picture.pixel(1, 1).expect("inside the frame");
    eprintln!(
        "crcbl forward e2e: views — primary centre {:?}, view centre {:?}, view corner {:?}",
        centre(&primary, MESH_EXTENT),
        centre(&shown, VIEW_EXTENT),
        corner(&shown)
    );
    assert_ne!(
        centre(&shown, VIEW_EXTENT),
        corner(&shown),
        "the view's centre has to be the cube, or the comparison below is between two empty \
         pictures"
    );

    renderer.set_instance_views(cube, ViewMask::ALL.without(view));
    let (primary_after, hidden) = render_both(
        &headless,
        &mut renderer,
        &mut pool,
        view,
        (image, image_view),
    );
    assert_eq!(
        centre(&hidden, VIEW_EXTENT),
        corner(&hidden),
        "the cube is hidden from the view, so its centre is the background"
    );
    assert_ne!(
        centre(&primary_after, MESH_EXTENT),
        corner(&primary_after),
        "and the primary camera still draws the cube it was never hidden from"
    );
    assert_eq!(
        centre(&primary_after, MESH_EXTENT),
        centre(&primary, MESH_EXTENT),
        "whose picture the view's mask does not touch"
    );

    device.wait_idle().expect("idle");
    renderer.destroy(device);
    device.destroy_image_view(image_view);
    device.destroy_image(image);
    pool.destroy(device);
    headless.finish();
}
