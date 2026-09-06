//! CMAA2 through the frame, measured against the same scene without it.
//!
//! `crcbl_render::cmaa2` records five passes into the resolve slot —
//! `cmaa2-clear`, `cmaa2-edges`, `cmaa2-shapes`, `cmaa2-accumulate` and
//! `cmaa2-apply` — where [`RenderEffects::ANTIALIASING`] records one. What a GPU
//! test can say about that is not what a golden would say (a blessed picture of
//! an antialiased cube is a picture of *something*, and stays green when the
//! filter degrades into a blur or into a copy). It is the shape of the
//! difference, and these are the retired SMAA tier's three measurements on the
//! same scene — `docs/plan/49-antialiasing.md` says in as many words that CMAA2
//! is held to its observer:
//!
//! * **The frame changed at all.** Five passes that ran and wrote their source
//!   through would leave the frame byte-identical to the no-AA one, which is
//!   the whole failure mode a blessed image cannot see.
//! * **It changed in a band along the silhouettes and nowhere else.** CMAA2
//!   classifies only pixels its edge pass marked, so a pixel with no luma
//!   discontinuity anywhere near it must come out of the apply untouched. A
//!   filter that lost its candidate list, read the wrong texel or accumulated a
//!   weight nothing derived touches the flat faces too — and the flat faces are
//!   most of this frame.
//! * **It changed by a little, not by a lot.** A reconstructed line is at most
//!   half a pixel from the aliased boundary it replaces, so a fully-weighted
//!   edge pixel moves by half the discontinuity. Bytes moving by tens across the
//!   band is a blur or a shifted read, not an antialias.
//! * **And the silhouettes came out softer than they went in**, counted rather
//!   than looked at: fewer pixels whose neighbour-to-neighbour luma step is
//!   still nearly the whole discontinuity. That is the one measurement that
//!   says the filter did the thing it is for, and the threshold it is counted
//!   at matters — see [`HARD_LUMA_STEP`], which is where the obvious version of
//!   this count moves the wrong way.
//!
//! Two more live here that the tier this replaced had no need of, because it
//! was three fullscreen draws and this one is a scatter into shared memory:
//! [`the_same_frame_resolves_to_the_same_bytes_twice`] and
//! [`a_frame_that_overflows_the_lists_still_finishes_and_stays_finite`]. Each
//! carries its own argument.
//!
//! # Both tiers are drawn, because they share one slot
//!
//! `crcbl_render::forward` records CMAA2 *instead of* FXAA, never both, so the
//! frame this compares against is not only "CMAA2 off" but "the cheap tier in
//! the same slot". Drawing all three — no resolve, FXAA, CMAA2 — is what
//! separates a wired-up CMAA2 from a request that fell through to the tier that
//! was already there: two identical submissions differ by exactly zero, and
//! that is the assertion.
//!
//! # The thresholds
//!
//! Swept on both local adapters before they were pinned; each constant carries
//! its own measurements. The two adapters are radv on the discrete card and
//! lavapipe, which rasterise this silhouette a texel apart — every bound here
//! is set off the worse of the two with room, on
//! `docs/plan/12-testing.md`'s terms for a measurement that has to survive a
//! different rasteriser.

use crate::harness::Headless;
use crate::mesh_scene::{mesh_camera, place_cube, render_mesh};
use crcbl::render::{
    EffectOverride, EffectRequest, ForwardRenderer, Projection, RenderEffects, TransientPool,
};
use crcbl_golden::Image;

/// The neighbour-to-neighbour luma step, in `u8` units, that counts as an edge.
///
/// Well above the couple of units of shading gradient across a lit face and
/// well under the tens a silhouette carries, so the mask below is the
/// silhouette and not the shading.
const EDGE_LUMA_STEP: f64 = 12.0;

/// How far from an edge pixel the resolve is allowed to reach, in pixels.
///
/// CMAA2's blend items move a pixel toward one *immediate* neighbour — the one
/// on the other side of the boundary the run was found along — so a changed
/// pixel is either on an edge or beside one. The extra pixel of slack is for
/// the mask itself: the shader detects its edges on its own luma estimate at
/// its own threshold, which need not agree pixel-for-pixel with the one this
/// file computes on the read-back frame.
const BAND_RADIUS: u32 = 2;

