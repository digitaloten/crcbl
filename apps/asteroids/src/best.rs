//! Where asteroids's best score is kept.
//!
//! | Target | Where |
//! | --- | --- |
//! | native, windowed | `~/.config/asteroids/best.bin` |
//! | native, `--headless` | nowhere, so a CI run leaves no trace |
//! | `wasm32` | the Origin Private File System |
//!
//! # This file used to be two hundred lines
//!
//! It held a `Backing` enum, three platform arms, the little-endian encode, the
//! corrupt-file case and the headless rule — and so did the same file in the
//! other three samples, line for line, under different type names. All of it is
//! [`crcbl::store::record::Record`] now.
//!
//! What is left is what genuinely is this game's, and it is what the engine
//! could not have guessed: **which application directory, which file name, and
//! — in the browser — which store**. The OPFS handle is installed by this
//! sample's own shim before the restore pass runs, and nothing in `crcbl-store`
//! can reach for it.

use crcbl::store::record::Record;

/// The application directory. `~/.config/asteroids/` on Linux.
///
/// Names the config directory natively. A browser has no directory to name
/// and ignores it — see `Backing::platform`.
const APP: &str = "asteroids";

/// The file inside it.
const BEST_FILE: &str = "best.bin";

/// Opens the best score, or an in-memory one for a headless run.
#[must_use]
pub fn open(headless: bool) -> Record {
    Record::for_app(APP, BEST_FILE, headless)
}
