//! The browser entry point: what the JS shim in `web/` calls.
//!
//! `apps/horde` is a `cdylib` on `wasm32-unknown-unknown`, and this module
//! is the only thing in it a browser can reach. Everything here is an
//! `extern "C"` export with `#[unsafe(no_mangle)]`; there are **no imports**.
//!
//! # Only the symbol names are this game's
//!
//! The state machine behind these exports, the log queue and the five-call
//! protocol are [`crcbl::web`], and [`crcbl::web_exports!`] writes the lifecycle
//! symbols listed below. That module is also where the reasons live: why
//! start-up is polled rather than blocking, why the clock is the browser's, and
//! why a sample's wasm module imports nothing of its own.
//!
//! What is left here is what is genuinely horde's: the [`WebPending`](crcbl::web::WebPending) impl,
//! which opens the game with its own [`Options`], the two accessors
//! `crate::best` reads a score through, and the three exports at the bottom:
//! the scale fixture `--prefill` already is on a command line, and the pair of
//! thread-evidence counters, which no other sample has because no other sample
//! runs a pass on the job pool. The symbol names stay here too, written out one per line —
//! two demos can be open in one browser and the exports must not collide, so the
//! macro takes each name as an argument rather than building it from a prefix.
//!
//! # The symbols this module exports
//!
//! `__crcbl_horde_` is this module's prefix. **What each of the ten lifecycle
//! symbols means, the four other ABI prefixes a page drives, the status codes
//! and the order a page calls them all in are [`crcbl::web`]'s module docs**,
//! written once rather than once a sample.
//!
//! [`__crcbl_horde_prepare`], [`__crcbl_horde_log_level`],
//! [`__crcbl_horde_boot`], [`__crcbl_horde_frame`], [`__crcbl_horde_status`],
//! [`__crcbl_horde_shutdown`], [`__crcbl_horde_error_ptr`],
//! [`__crcbl_horde_error_len`], [`__crcbl_horde_log_take`],
//! [`__crcbl_horde_log_ptr`].
//!
//! ## …and the ones only this sample has
//!
//! | Symbol | Signature (wasm) | Meaning |
//! | --- | --- | --- |
//! | [`__crcbl_horde_prefill`] | `(u32) -> u32` | Stage N enemies and start the run, before `boot`. The count recorded, or `0` if it is too late. |
//! | [`__crcbl_horde_sim_threads`] | `() -> u32` | How many distinct threads have run a steering chunk. |
//! | [`__crcbl_horde_sim_workers`] | `() -> u32` | How many workers the pool that last ran the steering pass has. |

use std::rc::Rc;

use crcbl::store::web::FetchSource;

use crate::app::{Loop, PendingLoop};
use crate::args::Options;

// `STATUS_PREPARED` is the shim's wire format and has exactly one definition;
// see [`crcbl::web`]. Imported rather than reached through the path because the
// export below reads it to decide whether start-up has gone past the point
// where its value would still be taken.
use crcbl::web::STATUS_PREPARED;

// ---------------------------------------------------------------------------
// This game's half of the lifecycle
// ---------------------------------------------------------------------------

// **There is no `WebLoop` impl here.** `crcbl::web` blanket-implements it for
// every `crcbl::engine::Loop`, and the two halves that were ever this game's —
// its name and the log line a finished run is worth — are `HostedGame::NAME`
// and `HostedGame::log_summary` in `app.rs`. What is left below is start-up,
// which stays here because the options it opens with are horde's.
// **`WebPending` is deliberately not imported.** The macro's guard against a
// missing inherent method resolves `PendingLoop::poll` by path, and an import
// would let that resolve to the trait method instead — which is the infinite
// recursion the guard exists to catch. See `crcbl::impl_web_pending`.
crcbl::impl_web_pending!(PendingLoop, Loop, Options, crate::app::HordeError);

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

