//! Sparks — the native front end.
//!
//! ```text
//! sparks [--headless] [--frames N] [--size WxH] [--tick-hz N] [--seed N]
//! ```
//!
//! Argv in, exit code out, and nothing else: the sample itself is the
//! `crcbl_sparks` library this binary links, which is also what the browser's
//! wasm entry point drives.
//!
//! Exit codes: 0 ran, 1 it failed, 2 bad arguments.

use std::process::ExitCode;

use crcbl_sparks::{USAGE, parse, run};

fn main() -> ExitCode {
    crcbl::args::run_front_end(
        "sparks",
        USAGE,
        parse(std::env::args().skip(1)),
        run,
        |summary| {
            format!(
                "sparks: {} frames, {} ticks on the {} shell at {}x{}, {} \
                 ({} live, {} instance(s) drawn, {} spawn(s) clamped, {:?})",
                summary.run.frames,
                summary.run.ticks,
                summary.run.backend,
                summary.run.extent.0,
                summary.run.extent.1,
                // What the window system actually did, not what `--fullscreen`
                // asked for. It is free to refuse.
                summary.run.mode,
                summary.live,
                summary.drawn,
                summary.clamped,
                summary.run.exit,
            )
        },
    )
}
