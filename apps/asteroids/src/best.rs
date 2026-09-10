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

#[cfg(test)]
mod tests {
    use super::*;

    /// **A headless run must leave nothing behind**, and this module's whole
    /// part in that rule is the `headless` it hands
    /// [`Record::for_app`](crcbl::store::record::Record::for_app).
    ///
    /// A literal `false` here would write the best score into whoever's config
    /// directory the suite runs as, and the number would still come back — from
    /// the file instead of from memory — so every other assertion about the
    /// best would pass exactly as it does now. What is read is therefore that
    /// a second open starts over, which is the one reading a write would move.
    #[test]
    fn a_headless_run_keeps_its_best_in_memory_and_writes_nothing() {
        let mut record = open(true);
        assert!(record.raise(500), "it still tracks the value in memory");
        assert_eq!(record.get(), 500);
        assert_eq!(open(true).get(), 0, "a headless run left a file behind");
    }
}
