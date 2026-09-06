//! The panels hud actually presented, off a real device, against a checked-in
//! golden — and four claims about the picture in front of it.
//!
//! # This is the flag's test as much as the frame's
//!
//! `apps/breakout/tests/golden.rs` is the pattern and its module docs carry the
//! full argument; the short version is that this suite runs the **compiled
//! binary** with `--screenshot` and everything it asserts is about the file that
//! binary left behind. The thing under test is the frame a player would have
//! seen, not a draw list a test rebuilt to look like it.
//!
//! It is also why there is no second render here and no in-process device: the
//! suite owns no GPU at all, and a failure in it is a failure of the sample.
//!
//! # The one sample whose frame is a clear plus `ui-composite`
//!
//! hud is exempt from sample rule 11 — no `.crpix` art, no sprite pass — so its
//! frame is a `backdrop` pass and the UI compositor and nothing else. That makes
//! it the only golden in the tree that would catch a UI-pass regression with no
//! sprite pass in front of it to have tripped first, and it is why the claims
//! below are about the widgets themselves: the health bar's fill, the mana
//! bar's, and the empty page they sit on.
//!
//! # A golden alone cannot say the frame is right
//!
//! Two blank frames compare perfectly, and so do two uniformly dark ones against
//! a uniformly dark reference. So the golden is the *last* assertion and the
//! ones before it are ratios between blocks of pixels, which say **where** the
//! frame is bright, dark and coloured rather than what any one pixel is.
//!
//! The two bar fills are the pair that matters most: health is red over blue and
//! mana is blue over red, so a readback whose channels were written the wrong
//! way round fails both — and the structural half of the golden comparison,
//! computed on luma, barely moves for a swap.
//!
//! # The invocation is `crcbl-sample-test`'s
//!
//! [`SampleRun`] runs the binary, checks its summary and hands back the frame,
//! because four other samples' suites want the same thing — see that crate.
//! What stays here is what is about *this frame*: the claims below, and the
//! constants they are measured against.
//!
//! # Feature-gated *and* ignored
//!
//! The pair `crcbl`'s `render-e2e` and breakout's `golden-e2e` use. A plain
//! `cargo test --workspace --all-features` on a machine with no GPU must stay
//! green, and `tests/run-hud-golden.sh` is the only thing that turns both off —
//! and it fails when the suite reports zero tests run.
//!
//! # No darkening test here
//!
//! `apps/breakout/tests/golden.rs` carries
//! `a_uniformly_darkened_frame_is_refused_by_the_tolerance_the_golden_uses`,
//! which pins that a uniform multiply is refused. That is a property of
//! `crcbl_golden::Tolerance::RASTERISER`, which this suite compares under
//! unchanged, so a copy here would be a second thing to keep in step and would
//! prove nothing about hud.

#![cfg(feature = "golden-e2e")]

use std::path::PathBuf;

use crcbl_golden::{Golden, Image};
use crcbl_sample_test::{Block, SampleRun, required_backend};

/// How many frames the run presents before the one that gets written.
///
/// The budget `.github/workflows/ci.yml`'s **Run hud headless against lavapipe**
/// step already gives this binary, so the golden is a picture of a run that
/// workflow was making anyway rather than a second frame index to keep track of.
/// One second of the default 60 Hz simulation: far past start-up — the atlas has
/// uploaded, the first tick has run and the page has been laid out — and far
/// past the offscreen ring's frames-in-flight, so the image written has been
/// round the ring several times. The ticker is seeded, so the floating damage
/// numbers and the ability cooldowns are in the same place every run.
const FRAMES: u32 = 60;

/// The extent the checked-in golden is blessed at.
///
/// `crcbl::engine::DEFAULT_WINDOW_SIZE` at scale 1, which is what a headless
/// run's offscreen ring renders at when `--size` says nothing — so the golden is
/// the frame the default invocation produces rather than one a flag had to ask
/// for.
const EXTENT: (u32, u32) = (960, 720);

/// How many distinct colours a frame of this page has to have.
///
/// **Far lower than any other sample's, and that is what hud is.** There is no
/// sprite art here at all: a flat backdrop, two bars over their tracks, a stat
/// panel, the wave banner, four ability tiles and one text colour. Every one of
/// those is a solid fill, so the whole page is a page of a dozen-odd colours by
/// construction — a frame with fewer than this drew the clear and very little
/// else. Counted rather than guessed at — radv draws 17 and
/// `Image::distinct_colors` stops counting at the bound it is given.
const MIN_COLORS: usize = 12;

/// Half-extents, in pixels, of the block each claim below averages over.
///
/// **Smaller than breakout's**, because hud's subjects are bars a dozen pixels
/// tall and a mana fill a dozen wide at wave one: a six-pixel half straddles the
/// fill's right edge and drags the track's colour into the mean. It is still a
/// block rather than a pixel, because a single pixel is a sample of the
/// rasteriser as much as of the picture.
///
/// A smaller block is not a looser comparison — the golden below is compared
/// under the same `crcbl_golden::Tolerance::RASTERISER` every other sample uses,
/// which is why no darkening test is repeated here.
const BLOCK: (u32, u32) = (4, 4);

/// Inside the health bar's fill, left of the `200 / 200` label.
const HEALTH_AT: (u32, u32) = (120, 60);

/// Inside the mana bar's fill, which at wave one is the leftmost sliver of its
/// track.
const MANA_AT: (u32, u32) = (42, 106);

/// The empty middle of the page, which no widget reaches.
const BACKDROP_AT: (u32, u32) = (600, 300);

