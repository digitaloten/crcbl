//! The browser entry point: what the JS shim in `web/` calls.
//!
//! `apps/breach` is a `cdylib` on `wasm32-unknown-unknown`, and this module is
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
//! What is left here is what is genuinely breach's: the
//! [`WebPending`](crcbl::web::WebPending) impl, which opens the sample with its
//! own [`Options`], and [`__crcbl_breach_map`], which is `--map` reachable from
//! a page. The symbol names stay here too, written out one per line — two demos
//! can be open in one browser and the exports must not collide, so the macro
//! takes each name as an argument rather than building it from a prefix.
//!
//! # What the page shows, and what decides it
//!
//! `Options::default()` is the whole of the page's configuration: a page has no
//! argv, so the tick rate and the pistol are the ones the binary opens with, and
//! the map is whichever [`__crcbl_breach_map`] was last told — the **firing
//! range** unless something asked otherwise, which is what keeps
//! `/demos/breach/` the page it has always been.
//!
//! What a visitor sees on the range is it shooting itself within a second of
//! arriving, and their first step or trigger pull taking it over —
//! [`crate::game`] carries that argument. What they see on `?map=practice` is
//! three bots already walking their patrols, and one of them already shooting
//! at them: that map needs no warm-up because it is not empty.
//!
//! # This page is `docs/plan/sample/11-breach.md`'s milestone 0, and no more
//!
//! That doc's milestone 1 onward is a competitive shooter, and it says in as
//! many words that a browser build of it would be a claim the platform cannot
//! back: no anti-cheat, no unreliable channel and no measurable latency. Raw
//! mouse input was on that list and is not any more: the web shell grants
//! `POINTER_LOCK` and `RAW_POINTER_MOTION`, because `requestPointerLock` takes
//! `unadjustedMovement: true` and that is the bypass the capability exists to
//! promise. So this page looks around with the mouse, on the same
//! `ShellCaps::has_mouselook` binding the native build takes, and the arrows
//! remain the second binding [`crate::app`] describes where it is made.
//!
//! What milestone 0 *is* about is the fallback paths, and a browser is where
//! they are not hypothetical: there is no mesh stage and no ray query, so the
//! frame goes through `IndirectPerBatch` and `LightingPath::Rasterised` by
//! construction. [`crate::gpu::Paths`] is what names them, and the `[HUD]`
//! heartbeat is where a browser can read them.
//!
//! **What this sample does not add to the macro.** There is no `asset_source`
//! accessor here, because breach has nothing to read out of it: the range is
//! built in code, it keeps no score across runs and no save, and every byte it
//! draws with (the geometry, the shaders, the glyph atlas, and
//! `crate::loadout`'s item table) is compiled into the module. The two backends
//! are still installed by the macro's `prepare`, because the shared shim's boot
//! sequence drives both ABIs before it boots the demo and both must answer.
//!
//! # The symbols this module exports
//!
//! `__crcbl_breach_` is this module's prefix. **What each of the ten lifecycle
//! symbols means, the four other ABI prefixes a page drives, the status codes
//! and the order a page calls them all in are [`crcbl::web`]'s module docs**,
//! written once rather than once a sample.
//!
//! [`__crcbl_breach_prepare`], [`__crcbl_breach_log_level`],
//! [`__crcbl_breach_boot`], [`__crcbl_breach_frame`],
//! [`__crcbl_breach_status`], [`__crcbl_breach_shutdown`],
//! [`__crcbl_breach_error_ptr`], [`__crcbl_breach_error_len`],
//! [`__crcbl_breach_log_take`], [`__crcbl_breach_log_ptr`].
//!
//! ## …and the ones only this sample has
//!
//! | Symbol | Signature (wasm) | Meaning |
//! | --- | --- | --- |
//! | [`__crcbl_breach_map`] | `(u32) -> u32` | Which map to open, as an index into [`MapChoice::ALL`](crate::map::MapChoice::ALL). **Before `boot`**. `1` if it was recorded, `0` if it was refused. |

use crate::app::{Loop, PendingLoop};
use crate::args::Options;

// `STATUS_PREPARED` is the shim's wire format and has exactly one definition;
// see [`crcbl::web`]. Imported rather than reached through the path because the
// export below reads it to decide whether start-up has gone past the point
// where its value would still be taken.
use crate::map::MapChoice;
use crcbl::web::STATUS_PREPARED;

// ---------------------------------------------------------------------------
// This sample's half of the lifecycle
// ---------------------------------------------------------------------------

// **There is no `WebLoop` impl here.** `crcbl::web` blanket-implements it for
// every `crcbl::engine::Loop`, and the two halves that were ever this sample's
// — its name and the log line a finished run is worth — are `HostedGame::NAME`
// and `HostedGame::log_summary` in `app.rs`. What is left is start-up, and the
// macro below writes it.
//
// Breach's `Options` is the shared set and nothing else, so the browser wants
// it exactly as the binary's default builds it.
//
// **`WebPending` is deliberately not imported.** The macro's guard against a
// missing inherent method resolves `PendingLoop::poll` by path, and an import
// would let that resolve to the trait method instead — which is the infinite
// recursion the guard exists to catch. See `crcbl::impl_web_pending`.
crcbl::impl_web_pending!(PendingLoop, Loop, Options, crate::app::BreachError);

// ---------------------------------------------------------------------------
// Which map, reachable from a page
// ---------------------------------------------------------------------------

/// Opens the run on [`MapChoice::ALL`]`[map]` rather than on the default one.
///
/// **This is `--map`, reachable from a page**, and it is the same code path:
/// `crate::app`'s `assemble` reads [`Options::map`](crate::args::Options) and
/// hands it to both the simulation and the renderer, whether the options came
/// from a command line or from here. `web/demos/breach/main.js` is what turns a
/// `?map=` query into this call, so the name a visitor types and the name
/// `--map` takes are one table — [`MapChoice::from_name`].
///
/// **Call it before [`__crcbl_breach_boot`]**, because that is what builds the
/// game: the value is read once, when the options are taken. Answers `1` if the
/// choice was recorded and `0` if it was refused — either because start-up has
/// already gone past the point where it would be read, or because `map` is not
/// an index into [`MapChoice::ALL`]. Those are the only two mistakes a page can
/// make with it, and both leave the run on the map it would have opened anyway.
#[unsafe(no_mangle)]
pub extern "C" fn __crcbl_breach_map(map: u32) -> u32 {
    if __crcbl_breach_status() > STATUS_PREPARED {
        return 0;
    }
    let Ok(index) = usize::try_from(map) else {
        return 0;
    };
    let Some(&choice) = MapChoice::ALL.get(index) else {
        return 0;
    };
    crate::args::request_map(choice);
    1
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

crcbl::web_exports! {
    pending: PendingLoop<dyn crcbl::shell::Shell>,
    prepare: __crcbl_breach_prepare,
    log_level: __crcbl_breach_log_level,
    boot: __crcbl_breach_boot,
    frame: __crcbl_breach_frame,
    status: __crcbl_breach_status,
    shutdown: __crcbl_breach_shutdown,
    error_ptr: __crcbl_breach_error_ptr,
    error_len: __crcbl_breach_error_len,
    log_take: __crcbl_breach_log_take,
    log_ptr: __crcbl_breach_log_ptr,
}
