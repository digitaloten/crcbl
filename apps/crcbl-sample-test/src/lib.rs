//! The fixture every sample's golden suite drives its binary through.
//!
//! ```text
//! SampleRun { … }.screenshot(backend) ──▶ the compiled sample, --screenshot
//!                                            │
//!                                            ├─ its stdout: the frame count,
//!                                            │  the state, the tick count
//!                                            └─ the PNG it left behind ──▶ Image
//! ```
//!
//! # What a sample's golden suite actually is
//!
//! `apps/{asteroids,breakout,flappy,horde,hud}/tests/golden.rs` all do the same
//! thing and none of them owns a GPU: each runs the **compiled binary** with
//! `--screenshot`, reads back the file it wrote, makes a few ratio claims about
//! where the frame is bright and dark, and finally compares it against a
//! checked-in golden. `--screenshot` is an engine flag on `crcbl::args::Common`
//! rather than a per-sample one, which is the whole reason the frame under test
//! is the frame a player would have seen — menu, HUD and every pass the game
//! hung off the swapchain image included.
//!
//! Five copies of that machinery is duplicated **knowledge**: a fix to the
//! adapter-line parse or to the stale-file removal has to land five times, and
//! the copy that gets missed stays green while testing something slightly
//! different. It had already happened — see `docs/notes/samples.md`.
//!
//! # Why it is a crate rather than an include
//!
//! A test binary cannot reach another test binary's helper, so the only two
//! homes are a `#[path]`-included file and a crate. The crate wins because an
//! included file is compiled once per suite and each copy gets its own
//! `const`s, which is fine for five callers and surprising for the sixth.
//!
//! It cannot live in `crcbl-golden`: this spawns a sample binary and reads
//! [`crcbl::backend::BACKEND_ENV_VAR`], so it depends on `crcbl`, and
//! `crcbl-golden` is the leaf those suites already depend on. The arithmetic
//! that *doesn't* need `crcbl` went there instead — see
//! [`crcbl_golden::srgb`].

use std::path::PathBuf;
use std::process::Command;

use crcbl_golden::Image;

/// Which backend must draw, from the environment.
///
/// **Required, with no default.** Every backend draws a sample's frame
/// identically by construction, so a run that fell back to another one produces
/// a frame that passes and proves nothing about the one that was wanted —
/// `crcbl::backend::open` would otherwise answer the question for you. The same
/// argument each `run-*-golden.sh` makes, made where it can be enforced.
///
/// `harness` is the script that names one, quoted back at whoever ran the suite
/// by hand.
///
/// # Panics
///
/// When [`crcbl::backend::BACKEND_ENV_VAR`] is unset.
#[must_use]
pub fn required_backend(harness: &str) -> String {
    std::env::var(crcbl::backend::BACKEND_ENV_VAR).unwrap_or_else(|_| {
        panic!(
            "{} is not set, so nothing would pin the backend and a fallback would pass. \
             Run {harness}, which names one.",
            crcbl::backend::BACKEND_ENV_VAR
        )
    })
}

/// The adapter the binary opened, read out of its own log.
///
/// From the run rather than from the environment the test exported: a variable
/// that never reached the process and a pin that was honoured look identical
/// from outside. Each `run-*-golden.sh` reads this line back out of its suite
/// for the same reason.
///
/// # Panics
///
/// When the log names no adapter, which means the run never opened one.
#[must_use]
pub fn adapter_line(stderr: &str) -> String {
    stderr
        .lines()
        .find(|line| line.contains(" adapter \""))
        .map(|line| line[line.find("hal: ").map_or(0, |at| at + "hal: ".len())..].to_string())
        .unwrap_or_else(|| panic!("the run never said which adapter it opened:\n{stderr}"))
}

/// A frame, read in blocks of a fixed half-extent.
///
/// A block rather than a pixel, because a single pixel is a sample of the
/// rasteriser as much as of the picture: a glyph edge or a nine-slice seam
/// landing a pixel either way moves it. The half-extent is bound once because
/// every suite reads its whole frame at one size — passing it per call would be
/// an argument that never varies.
#[derive(Debug)]
pub struct Block<'a> {
    image: &'a Image,
    half: (u32, u32),
}

