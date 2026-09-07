//! The browser entry point: what the JS shim in `web/` calls.
//!
//! `apps/shard` is a `cdylib` on `wasm32-unknown-unknown`, and this module is the
//! only thing in it a browser can reach. Everything here is an `extern "C"`
//! export with `#[unsafe(no_mangle)]`; there are **no imports**.
//!
//! # Only the symbol names are this sample's
//!
//! The state machine behind these exports, the log queue and the five-call
//! protocol are [`crcbl::web`], and [`crcbl::web_exports!`] writes the ten symbols
//! listed below. That module is also where the reasons live: why start-up is
//! polled rather than blocking, why the clock is the browser's, and why a sample's
//! wasm module imports nothing of its own.
//!
//! What is left here is what is genuinely shard's: the
//! [`WebPending`](crcbl::web::WebPending) impl, which opens the sample with its
//! own [`Options`]. The symbol names stay here too, written out one per line — two
//! demos can be open in one browser and the exports must not collide, so the macro
//! takes each name as an argument rather than building it from a prefix.
//!
//! # This page is `docs/plan/sample/15-shard.md`'s milestone 1, all six verbs in
//!
//! That doc's milestone 1 is "a complete play session — explore, fight, loot,
//! level, save, resume — in a browser, from the same build that runs natively".
//! **All six are here.** A visitor walks a torch-lit zone, puts its torches out,
//! fights what is standing in it, takes what it leaves — at the
//! [`Rarity`](crate::loot::Rarity) the seed rolled for it — into a grid inventory
//! they can drag items around in, levels on what both are worth, and comes back
//! to a character where they left them *still carrying it*: the save goes into
//! the Origin Private File System through [`crate::save`], on the same build that
//! writes it to the platform data directory natively. What a level is worth is a
//! deeper health pool and nothing else — there is no skill and no stat point to
//! spend it on, which `docs/backlog.md` carries with what it would take.
//!
//! The grid is `docs/plan/34-inventory.md`'s kit, which links into the wasm
//! module like any other: it has no IO, no clock and no `glam`, which is what
//! lets a browser build take it whole. See [`crate::loot`].
//!
//! # And it is the sample the fallback paths were built for
//!
//! `docs/plan/sample/15-shard.md` says why the web slice comes before the native
//! world: "the fallback paths are what every browser visitor and every Apple
//! machine runs, and a fallback proven after the fact is a fallback nobody
//! proved." A browser has no ray query, no mesh stage and no bindless, so the
//! frame goes through `IndirectPerBatch`, `ArrayPages` and
//! `LightingPath::Rasterised` **by construction** — with real content on top of
//! them: a zone of modular tiles, a torch over every brazier, shadow
//! tiles, screen-space occlusion and reflections, and a baked irradiance volume.
//! [`crate::gpu::Paths`] is what names them, and the `[HUD]` heartbeat is where a
//! browser can read them.
//!
//! **What this sample does not add to the macro.** There is no `asset_source`
//! accessor here, because shard has nothing to read out of one: the zone is built
//! in code and every byte it draws with (the geometry, the shaders, the glyph
//! atlas, the probe volume) is compiled into the module. The **OPFS** half is a
//! different matter and needs no accessor either — [`crate::save`] reaches the
//! store the macro's `prepare` installed through
//! [`crcbl::store::web::opfs::installed`], which is the same handle
//! [`crcbl::store::record::Backing::platform`] uses for a high score. Both
//! backends are installed by `prepare` because the shared shim's boot sequence
//! drives both ABIs before it boots the demo and both must answer.
//!
//! # The symbols this module exports
//!
//! `__crcbl_shard_` is this module's prefix. **What each of the ten lifecycle
//! symbols means, the four other ABI prefixes a page drives, the status codes
//! and the order a page calls them all in are [`crcbl::web`]'s module docs**,
//! written once rather than once a sample.
//!
//! [`__crcbl_shard_prepare`], [`__crcbl_shard_log_level`],
//! [`__crcbl_shard_boot`], [`__crcbl_shard_frame`], [`__crcbl_shard_status`],
//! [`__crcbl_shard_shutdown`], [`__crcbl_shard_error_ptr`],
//! [`__crcbl_shard_error_len`], [`__crcbl_shard_log_take`],
//! [`__crcbl_shard_log_ptr`].

use crate::app::{Loop, PendingLoop};
use crate::args::Options;

// ---------------------------------------------------------------------------
// This sample's half of the lifecycle
// ---------------------------------------------------------------------------

// **There is no `WebLoop` impl here.** `crcbl::web` blanket-implements it for
// every `crcbl::engine::Loop`, and the two halves that were ever this sample's —
// its name and the log line a finished run is worth — are `HostedGame::NAME` and
// `HostedGame::log_summary` in `app.rs`. What is left is start-up, and the macro
// below writes it.
//
// Shard's `Options` is the shared set and nothing else, so the browser wants it
// exactly as the binary's default builds it.
//
// **`WebPending` is deliberately not imported.** The macro's guard against a
// missing inherent method resolves `PendingLoop::poll` by path, and an import
// would let that resolve to the trait method instead — which is the infinite
// recursion the guard exists to catch. See `crcbl::impl_web_pending`.
crcbl::impl_web_pending!(PendingLoop, Loop, Options, crate::app::ShardError);

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

crcbl::web_exports! {
    pending: PendingLoop<dyn crcbl::shell::Shell>,
    prepare: __crcbl_shard_prepare,
    log_level: __crcbl_shard_log_level,
    boot: __crcbl_shard_boot,
    frame: __crcbl_shard_frame,
    status: __crcbl_shard_status,
    shutdown: __crcbl_shard_shutdown,
    error_ptr: __crcbl_shard_error_ptr,
    error_len: __crcbl_shard_error_len,
    log_take: __crcbl_shard_log_take,
    log_ptr: __crcbl_shard_log_ptr,
}