crcbl::web_exports! {
    pending: PendingLoop<dyn crcbl::shell::Shell>,
    prepare: __crcbl_horde_prepare,
    log_level: __crcbl_horde_log_level,
    boot: __crcbl_horde_boot,
    frame: __crcbl_horde_frame,
    status: __crcbl_horde_status,
    shutdown: __crcbl_horde_shutdown,
    error_ptr: __crcbl_horde_error_ptr,
    error_len: __crcbl_horde_error_len,
    log_take: __crcbl_horde_log_take,
    log_ptr: __crcbl_horde_log_ptr,
}

/// The asset source the shim pre-loads into, if `prepare` ran.
#[must_use]
pub fn asset_source() -> Option<Rc<FetchSource>> {
    STORAGE.with(|slot| slot.borrow().as_ref().map(|(_, assets)| Rc::clone(assets)))
}

// ---------------------------------------------------------------------------
// The scale fixture, and the steering pass's thread evidence
// ---------------------------------------------------------------------------

/// Stage `enemies` enemies across the arena before the first tick, and start the
/// run without waiting at the title screen.
///
/// **This is `--prefill`, reachable from a page**, and it is the same code path:
/// `crate::app`'s `assemble` reads [`Options::prefill`](crate::args::Options)
/// and calls [`Game::stage_field`](crate::game::Game::stage_field), whether the
/// options came from a command line or from here. The cap rises to fit, exactly
/// as the flag's docs say.
///
/// **Call it before [`__crcbl_horde_boot`]**, because that is what builds the
/// game: the value is read once, when the options are taken. Answers the count
/// recorded, or `0` when start-up has already gone past the point where it would
/// be read — which is the only way this can refuse, and the only mistake a page
/// can make with it.
///
/// `0` is the demo site's answer: nothing calls this unless a page was asked to,
/// and a run that does not call it waits at the title screen with an empty arena
/// exactly as it does today.
#[unsafe(no_mangle)]
pub extern "C" fn __crcbl_horde_prefill(enemies: u32) -> u32 {
    if __crcbl_horde_status() > STATUS_PREPARED {
        return 0;
    }
    crate::args::request_prefill(enemies as usize);
    enemies
}

// ---------------------------------------------------------------------------
// The steering pass's thread evidence
// ---------------------------------------------------------------------------

/// How many distinct threads have run a steering chunk.
///
/// **The one thing in this ABI that can tell a threaded run from an inline
/// one.** Everything else here — the status code, the error string, the log
/// queue — describes what the game did, and a run whose steering pass was split
/// across workers does exactly what a run that ran every chunk on the calling
/// thread does: `crate::game`'s `steer_enemies` is bit-identical at any worker
/// count, by construction rather than by luck. So the frames say nothing, and
/// this is what `web/run-horde-threads-e2e.sh` asserts on.
///
/// `1` is the published site's answer and is not a failure: the browser's spawn
/// backend refuses every spawn until a page has announced itself through
/// `__crcbl_web_jobs_host_ready`, so a demo loaded from an origin that cannot
/// construct a shared memory runs the pass inline. `0` before the first tick
/// that steered anything.
///
/// Only ever grows. See `crate::game::steer_threads` for why a thread that
/// cannot read its own thread-local is missed rather than counted twice.
#[unsafe(no_mangle)]
pub extern "C" fn __crcbl_horde_sim_threads() -> u32 {
    crate::game::steer_threads()
}

/// How many workers the pool that last ran the steering pass has.
///
/// The other half of the question, and what separates the two ways
/// [`__crcbl_horde_sim_threads`] stays at one: zero says the run degraded onto
/// the inline path and no worker was ever asked for, and a non-zero count says
/// the workers exist and took no chunk. `0` before the first tick that steered
/// anything, which is the same answer a pool with no workers gives — the two are
/// separated by the thread count, not by this.
#[unsafe(no_mangle)]
pub extern "C" fn __crcbl_horde_sim_workers() -> u32 {
    crate::game::steer_workers()
}
