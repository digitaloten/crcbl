//! The browser entry point: what the JS shim in `web/` calls.
//!
//! `apps/sparks` is a `cdylib` on `wasm32-unknown-unknown`, and this module is
//! the only thing in it a browser can reach. Everything here is an `extern "C"`
//! export with `#[unsafe(no_mangle)]`; there are **no imports**.
//!
//! # Only the symbol names are this sample's
//!
//! The state machine behind these exports, the log queue and the five-call
//! protocol are [`crcbl::web`], and [`crcbl::web_exports!`] writes the ten
//! symbols listed below. That module is also where the reasons live: why
//! start-up is polled rather than blocking, why the clock is the browser's, and
//! why a sample's wasm module imports nothing of its own.
//!
//! What is left here is what is genuinely sparks': the
//! [`WebPending`](crcbl::web::WebPending) impl, which opens the sample with its
//! own [`Options`]. The symbol names stay here too, written out one per line —
//! two demos can be open in one browser and the exports must not collide, so the
//! macro takes each name as an argument rather than building it from a prefix.
//!
//! **What this sample does not add to that.** There is no `asset_source`
//! accessor here, because sparks has nothing to read out of it: the stage is
//! `crcbl::greybox` primitives, the effects are values in `crate::effects`, and
//! every byte it draws with is compiled into the module. The two backends are
//! still installed by the macro's `prepare`, because the shared shim's boot
//! sequence drives both ABIs before it boots the demo and both must answer.
//!
//! # The symbols this module exports
//!
//! `__crcbl_sparks_` is this module's prefix. **What each of the ten lifecycle
//! symbols means, the four other ABI prefixes a page drives, the status codes
//! and the order a page calls them all in are [`crcbl::web`]'s module docs**,
//! written once rather than once a sample.
//!
//! [`__crcbl_sparks_prepare`], [`__crcbl_sparks_log_level`],
//! [`__crcbl_sparks_boot`], [`__crcbl_sparks_frame`],
//! [`__crcbl_sparks_status`], [`__crcbl_sparks_shutdown`],
//! [`__crcbl_sparks_error_ptr`], [`__crcbl_sparks_error_len`],
//! [`__crcbl_sparks_log_take`], [`__crcbl_sparks_log_ptr`].

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
// Sparks' `Options` has one field of its own and the browser wants it at its
// default: a page opens on the published seed, which is what makes the demo
// everyone loads the same demo — and what makes the per-particle hash's replay
// something a visitor can see rather than a property in a test.
//
// **`WebPending` is deliberately not imported.** The macro's guard against a
// missing inherent method resolves `PendingLoop::poll` by path, and an import
// would let that resolve to the trait method instead — which is the infinite
// recursion the guard exists to catch. See `crcbl::impl_web_pending`.
crcbl::impl_web_pending!(PendingLoop, Loop, Options, crate::app::SparksError);

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

crcbl::web_exports! {
    pending: PendingLoop<dyn crcbl::shell::Shell>,
    prepare: __crcbl_sparks_prepare,
    log_level: __crcbl_sparks_log_level,
    boot: __crcbl_sparks_boot,
    frame: __crcbl_sparks_frame,
    status: __crcbl_sparks_status,
    shutdown: __crcbl_sparks_shutdown,
    error_ptr: __crcbl_sparks_error_ptr,
    error_len: __crcbl_sparks_error_len,
    log_take: __crcbl_sparks_log_take,
    log_ptr: __crcbl_sparks_log_ptr,
}
