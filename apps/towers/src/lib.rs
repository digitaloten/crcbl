//! Towers — co-op tower defense, and the ladder's flagship.
//!
//! `docs/plan/sample/07-towers.md`, **milestone 1, slices 1 and 2**: the solo
//! loop on a hardcoded map, natively and in a browser. Creeps walk a path,
//! towers shoot them, kills pay gold, a scripted table of waves runs out, and
//! the run is won or lost. That document's status section is the list of what
//! each remaining slice owes.
//!
//! # What it proves
//!
//! Three things, and every one of them is a `crcbl-phys` L0 query the plan
//! named this sample as the forcing function for:
//!
//! * **A trigger volume is how a creep reaches the exit.**
//!   [`crcbl::phys::PhysicsWorld::set_trigger`] makes the volume non-solid, so
//!   sweeps and rays pass through it and only
//!   [`overlap_sphere`](crcbl::phys::PhysicsWorld::overlap_sphere) reports it —
//!   which is exactly the pair a "did anything get in here?" volume wants.
//!   [`map`] registers it and [`creep::has_reached_the_exit`] asks it.
//! * **Acquisition is a sphere overlap, and the filter is the interesting
//!   half.** The same query hands a tower the ground slab and the exit volume,
//!   so [`tower::acquire`] answers with a creep or with nothing.
//! * **CCD against a target that is itself moving.** A bolt covers more ground
//!   in one tick than a creep is wide, and the creeps have already walked by
//!   the time the bolts sweep — so
//!   [`sweep_sphere`](crcbl::phys::PhysicsWorld::sweep_sphere) over the
//!   segment is the only thing that sees the hit.
//!   `tower::tests::a_bolt_hits_a_creep_that_a_test_at_either_end_of_the_tick_would_miss`
//!   is that claim with both static readings taken beside it.
//!
//! ```text
//!   keys ──▶ ActionMap ──▶ Controls ──wire──▶ Intent ──▶ Stage
//!                                                          │
//!   WaveSystem ──▶ CreepSystem ──▶ exit trigger ──▶ lives  │
//!        │              │                                  │
//!        │              └──▶ ProjectileSystem ──▶ TowerSystem
//!        └──────────────────────────────▶ EconomySystem ──▶ gold, win, lose
//! ```
//!
//! # It is a real client/server sample
//!
//! `docs/plan/sample/00-samples-overview.md` rule 2 has no exemption for a
//! tower defense, and this sample's own document is explicit that solo and
//! co-op are one build: `PlaceTower` and `StartWave` are **commands** the
//! client seals into bytes and the server validates over an
//! `InMemoryTransport`. A refused command is counted, which is how "the server
//! decides" is a number rather than a claim. What is not here is a second
//! player, because `crcbl-net` ships no transport but the loopback.
//!
//! # The path is a polyline, and the engine owes a spline
//!
//! That document asks for creeps that walk a spline. Nothing in `crcbl-phys` or
//! `crcbl-scene` offers a spline type, so [`path`] measures straight legs
//! between [`map::PATH`]'s waypoints and a creep turns a corner in one tick.
//! It is the one engine gap this slice found, and it is recorded rather than
//! worked around.
//!
//! # What is not here yet
//!
//! **Slice 1 is one map, one tower, one creep and three waves.** No splash or
//! slow towers and no upgrade tiers; no tanky or swarm creeps and no
//! world-space health bars; seven of the plan's ten waves; no `.crpix` art and
//! so no build menu worth the name (rule 11 is owed, not exempted); no spatial
//! audio (rule 8 is owed, not exempted); no save or resume; and no dev fly/walk
//! camera. There is no pointer or touch input either, on the page as well as in
//! the window — [`app`] says why a tap waits for the build menu.
//! `docs/plan/sample/07-towers.md` carries the list with what each would take.
//!
//! # One library, two front ends
//!
//! `src/main.rs` is argv and an exit code; everything else is here.
//! `src/web.rs` is the second front end — compiled only on `wasm32`, which is
//! why it is not linked on a host build — and it is what the demo site's shim
//! drives once per `requestAnimationFrame`.

pub mod app;
mod args;
pub mod camera;
pub mod creep;
pub mod game;
mod gpu;
pub mod map;
pub mod menu;
pub mod page;
pub mod path;
pub mod tower;
pub mod wave;

#[cfg(target_arch = "wasm32")]
pub mod web;

pub use app::{Loop, PendingLoop, Summary, Towers, TowersError, run, start, with_shell};
pub use args::{Invocation, Options, USAGE, parse};
pub use creep::{Creep, CreepView};
pub use game::{Controls, DEFAULT_TICK_HZ, Game, GameError, RenderState, Stats};
pub use gpu::{Gpu, Paths};
pub use map::{PLOTS, Plot};
pub use menu::{MenuAction, MenuKind, Menus};
pub use page::PageStats;
pub use tower::{Bolt, BoltOutcome, Tower};
pub use wave::{Outcome, WAVES, Wave, Waves};