impl<'a> Block<'a> {
    /// Reads `image` in blocks reaching `half` pixels either side of a centre.
    #[must_use]
    pub fn new(image: &'a Image, half: (u32, u32)) -> Self {
        Self { image, half }
    }

    /// The mean brightness of the block around `centre`, out of 255.
    #[must_use]
    pub fn brightness(&self, centre: (u32, u32)) -> f32 {
        self.mean(centre, None)
    }

    /// The mean of one channel over the same block, out of 255.
    #[must_use]
    pub fn channel(&self, centre: (u32, u32), index: usize) -> f32 {
        self.mean(centre, Some(index))
    }

    /// `index` names a channel, or `None` averages the three colour channels.
    fn mean(&self, centre: (u32, u32), index: Option<usize>) -> f32 {
        let mut total = 0.0f32;
        let mut count = 0u32;
        let last = (self.image.width() - 1, self.image.height() - 1);
        for y in centre.1.saturating_sub(self.half.1)..=(centre.1 + self.half.1).min(last.1) {
            for x in centre.0.saturating_sub(self.half.0)..=(centre.0 + self.half.0).min(last.0) {
                let pixel = self.image.pixel(x, y).expect("inside the frame");
                total += match index {
                    Some(index) => f32::from(pixel[index]),
                    None => (f32::from(pixel[0]) + f32::from(pixel[1]) + f32::from(pixel[2])) / 3.0,
                };
                count += 1;
            }
        }
        total / count as f32
    }
}

/// One sample binary's screenshot run, described.
///
/// Every field is required and there is no [`Default`], on purpose: a new field
/// has to be a compile error at all five call sites, because the alternative is
/// a suite that silently stops checking whatever the field was added for.
#[derive(Debug)]
pub struct SampleRun<'a> {
    /// What the sample calls itself in failure messages — `"breakout"`.
    pub name: &'a str,
    /// The compiled binary, which a caller spells `env!("CARGO_BIN_EXE_…")`.
    ///
    /// Passed in rather than derived from [`name`](Self::name): that macro
    /// resolves in the caller's own test target and nowhere else.
    pub binary: &'a str,
    /// Where the frame is written, which a caller spells
    /// `env!("CARGO_TARGET_TMPDIR")`.
    ///
    /// Cargo gives an integration test that directory for exactly this, and it
    /// is already inside the `/target` ignore — so there is no new ignore rule
    /// and a reviewer has a path to open.
    pub tmp_dir: &'a str,
    /// The file's name inside [`tmp_dir`](Self::tmp_dir) — `"board.png"`.
    pub file: &'a str,
    /// How many frames the run presents before the one that gets written.
    pub frames: u32,
    /// The extent the checked-in golden was blessed at.
    pub extent: (u32, u32),
    /// Arguments this sample needs and the others do not — `apps/horde`'s
    /// `--prefill`, which is what puts a field on its frame at all.
    pub args: &'a [&'a str],
    /// Substrings the run's summary must contain.
    ///
    /// The state the golden was blessed on, named rather than assumed:
    /// `"WaitingToStart"` for a title screen, `"Playing"` for a prefilled run.
    /// A frame drawn from another state is a picture of a different game.
    pub stdout_contains: &'a [&'a str],
    /// Whether the summary's *simulated* tick count must have moved.
    ///
    /// Off for a sample whose summary carries no such count. Where it is on it
    /// is load-bearing — see [`screenshot`](Self::screenshot).
    pub simulation_advanced: bool,
}