/// The luma step that counts as a *hard* one, in the same `u8` units.
///
/// [`EDGE_LUMA_STEP`] finds the silhouette; this measures how sharply it still
/// steps. **Counting the pixels over the lower threshold cannot say a filter
/// antialiased anything** — it goes *up*, because a hard step spread over three
/// pixels is three steps that each still clear 12. Measured on this scene: 616
/// pixels over 12 with no resolve. What falls is the count of steps that are
/// still nearly the whole discontinuity, which is what the gradient histogram
/// shows a resolve doing.
const HARD_LUMA_STEP: f64 = 96.0;

/// How much of the frame CMAA2 must move before the passes count as having run.
///
/// Swept on radv and lavapipe at 2026-09-06: 0.0162 of the frame on both. A
/// quarter of that, so the floor is a floor rather than the measurement — and a
/// resolve that wrote its source through would measure exactly zero.
const MIN_CHANGED_FRACTION: f64 = 0.004;

/// How much of the frame CMAA2 may move before it has stopped being a band.
///
/// Four times the same measurement, which still leaves it an order of magnitude
/// under a filter that touched the flat faces: this scene's silhouette band is
/// 3215 pixels of 49152, and everything outside it is flat shading.
const MAX_CHANGED_FRACTION: f64 = 0.065;

/// How far the band's pixels may move, per channel on average.
///
/// The same sweep: 25.919 on radv and 25.813 on lavapipe, against a
/// discontinuity of well over a hundred — a reconstructed boundary is at most
/// half a pixel from the aliased one, so a fully-weighted edge pixel moves by
/// half the step and the average over the band is far less. A little over
/// 1.7 times the worse measurement.
const MAX_MEAN_BAND_DELTA: f64 = 44.0;

/// What fraction of the unfiltered frame's hard steps may survive the resolve.
///
/// The sweep at [`HARD_LUMA_STEP`]: 332 pixels with no resolve, against 145 on
/// both adapters — well under half of them, and better than FXAA in the same
/// slot (150 on radv, 154 on lavapipe). Pinned at a half again the worse
/// measurement, so a resolve has to remove a third of the hard steps to pass and
/// the assertion is nowhere near the measurement in either direction.
const MAX_HARD_STEP_RATIO: f64 = 0.65;

/// Rec. 709 luma of an RGBA8 pixel, in the same `u8` units.
fn luma(pixel: [u8; 4]) -> f64 {
    0.2126 * f64::from(pixel[0]) + 0.7152 * f64::from(pixel[1]) + 0.0722 * f64::from(pixel[2])
}

/// The larger of the two forward neighbour luma steps at `(x, y)`.
///
/// Zero on the last row and column, which have no forward neighbour to step to.
fn gradient(image: &Image, x: u32, y: u32) -> f64 {
    let here = luma(image.pixel(x, y).expect("inside the image"));
    let right = image.pixel(x + 1, y).map_or(here, luma);
    let down = image.pixel(x, y + 1).map_or(here, luma);
    (here - right).abs().max((here - down).abs())
}

/// Every pixel whose luma steps by at least `step` into a neighbour.
fn step_mask(image: &Image, step: f64) -> Vec<bool> {
    let mut mask = vec![false; (image.width() * image.height()) as usize];
    for y in 0..image.height() {
        for x in 0..image.width() {
            mask[(y * image.width() + x) as usize] = gradient(image, x, y) >= step;
        }
    }
    mask
}

/// How many pixels [`step_mask`] marks.
fn steps_over(image: &Image, step: f64) -> usize {
    step_mask(image, step).iter().filter(|on| **on).count()
}

/// The mask grown by [`BAND_RADIUS`] in every direction.
fn dilate(mask: &[bool], width: u32, height: u32) -> Vec<bool> {
    let mut grown = vec![false; mask.len()];
    for y in 0..height {
        for x in 0..width {
            if !mask[(y * width + x) as usize] {
                continue;
            }
            let (x0, y0) = (x.saturating_sub(BAND_RADIUS), y.saturating_sub(BAND_RADIUS));
            let x1 = (x + BAND_RADIUS).min(width - 1);
            let y1 = (y + BAND_RADIUS).min(height - 1);
            for gy in y0..=y1 {
                for gx in x0..=x1 {
                    grown[(gy * width + gx) as usize] = true;
                }
            }
        }
    }
    grown
}

/// What one frame's difference from the no-AA frame looks like.
struct Difference {
    /// Pixels differing in any channel.
    changed: usize,
    /// Of those, the ones with no edge within [`BAND_RADIUS`].
    changed_off_band: usize,
    /// Mean absolute per-channel move over the changed pixels.
    mean_delta: f64,
    /// The largest single-channel move anywhere.
    max_delta: u8,
}

