//! Where breakout's high score is kept.
//!
//! | Target | Where |
//! | --- | --- |
//! | native, windowed | `~/.config/breakout/high_score.bin` |
//! | native, `--headless` | nowhere, so a CI run leaves no trace |
//! | `wasm32` | the Origin Private File System |
//!
//! # This file used to be two hundred lines
//!
//! It held a `Backing` enum, three platform arms, the little-endian encode, the
//! corrupt-file case and the headless rule — and so did
//! `apps/flappy/src/best.rs`, `apps/asteroids/src/best.rs` and
//! `apps/horde/src/best.rs`, line for line, under different type names. All of
//! it is [`crcbl::store::record::Record`] now.
//!
//! What is left is what genuinely is breakout's, and it is what the engine
//! could not have guessed: **which application directory, which file name, and
//! — in the browser — which store**. The OPFS handle is installed by this
//! sample's own shim before the restore pass runs, and nothing in `crcbl-store`
//! can reach for it.
//!
//! A page whose shim never restored answers
//! [`StorageError::Pending`](crcbl::store::StorageError::Pending), which
//! `Record` treats as "no previous save" for reading and *not* as a reason to
//! skip writing — the write is what makes the next session's read succeed.

use crcbl::store::record::Record;

/// The application directory. `~/.config/breakout/` on Linux.
///
/// Names the config directory natively. A browser has no directory to name
/// and ignores it — see `Backing::platform`.
const APP: &str = "breakout";

/// The file inside it.
const HIGH_SCORE_FILE: &str = "high_score.bin";

/// Opens the high score, or an in-memory one for a headless run.
#[must_use]
pub fn open(headless: bool) -> Record {
    Record::for_app(APP, HIGH_SCORE_FILE, headless)
}