impl SampleRun<'_> {
    /// Runs the real binary with `--screenshot` and hands back the frame it
    /// wrote, with the adapter line it drew on.
    ///
    /// **The stale file is removed first**, and its absence is what makes the
    /// assertion here mean anything: a `--screenshot` that quietly did nothing
    /// would otherwise pass on the previous run's picture forever.
    ///
    /// **The simulated tick count is the other half**, where
    /// [`simulation_advanced`](Self::simulation_advanced) asks for it. The
    /// pictures these suites guard are of menus over still fields, so a build
    /// whose `Game::tick` did nothing presented its frames, wrote a
    /// byte-identical image and passed every pixel claim — measured, by
    /// emptying `tick`. It has to be the *simulated* count and not the loop's:
    /// the loop counts the times it called `tick` and reads the same either
    /// way, while `sim_ticks` comes from `Game::ticks_run` and goes to zero.
    /// `apps/flappy`'s golden asserted the loop's number first, passed the
    /// frozen build, and was the same defect it was written to catch. Half of
    /// [`frames`](Self::frames) rather than the exact figure, because the exact
    /// one is the accumulator's business; zero is the case that matters.
    ///
    /// # Panics
    ///
    /// When the binary fails to run, exits non-zero, writes nothing, writes a
    /// frame at another extent, or reports a summary that does not match what
    /// this run asked for.
    #[must_use]
    pub fn screenshot(&self, backend: &str) -> (Image, String) {
        let name = self.name;
        let path = PathBuf::from(self.tmp_dir).join(self.file);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => panic!("could not clear {}: {error}", path.display()),
        }

        let frames = self.frames.to_string();
        let output = Command::new(self.binary)
            .args(["--backend", backend, "--frames", &frames])
            .args(self.args)
            .args([
                // Not because a headless run needs saying — `--screenshot`
                // turns it on — but because saying it is how these suites record
                // that the picture is of the offscreen ring and not of a window.
                "--headless",
                // It is on by default in a debug build and it draws frame times.
                // A golden of a frame with `0.007 ms` written on it is a golden
                // that fails on the next machine.
                "--no-debug-overlay",
                "--screenshot",
            ])
            .arg(&path)
            .output()
            .unwrap_or_else(|error| panic!("the {name} binary runs: {error}"));

        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert_eq!(
            output.status.code(),
            Some(0),
            "{name} exited {:?} on {backend}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            output.status.code()
        );
        // The binary's log, re-emitted whether or not it passed. `.output()`
        // keeps the child's stderr, and a green run would otherwise show none of
        // it — so a `vk validation:` line the layer wrote there would reach
        // nothing. Each `run-*-golden.sh` reads its suite's log for exactly that
        // line and for the messenger's own announcement, and both live in the
        // child's stderr, not this process's.
        eprint!("{stderr}");

        // The run has to have played the game, not merely started and stopped.
        // A frame written by a run that never reached the simulation is a
        // picture of start-up, and the summary is the only thing that can tell
        // the two apart from out here.
        assert!(
            stdout.contains(&format!("{} frames", self.frames)),
            "the summary does not say the run presented {} frames:\n{stdout}",
            self.frames
        );
        for wanted in self.stdout_contains {
            assert!(
                stdout.contains(wanted),
                "the summary does not say `{wanted}`, so this is not the state the golden was \
                 blessed on:\n{stdout}"
            );
        }
        if self.simulation_advanced {
            let simulated: u32 = stdout
                .split_once(" simulated)")
                .and_then(|(before, _)| before.rsplit('(').next())
                .and_then(|word| word.parse().ok())
                .unwrap_or_else(|| panic!("the summary names no simulated tick count:\n{stdout}"));
            assert!(
                simulated >= self.frames / 2,
                "the simulation advanced {simulated} times over {} frames, so it was not \
                 running and this image is of a game that never started:\n{stdout}",
                self.frames
            );
        }

        assert!(
            path.exists(),
            "{name} exited 0 and wrote no {} — `--screenshot` did nothing",
            path.display()
        );
        let image = Image::load_png(&path).expect("the screenshot is a readable PNG");
        assert_eq!(
            (image.width(), image.height()),
            self.extent,
            "the binary wrote a {}x{} frame, which is not the extent the golden was blessed at",
            image.width(),
            image.height()
        );
        (image, adapter_line(&stderr))
    }
}
