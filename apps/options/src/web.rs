//! The browser entry point: what the JS shim in `web/` calls.
//!
//! `apps/options` is a `cdylib` on `wasm32-unknown-unknown`, and this module is
//! the only thing in it a browser can reach. Everything here is an `extern "C"`
//! export with `#[unsafe(no_mangle)]`; there are **no imports**.
//!
//! # Only the symbol names are this sample's
//!
//! The state machine behind these exports, the log queue and the five-call
//! protocol are [`crcbl::web`], and [`crcbl::web_exports!`] writes the ten
//! symbols listed below. That module is where the reasons live: why
//! start-up is polled rather than blocking, why the clock is the browser's, and
//! why a sample's wasm module imports nothing of its own. What is left here is
//! the [`WebPending`](crcbl::web::WebPending) impl, which opens the sample with
//! its own [`Options`], and the symbol names — written out one per line, since
//! two demos can be open in one browser and their exports must not collide.
//!
//! # The half of this sample that only exists here
//!
//! The point of a settings screen is that a setting outlives the run, and in a
//! browser tab that is a claim about the Origin Private File System and nothing
//! else. `__crcbl_options_prepare` installs the OPFS backend before anything
//! reads a key — which is what [`crcbl::web`]'s call ordering is for — and
//! [`SettingsStack::with_platform_storage`](crcbl::store::settings::SettingsStack::with_platform_storage)
//! resolves to it on `wasm32`, so [`Screen::opened`](crate::app::Screen::opened)
//! reads the player's file and `SAVE` writes it back with no arm of its own.
//! Where no store is installed the reader answers an empty stack and
//! [`SaveState::Nowhere`](crate::app::SaveState) is what the screen shows: a
//! settings screen that silently forgets is the worst version of this bug.
//!
//! # The symbols this module exports
//!
//! `__crcbl_options_` is this module's prefix. **What each of the ten
//! lifecycle symbols means, the four other ABI prefixes a page drives, the
//! status codes and the order a page calls them all in are [`crcbl::web`]'s
//! module docs**, written once rather than once a sample.
//!
//! [`__crcbl_options_prepare`], [`__crcbl_options_log_level`],
//! [`__crcbl_options_boot`], [`__crcbl_options_frame`],
//! [`__crcbl_options_status`], [`__crcbl_options_shutdown`],
//! [`__crcbl_options_error_ptr`], [`__crcbl_options_error_len`],
//! [`__crcbl_options_log_take`], [`__crcbl_options_log_ptr`].

use crate::app::{Loop, PendingLoop};
use crate::args::Options;

// ---------------------------------------------------------------------------
// This sample's half of the lifecycle
// ---------------------------------------------------------------------------

// **There is no `WebLoop` impl here.** `crcbl::web` blanket-implements it for
// every `crcbl::engine::Loop`, and the two halves that were ever this sample's
// — its name and the log line a finished run is worth — are `HostedGame::NAME`
// and `HostedGame::log_summary` in `app.rs`.
//
// **`WebPending` is deliberately not imported.** The macro's guard against a
// missing inherent method resolves `PendingLoop::poll` by path, and an import
// would let that resolve to the trait method instead — which is the infinite
// recursion the guard exists to catch. See `crcbl::impl_web_pending`.
crcbl::impl_web_pending!(PendingLoop, Loop, Options, crate::app::OptionsError);

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

crcbl::web_exports! {
    pending: PendingLoop<dyn crcbl::shell::Shell>,
    prepare: __crcbl_options_prepare,
    log_level: __crcbl_options_log_level,
    boot: __crcbl_options_boot,
    frame: __crcbl_options_frame,
    status: __crcbl_options_status,
    shutdown: __crcbl_options_shutdown,
    error_ptr: __crcbl_options_error_ptr,
    error_len: __crcbl_options_error_len,
    log_take: __crcbl_options_log_take,
    log_ptr: __crcbl_options_log_ptr,
}