/// Compares `frame` against `base` under `base`'s own dilated edge mask.
fn difference(base: &Image, frame: &Image, band: &[bool]) -> Difference {
    let (width, height) = (base.width(), base.height());
    let mut changed = 0;
    let mut changed_off_band = 0;
    let mut total_delta = 0u64;
    let mut max_delta = 0u8;
    for y in 0..height {
        for x in 0..width {
            let one = base.pixel(x, y).expect("inside the image");
            let other = frame.pixel(x, y).expect("the same extent");
            // Alpha is the swapchain's own and never a claim about the filter.
            let delta: u32 = (0..3).map(|c| u32::from(one[c].abs_diff(other[c]))).sum();
            if delta == 0 {
                continue;
            }
            changed += 1;
            total_delta += u64::from(delta);
            max_delta = max_delta.max((0..3).map(|c| one[c].abs_diff(other[c])).max().unwrap_or(0));
            if !band[(y * width + x) as usize] {
                changed_off_band += 1;
            }
        }
    }
    Difference {
        changed,
        changed_off_band,
        mean_delta: if changed == 0 {
            0.0
        } else {
            total_delta as f64 / (changed as f64 * 3.0)
        },
        max_delta,
    }
}

/// The demo cube drawn with the resolve slot set as the caller asks, and
/// CMAA2's two append lists capped at `cap` entries where one is given.
fn cube_frame(effects: EffectOverride, cap: Option<u32>) -> Image {
    let headless = Headless::open_for_mesh();
    let mut pool = TransientPool::new();
    let mut renderer =
        ForwardRenderer::new(headless.device.as_ref(), headless.queue, headless.format)
            .expect("the forward renderer builds");
    renderer.set_effect_request(EffectRequest {
        programmatic: effects,
        ..EffectRequest::default()
    });
    renderer.set_cmaa2_capacity_cap(cap);
    place_cube(&mut renderer);
    let camera = mesh_camera(Projection::default());
    render_mesh(&headless, &mut renderer, &mut pool, &camera, None)
}

/// The override that puts the named tier in the resolve slot, and nothing else
/// in it.
fn tier(fxaa: bool, cmaa2: bool) -> EffectOverride {
    EffectOverride::none()
        .force(RenderEffects::ANTIALIASING, Some(fxaa))
        .force(RenderEffects::CMAA2, Some(cmaa2))
}