/// How much brighter the health bar's fill must be than the page behind it.
///
/// A ratio rather than a level, because a level is a second golden written in
/// numbers and moves whenever the art does. Measured before it was fixed rather
/// than guessed: radv draws the fill at 170.7/255 over a backdrop of 72.3, which
/// is 2.4. Each claim prints what it actually got, so the next person sizing it
/// does not have to re-derive it.
const FILL_OVER_BACKDROP: f32 = 1.8;

/// How much redder than its other channels the health bar's fill must be.
///
/// Half of the claim a channel-order mistake fails: a BGRA readback written as
/// RGBA turns this bar cyan and leaves every brightness ratio here happy. radv
/// draws it at red 234 / green 134 / blue 144, so 1.7 against the nearer of the
/// two.
const HEALTH_REDNESS: f32 = 1.3;

/// How much bluer than red the mana bar's fill must be.
///
/// The other half, and pointing the opposite way, so the pair cannot both be
/// satisfied by a frame whose channels were rotated rather than swapped. radv
/// draws it at blue 246 / red 139, which is 1.8.
const MANA_BLUENESS: f32 = 1.3;

/// The floor a block has to clear to have drawn anything at all.
///
/// Out of 255, and low because that is the question it asks: not "is this the
/// right shade" — the golden answers that — but "did a pass put anything here".
/// The darkest block any claim below reads is the backdrop's 72.3, so this
/// leaves an order of magnitude.
const DREW_AT_ALL: f32 = 6.0;

/// The claims in front of the golden: it drew, and it drew in the right places.
fn inspect(image: &Image) {
    let block = Block::new(image, BLOCK);
    let colors = image.distinct_colors(MIN_COLORS);
    assert!(
        colors >= MIN_COLORS,
        "a page with {colors} distinct colour(s) (counted to {MIN_COLORS}) is not \
         evidence — nothing drew, or only the clear did"
    );

    // ---- 1. the bars are widgets on top of a page --------------------------
    let health = block.brightness(HEALTH_AT);
    let backdrop = block.brightness(BACKDROP_AT);
    eprintln!("hud golden: health fill {health:.1}/255, backdrop {backdrop:.1}/255");
    // No separate "the fill drew at all" floor: the page behind it is already at
    // 72/255, so any floor low enough to be a drew-at-all check is one the page
    // itself would clear. The ratio is the claim, and a UI pass that never ran
    // leaves both points reading the same backdrop and fails it at 1.0.
    assert!(
        health > backdrop * FILL_OVER_BACKDROP,
        "the health bar's fill is {health:.1} and the page behind it is {backdrop:.1} — the \
         bar is not on top of the page, or the whole frame has been flattened"
    );

    // ---- 2. the page is dark rather than absent ----------------------------
    //
    // The other half of claim 1, and the one that stops it being satisfied by a
    // frame that lost its backdrop: `health > backdrop * ratio` holds for
    // `backdrop == 0`, which is what a pass that never ran looks like.
    assert!(
        backdrop > DREW_AT_ALL,
        "the page is at {backdrop:.1}/255, so the backdrop pass reached nothing"
    );

    // ---- 3. the health bar is red, in that order ---------------------------
    let health_red = block.channel(HEALTH_AT, 0);
    let health_green = block.channel(HEALTH_AT, 1);
    let health_blue = block.channel(HEALTH_AT, 2);
    eprintln!(
        "hud golden: health red {health_red:.1}, green {health_green:.1}, blue {health_blue:.1}"
    );
    assert!(
        health_red > health_green * HEALTH_REDNESS && health_red > health_blue * HEALTH_REDNESS,
        "the health bar reads red {health_red:.1} / green {health_green:.1} / blue \
         {health_blue:.1} — either no bar drew there, or the readback's channels were \
         written the wrong way round"
    );

    // ---- 4. the mana bar is blue, in that order ----------------------------
    //
    // Pointing the opposite way to claim 3, so the pair cannot both be satisfied
    // by a frame whose channels were rotated rather than swapped.
    let mana_blue = block.channel(MANA_AT, 2);
    let mana_red = block.channel(MANA_AT, 0);
    eprintln!("hud golden: mana blue {mana_blue:.1}, red {mana_red:.1}");
    assert!(
        mana_blue > DREW_AT_ALL && mana_blue > mana_red * MANA_BLUENESS,
        "the mana bar reads blue {mana_blue:.1} / red {mana_red:.1} — either no fill drew \
         there, or the readback's channels were written the wrong way round"
    );
}

/// **The frame the binary presented, against the checked-in golden.**
#[test]
#[ignore = "needs a real GPU and a backend pin; run tests/run-hud-golden.sh"]
fn the_frame_the_binary_wrote_matches_its_golden() {
    let backend = required_backend("tests/run-hud-golden.sh");
    let (image, adapter) = SampleRun {
        name: "hud",
        binary: env!("CARGO_BIN_EXE_hud"),
        tmp_dir: env!("CARGO_TARGET_TMPDIR"),
        file: "panels.png",
        frames: FRAMES,
        extent: EXTENT,
        args: &[],
        stdout_contains: &["wave 1"],
        simulation_advanced: false,
    }
    .screenshot(&backend);
    eprintln!("hud golden: device on {adapter}");
    inspect(&image);

    let reference = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/panels.png");
    let comparison = Golden::new(reference)
        .check(&image)
        .expect("the reference is readable")
        .into_result()
        .unwrap_or_else(|message| panic!("on {backend}: {message}"));
    eprintln!("hud golden: panels on {backend} — {}", comparison.summary());
}
