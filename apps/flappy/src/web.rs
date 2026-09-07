//! The browser entry point: what the JS shim in `web/` calls.
//!
//! `apps/flappy` is a `cdylib` on `wasm32-unknown-unknown`, and this module is
//! the only thing in it a browser can reach. Everything here is an `extern "C"`
//! export with `#[unsafe(no_mangle)]`; there are **no imports**.
//!
//! # Only the symbol names are this game's
//!
//! The state machine behind these exports, the log queue and the five-call
//! protocol are [`crcbl::web`], and [`crcbl::web_exports!`] writes the ten
//! symbols listed below. That module is also where the reasons live: why
//! start-up is polled rather than blocking, why the clock is the browser's, and
//! why a sample's wasm module imports nothing of its own.
//!
//! What is left here is what is genuinely flappy's: the [`WebPending`](crcbl::web::WebPending) impl,
//! which opens the game with its own [`Options`], and the two accessors
//! `crate::best` reads a score through. The symbol names stay here too, written
//! out one per line — two demos can be open in one browser and the exports must
//! not collide, so the macro takes each name as an argument rather than building
//! it from a prefix.
//!
//! # The symbols this module exports
//!
//! `__crcbl_flappy_` is this module's prefix. **What each of the ten lifecycle
//! symbols means, the four other ABI prefixes a page drives, the status codes
//! and the order a page calls them all in are [`crcbl::web`]'s module docs**,
//! written once rather than once a sample.
//!
//! [`__crcbl_flappy_prepare`], [`__crcbl_flappy_log_level`],
//! [`__crcbl_flappy_boot`], [`__crcbl_flappy_frame`],
//! [`__crcbl_flappy_status`], [`__crcbl_flappy_shutdown`],
//! [`__crcbl_flappy_error_ptr`], [`__crcbl_flappy_error_len`],
//! [`__crcbl_flappy_log_take`], [`__crcbl_flappy_log_ptr`].

use std::rc::Rc;

use crcbl::store::web::FetchSource;

use crate::app::{Loop, PendingLoop};
use crate::args::Options;

// ---------------------------------------------------------------------------
// This game's half of the lifecycle
// ---------------------------------------------------------------------------

// **There is no `WebLoop` impl here.** `crcbl::web` blanket-implements it for
// every `crcbl::engine::Loop`, and the two halves that were ever this game's —
// its name and the log line a finished run is worth — are `HostedGame::NAME`
// and `HostedGame::log_summary` in `app.rs`. What is left below is start-up,
// which stays here because the options it opens with are flappy's.
// **`WebPending` is deliberately not imported.** The macro's guard against a
// missing inherent method resolves `PendingLoop::poll` by path, and an import
// would let that resolve to the trait method instead — which is the infinite
// recursion the guard exists to catch. See `crcbl::impl_web_pending`.
crcbl::impl_web_pending!(PendingLoop, Loop, Options, crate::app::FlappyError);

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

crcbl::web_exports! {
    pending: PendingLoop<dyn crcbl::shell::Shell>,
    prepare: __crcbl_flappy_prepare,
    log_level: __crcbl_flappy_log_level,
    boot: __crcbl_flappy_boot,
    frame: __crcbl_flappy_frame,
    status: __crcbl_flappy_status,
    shutdown: __crcbl_flappy_shutdown,
    error_ptr: __crcbl_flappy_error_ptr,
    error_len: __crcbl_flappy_error_len,
    log_take: __crcbl_flappy_log_take,
    log_ptr: __crcbl_flappy_log_ptr,
}

/// The asset source the shim pre-loads into, if `prepare` ran.
#[must_use]
pub fn asset_source() -> Option<Rc<FetchSource>> {
    STORAGE.with(|slot| slot.borrow().as_ref().map(|(_, assets)| Rc::clone(assets)))
}