/// **CMAA2 softens the silhouettes and leaves the rest of the frame alone.**
#[test]
#[ignore = "needs a real GPU; run crates/crcbl/tests/run-mesh-e2e.sh"]
fn cmaa2_changes_a_band_along_the_edges_and_nothing_else() {
    let none = cube_frame(tier(false, false), None);
    let fxaa = cube_frame(tier(true, false), None);
    let cmaa2 = cube_frame(tier(false, true), None);

    let (width, height) = (none.width(), none.height());
    let total = (width * height) as f64;
    let mask = step_mask(&none, EDGE_LUMA_STEP);
    let band = dilate(&mask, width, height);
    let edges_none = mask.iter().filter(|on| **on).count();
    let band_pixels = band.iter().filter(|on| **on).count();
    let hard_none = steps_over(&none, HARD_LUMA_STEP);
    let hard_fxaa = steps_over(&fxaa, HARD_LUMA_STEP);
    let hard_cmaa2 = steps_over(&cmaa2, HARD_LUMA_STEP);

    let against_cmaa2 = difference(&none, &cmaa2, &band);
    let against_fxaa = difference(&none, &fxaa, &band);
    let cmaa2_vs_fxaa = difference(&fxaa, &cmaa2, &band);

    eprintln!(
        "crcbl mesh e2e: cmaa2 — {width}x{height}; edge pixels {edges_none}, band \
         {band_pixels}; hard steps {hard_none} none, {hard_fxaa} fxaa, \
         {hard_cmaa2} cmaa2; cmaa2 changed {} ({:.4} of frame, {} off band), mean \
         {:.3}, max {}; fxaa changed {} ({} off band), mean {:.3}, max {}; cmaa2 \
         vs fxaa changed {}, mean {:.3}, max {}",
        against_cmaa2.changed,
        against_cmaa2.changed as f64 / total,
        against_cmaa2.changed_off_band,
        against_cmaa2.mean_delta,
        against_cmaa2.max_delta,
        against_fxaa.changed,
        against_fxaa.changed_off_band,
        against_fxaa.mean_delta,
        against_fxaa.max_delta,
        cmaa2_vs_fxaa.changed,
        cmaa2_vs_fxaa.mean_delta,
        cmaa2_vs_fxaa.max_delta,
    );

    // **The mask is a mask**, before anything measured under it means
    // something: a threshold that matched the whole frame would make the band
    // test vacuous, and one that matched nothing would make it unfalsifiable.
    assert!(
        edges_none > 0 && band_pixels < (total * 0.5) as usize,
        "the edge mask has to be the silhouette: {edges_none} edge pixels, \
         {band_pixels} in the band of {total}"
    );

    // **Five passes ran and something came out of them.**
    assert!(
        against_cmaa2.changed as f64 / total >= MIN_CHANGED_FRACTION,
        "cmaa2 moved {} of {total} pixels, which is a resolve that wrote its \
         source through",
        against_cmaa2.changed
    );
    // **And it is not the cheap tier under another name.**
    assert!(
        cmaa2_vs_fxaa.changed > 0,
        "the cmaa2 frame is byte-identical to the fxaa one, so the request fell \
         through to the tier that was already in the slot"
    );

    // **The band is a band.**
    assert!(
        against_cmaa2.changed as f64 / total <= MAX_CHANGED_FRACTION,
        "cmaa2 moved {} of {total} pixels, which is the whole frame rather than \
         its edges",
        against_cmaa2.changed
    );
    assert_eq!(
        against_cmaa2.changed_off_band, 0,
        "cmaa2 moved {} pixels with no luma discontinuity within {BAND_RADIUS}, \
         so the apply is not reading its candidate list",
        against_cmaa2.changed_off_band
    );

    // **By a little.**
    assert!(
        against_cmaa2.mean_delta <= MAX_MEAN_BAND_DELTA,
        "cmaa2 moved the band by {:.3} per channel on average, which is a blur \
         rather than a blend",
        against_cmaa2.mean_delta
    );

    // **And the silhouettes are softer than they were.**
    assert!(
        (hard_cmaa2 as f64) <= hard_none as f64 * MAX_HARD_STEP_RATIO,
        "cmaa2 left {hard_cmaa2} pixels stepping by {HARD_LUMA_STEP} where the \
         unfiltered frame has {hard_none}, so the resolve ran without \
         antialiasing anything"
    );
}

/// **The same frame resolves to the same bytes twice.**
///
/// This is the assertion the tier's whole apply design exists for. CMAA2's
/// blend items are *scattered*: a pixel's colour is a sum over items produced
/// by different work-groups, which reach it in whatever order the device
/// schedules. A float sum in that order would make the frame a function of the
/// scheduler — `docs/plan/49-antialiasing.md`'s determinism argument, and the
/// reason a golden could not be blessed on it — so
/// `cmaa2_shapes.slang`'s `accumulateMain` sums in fixed point with integer
/// atomics, which are associative and commutative, and
/// `cmaa2_apply.slang` converts once, per pixel, after every item has landed.
///
/// **Two whole runs, not two frames of one run.** Each `cube_frame` opens its
/// own device, records its own graph and submits its own frame, so what this
/// compares is two independent schedules of the same work rather than one warm
/// cache read twice.
///
/// It runs on radv and on lavapipe, which is the point: two rasterisers that
/// distribute work-groups differently is where an order dependence shows up as
/// two different pictures rather than as one that happens to be stable.
#[test]
#[ignore = "needs a real GPU; run crates/crcbl/tests/run-mesh-e2e.sh"]
fn the_same_frame_resolves_to_the_same_bytes_twice() {
    let first = cube_frame(tier(false, true), None);
    let second = cube_frame(tier(false, true), None);

    let differing = (0..first.height())
        .flat_map(|y| (0..first.width()).map(move |x| (x, y)))
        .filter(|&(x, y)| first.pixel(x, y) != second.pixel(x, y))
        .count();
    eprintln!(
        "crcbl mesh e2e: cmaa2 determinism — {}x{}, {differing} pixels differ \
         between two runs of the same frame",
        first.width(),
        first.height(),
    );

    // Not vacuous: the frame under test is one the tier actually resolved, and
    // the observer above is what says so — this pair would also be identical if
    // the resolve had written its source through, which is why both tests
    // exist and neither replaces the other.
    assert_eq!(
        differing, 0,
        "two runs of one frame differ in {differing} pixels, so the apply's \
         accumulation depends on the order its items arrived in"
    );
}

/// How many entries the overflow test leaves in each of CMAA2's lists.
///
/// Far below what this scene needs — the observer above prints the frame's edge
/// pixel count and it is in the thousands — so the edge pass fills the candidate
/// list within its first work-group and every classification after that is
/// dropped. A cap rather than a smaller frame because the capacity is a
/// *fraction* of the frame: there is no extent at which a cube's silhouette
/// overflows the list it was sized for.
const OVERFLOW_CAP: u32 = 8;

/// **A frame that overflows both lists still finishes, and every pixel of it is
/// a colour.**
///
/// The degradation this tier's capacities are chosen for: an entry past the cap
/// is dropped, `cmaa2_shapes.slang` and its `accumulateMain` clamp every count
/// they read back to the same capacity, and the pixel a dropped item was for
/// keeps the colour it came in with. What must **not** happen is a write outside
/// the list — which on a real device is another buffer's memory, and on a
/// validation layer is an error the run never returns from.
///
/// Four things are asserted and the last two are the ones with teeth: the run
/// completed, every pixel is a real colour rather than the uninitialised
/// contents of an aliased transient, the frame is **not** the uncapped one — so
/// the cap actually reached the shader and the drop path actually ran — and the
/// cap left far less of the resolve than the uncapped frame carries, which is
/// what says entries were dropped rather than merely reordered. Measured on
/// radv and lavapipe at 2026-09-06: the capped frame differs from the
/// unresolved one in 8 pixels where the uncapped frame differs in 788.
///
/// **What it cannot see is the write itself.** A device with robust buffer
/// access discards a store past a bound descriptor's range, so removing the
/// `slot < capacity` guard in `cmaa2_edges.slang` leaves this readback
/// bit-identical; what catches it is Vulkan's **GPU-assisted** validation,
/// which reports `VUID-vkCmdDispatch-storageBuffers-06936` against the
/// candidate list and which the harness's validation gate then fails on.
/// `docs/backlog.md` carries that the suite does not enable it by default.
#[test]
#[ignore = "needs a real GPU; run crates/crcbl/tests/run-mesh-e2e.sh"]
fn a_frame_that_overflows_the_lists_still_finishes_and_stays_finite() {
    let none = cube_frame(tier(false, false), None);
    let capped = cube_frame(tier(false, true), Some(OVERFLOW_CAP));
    let uncapped = cube_frame(tier(false, true), None);

    let (width, height) = (capped.width(), capped.height());
    let changed = |a: &Image, b: &Image| {
        (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter(|&(x, y)| a.pixel(x, y) != b.pixel(x, y))
            .count()
    };
    let against_none = changed(&none, &capped);
    let against_uncapped = changed(&uncapped, &capped);
    eprintln!(
        "crcbl mesh e2e: cmaa2 overflow — cap {OVERFLOW_CAP}: {against_none} pixels \
         differ from the unresolved frame, {against_uncapped} from the uncapped one"
    );

    // The run reached here, which is the first claim: an out-of-bounds write
    // past either list is a device loss or a validation error, and neither
    // returns an image.
    assert_eq!(
        (capped.width(), capped.height()),
        (none.width(), none.height()),
        "the capped frame came back at a different extent"
    );
    // Every pixel is a colour. A transient buffer is aliased against other
    // frames' memory, so a pixel the apply resolved out of an accumulation
    // nothing zeroed would come back as whatever was last in that allocation —
    // which the alpha channel is what catches, because every path out of
    // `cmaa2_apply.slang` writes it opaque.
    for y in 0..height {
        for x in 0..width {
            let pixel = capped.pixel(x, y).expect("inside the image");
            assert_eq!(
                pixel[3], 0xFF,
                "({x}, {y}) came back as {pixel:?}, which is not a colour this \
                 pass writes"
            );
        }
    }
    // And the overflow degraded the filter rather than removing it.
    assert!(
        against_uncapped > 0,
        "the capped frame is byte-identical to the uncapped one, so the cap \
         reached nothing and this test is measuring the frame it was not \
         written for"
    );
    // **Most of the resolve was dropped**, which is what says the lists filled
    // rather than merely being smaller than they might have been. A quarter is
    // a bound with room in it either way: the measurement is two orders of
    // magnitude apart, and a cap that dropped nothing would land at parity.
    assert!(
        against_none * 4 < against_uncapped,
        "the capped frame moved {against_none} pixels off the unresolved frame \
         against the uncapped frame's {against_uncapped}, which is not a list \
         that filled"
    );
}
