//! The simulation: the field, the creeps on it, the towers shooting them, and
//! the server that owns all three.
//!
//! ```text
//!  Stage ──▶ TowersModule ──▶ Server ──┐                     ┌──▶ Client
//!  (this file)                         └── InMemoryTransport ┘
//!                     │
//!                     └──▶ RenderState ──▶ crate::app, crate::page
//! ```
//!
//! # Solo is the same game over a loopback, and rule 2 has no exemption
//!
//! `docs/plan/sample/00-samples-overview.md` rule 2 is server-authoritative
//! always, and this sample's own document says the co-op build and the solo
//! build are one binary: `PlaceTower` and `StartWave` are **commands** the
//! client seals into bytes, the transport carries and the server validates. So
//! they are exactly that here, over `InMemoryTransport`, and the validation —
//! is that plot free, is there gold for it, is a wave already running — happens
//! on the server's side of the wire in `Stage::place_tower` and
//! [`crate::wave::Waves::start_now`]. A refused command is **counted**, so
//! "the server said no" is a number a run reports rather than something it
//! swallows.
//!
//! # What one tick does, and why it is in that order
//!
//! ```text
//!   1. commands       PlaceTower / StartWave / Restart, validated
//!   2. WaveSystem     the table releases a creep, if one is due
//!   3. CreepSystem    every creep walks and writes its sphere
//!   4. the exit       overlap_sphere vs the trigger volume → a life
//!   5. ProjectileSystem  every bolt sweeps → damage, a kill, gold
//!   6. TowerSystem    every ready tower acquires and fires
//!   7. EconomySystem  win at the end of the table, lose at zero lives
//! ```
//!
//! **The creeps move before the bolts sweep**, which is the whole of the CCD
//! claim: a bolt's segment is tested against where its target *is* at the end
//! of this tick rather than where it was at the start of it. And **the towers
//! fire last**, so a tower does not spend a shot on a creep another tower
//! killed on the same tick.
//!
//! # Nothing here does collision or intersection, and that is rule 9
//!
//! Every question about where things are is `crcbl-phys`':
//! [`overlap_sphere`](crcbl::phys::PhysicsWorld::overlap_sphere) for a tower's
//! range and for the exit volume,
//! [`sweep_sphere`](crcbl::phys::PhysicsWorld::sweep_sphere) for a bolt. This
//! file decides which query to ask and what an answer means.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use crcbl::ecs::{ClientInputs, GameModule, World};
use crcbl::math::DVec3;
use crcbl::net::ProtocolCompatibility;
use crcbl::phys::{ColliderId, PhysicsWorld};
use crcbl::session::Loopback;

use crate::creep::{self, Creep, CreepView};
use crate::map::{self, MAX_BOLTS, PLOTS};
use crate::tower::{self, Bolt, BoltOutcome, Tower};
use crate::wave::{self, MAX_CREEPS, Outcome, STARTING_GOLD, STARTING_LIVES, Waves};

/// Distinct from every other sample's, because they are distinct protocols: a
/// client built for one must not hand-shake with a server running another. The
/// low half spells `TWR`.
const COMPATIBILITY: ProtocolCompatibility = ProtocolCompatibility {
    protocol_version: 1,
    engine_build_id: 0x0043_5243_424C,
    schema_hash: 0x0000_0054_5752,
};

/// The default simulation rate. Reaches the server, the client and the stage,
/// so there is exactly one rate in the process.
pub const DEFAULT_TICK_HZ: u32 = 60;

/// How often the `[HUD]` heartbeat is logged, in ticks: one second of simulated
/// time at [`DEFAULT_TICK_HZ`].
pub const HEARTBEAT_TICKS: u64 = 60;

/// How long a finished run is left on screen before it starts again, in
/// seconds.
///
/// **A demo that has stopped is indistinguishable from a loop that has
/// stopped**, which is the argument `apps/breach`'s warm-up makes about its
/// empty room: a browser visitor arriving at a field with `LOST` written across
/// it and nothing moving cannot tell the difference. Long enough to read the
/// result, short enough that the next wave is on its way before anyone
/// reloads. `R` restarts a run without waiting.
pub const RESTART_S: f64 = 4.0;

// ---------------------------------------------------------------------------
// Controls and the wire
// ---------------------------------------------------------------------------

/// What the player is asking for this tick.
///
/// Every field is an **edge**: building, starting a wave and restarting are
/// things a key press does once, not things a held key does sixty times a
/// second. Which plot is highlighted is not here at all — that is presentation,
/// and what crosses the wire is the plot a player actually pressed build on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Controls {
    /// Build a tower on this plot.
    pub place: Option<u8>,
    /// Start the next wave now rather than at the end of the build phase.
    pub start_wave: bool,
    /// Throw the run away and start again.
    pub restart: bool,
}

/// One client's command frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Intent {
    place: Option<u8>,
    start_wave: bool,
    restart: bool,
}

const INTENT_START: u8 = 1 << 0;
const INTENT_RESTART: u8 = 1 << 1;

/// Every bit the flag byte defines. One set outside this mask is a frame
/// something other than [`Intent::to_wire`] wrote.
const INTENT_FLAGS: u8 = INTENT_START | INTENT_RESTART;

/// The plot byte on a frame that is not building anything.
///
/// A sentinel rather than a third flag bit, and it is safe to be one because
/// [`PLOTS`] is five rows long — `the_no_plot_sentinel_is_not_a_plot` asserts
/// that it never becomes a plot index.
const PLOT_NONE: u8 = u8::MAX;

/// How many bytes one sealed command is: a flag byte and a plot byte.
const INTENT_BYTES: usize = 2;

impl Intent {
    /// The wire form handed to `Client::set_input`.
    fn to_wire(self) -> Vec<u8> {
        let mut flags = 0;
        if self.start_wave {
            flags |= INTENT_START;
        }
        if self.restart {
            flags |= INTENT_RESTART;
        }
        vec![flags, self.place.unwrap_or(PLOT_NONE)]
    }

    /// The command a client sealed, read back on the server's side of the wire.
    ///
    /// `None` for anything this build did not write: a payload of the wrong
    /// length, or a flag outside [`INTENT_FLAGS`]. **The plot byte is not
    /// checked here**, and that is deliberate — a plot number is a thing the
    /// *rules* refuse rather than a thing the format cannot express, so it
    /// travels intact and [`Stage::place_tower`] turns it down. That is where
    /// the refusal is counted, and where a co-op build would report it to the
    /// player who asked.
    fn from_wire(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INTENT_BYTES {
            return None;
        }
        let flags = bytes[0];
        if flags & !INTENT_FLAGS != 0 {
            return None;
        }
        Some(Self {
            place: (bytes[1] != PLOT_NONE).then_some(bytes[1]),
            start_wave: flags & INTENT_START != 0,
            restart: flags & INTENT_RESTART != 0,
        })
    }

    /// Everything that arrived for this tick, folded into one.
    ///
    /// Normally one frame per tick and this is a decode. Several is a client
    /// whose clock ran ahead of the server's: the flags are OR-ed, because each
    /// is a thing the player asked for and a later frame that says nothing is
    /// not a retraction, and the **last** plot named wins — a player who
    /// pressed build twice inside one tick asked for the second plot, and
    /// merging two builds into one tick would drop a command rather than a
    /// keystroke.
    fn from_inputs(inputs: ClientInputs<'_>) -> Self {
        let mut merged = Self::default();
        for (_tick, data) in inputs.iter() {
            // A frame this build cannot read is skipped rather than taken as an
            // empty command, which would read as the player asking for nothing.
            let Some(frame) = Self::from_wire(data) else {
                continue;
            };
            merged.start_wave |= frame.start_wave;
            merged.restart |= frame.restart;
            if frame.place.is_some() {
                merged.place = frame.place;
            }
        }
        merged
    }
}

// ---------------------------------------------------------------------------
// The stage
// ---------------------------------------------------------------------------

/// Everything this sample simulates.
///
/// Behind an `Arc<Mutex<_>>` shared with [`TowersModule`], for the reason
/// `apps/orbit` gives: the module is what the server ticks and the frame is
/// what reads the result, and the two are not the same call stack.
struct Stage {
    world: PhysicsWorld,
    /// The exit volume's id, which is what an overlap's answer is compared
    /// against — see [`crate::creep::has_reached_the_exit`].
    exit: ColliderId,
    creeps: Vec<Creep>,
    towers: Vec<Tower>,
    bolts: Vec<Bolt>,
    waves: Waves,
    /// The team's shared purse and the team's shared lives — one of each,
    /// because co-op is what this sample is for.
    gold: u32,
    lives: u32,
    kills: u64,
    leaks: u64,
    shots: u64,
    built: u64,
    /// How many commands the server turned down. **The observable that says
    /// validation happens at all**: a build that trusted its client leaves this
    /// at zero while towers appear on occupied plots.
    refused: u64,
    outcome: Outcome,
    /// When the run ended, in [`Stage::elapsed`] seconds. Only read once it
    /// has.
    ended_at: f64,
    /// How many runs this stage has played, restarts included. The one number
    /// that survives a restart.
    runs: u64,
    ticks: u64,
    /// Seconds of **simulated** time, accumulated a tick at a time. What every
    /// clock in here is measured against, so a paused demo's waves stay where
    /// they are.
    elapsed: f64,
    /// The overlap queries' output buffer, hoisted so a tick that asks one
    /// question per creep and one per tower allocates nothing.
    scratch: Vec<ColliderId>,
}

impl Stage {
    /// An empty field with the first build phase running.
    fn new() -> Self {
        let (world, exit) = map::world();
        Self {
            world,
            exit,
            creeps: Vec::new(),
            towers: Vec::new(),
            bolts: Vec::new(),
            waves: Waves::new(),
            gold: STARTING_GOLD,
            lives: STARTING_LIVES,
            kills: 0,
            leaks: 0,
            shots: 0,
            built: 0,
            refused: 0,
            outcome: Outcome::Playing,
            ended_at: 0.0,
            runs: 1,
            ticks: 0,
            elapsed: 0.0,
            scratch: Vec::new(),
        }
    }

    /// Throws the run away and starts another one.
    ///
    /// Everything goes, the physics world included — a restart that kept the
    /// old world would keep every dead creep's collider in it. What survives is
    /// [`Stage::runs`], because a demo that has played itself four times should
    /// say so.
    fn reset(&mut self) {
        let runs = self.runs + 1;
        *self = Self::new();
        self.runs = runs;
    }

    /// Whether `plot` already has a tower on it.
    fn is_taken(&self, plot: usize) -> bool {
        self.towers.iter().any(|tower| tower.plot() == plot)
    }

    /// The server's half of the `PlaceTower` command: builds a tower, or turns
    /// the command down.
    ///
    /// Four ways to be refused, and each is a rule rather than a format
    /// problem: the run is over, the plot is not a plot, the plot is taken, or
    /// the purse is short. A client that predicted the build would have to
    /// predict all four.
    fn place_tower(&mut self, plot: u8) -> bool {
        let plot = plot as usize;
        if self.outcome.is_over()
            || plot >= PLOTS.len()
            || self.is_taken(plot)
            || self.gold < tower::COST
        {
            return false;
        }
        self.gold -= tower::COST;
        self.towers.push(Tower::new(plot));
        self.built += 1;
        true
    }
}

/// One tick of the simulation: a command in, and the six systems in the order
/// the module docs give.
fn run_tick(stage: &mut Stage, intent: Intent, dt: f64) {
    if intent.restart {
        stage.reset();
        return;
    }
    if let Some(plot) = intent.place
        && !stage.place_tower(plot)
    {
        stage.refused += 1;
    }
    if intent.start_wave && (stage.outcome.is_over() || !stage.waves.start_now(stage.elapsed)) {
        stage.refused += 1;
    }

    if stage.outcome.is_over() {
        // A finished run is left on screen and then played again — see
        // [`RESTART_S`]. Nothing else steps: the creeps that were walking went
        // with the life that ended it or with the wave that finished.
        if stage.elapsed - stage.ended_at >= RESTART_S {
            stage.reset();
            return;
        }
        stage.ticks += 1;
        stage.elapsed += dt;
        return;
    }

    // 1. The table releases, at most one creep a tick — see
    //    `crate::wave::Waves::step`.
    if let Some(wave) = stage.waves.step(stage.elapsed) {
        let creep = Creep::spawn(&mut stage.world, &wave);
        stage.creeps.push(creep);
    }

    // 2. Every creep walks, and writes its sphere where the walk left it.
    for creep in &mut stage.creeps {
        creep.advance(&mut stage.world, dt);
    }

    // 3. The exit volume takes what reached it. One overlap per creep, against
    //    the trigger `crate::map` registered.
    let mut leaked = 0_u32;
    {
        let Stage {
            world,
            creeps,
            exit,
            scratch,
            ..
        } = &mut *stage;
        let mut index = 0;
        while index < creeps.len() {
            if creep::has_reached_the_exit(world, &creeps[index], *exit, scratch) {
                creeps.swap_remove(index).despawn(world);
                leaked += 1;
            } else {
                index += 1;
            }
        }
    }
    if leaked > 0 {
        stage.leaks += u64::from(leaked);
        stage.lives = stage.lives.saturating_sub(leaked);
    }

    // 4. Every bolt flies, against the creeps as they are *now*.
    let mut index = 0;
    while index < stage.bolts.len() {
        let outcome = {
            let Stage {
                world,
                creeps,
                bolts,
                ..
            } = &mut *stage;
            bolts[index].step(world, creeps, dt)
        };
        match outcome {
            BoltOutcome::Flying => index += 1,
            BoltOutcome::Spent => {
                stage.bolts.swap_remove(index);
            }
            BoltOutcome::Hit(body) => {
                let damage = stage.bolts.swap_remove(index).damage();
                if let Some(hit) = stage.creeps.iter().position(|creep| creep.body() == body)
                    && stage.creeps[hit].wounded(damage)
                {
                    let bounty = stage.creeps[hit].bounty();
                    stage.creeps.swap_remove(hit).despawn(&mut stage.world);
                    stage.gold += bounty;
                    stage.kills += 1;
                }
            }
        }
    }

    // 5. Every ready tower acquires and fires.
    let now = stage.elapsed;
    for index in 0..stage.towers.len() {
        if !stage.towers[index].is_ready(now) {
            continue;
        }
        let muzzle = stage.towers[index].muzzle();
        let target = {
            let Stage {
                world,
                creeps,
                scratch,
                ..
            } = &mut *stage;
            tower::acquire(world, creeps, muzzle, scratch)
        };
        if let Some(creep) = target {
            let bolt = Bolt::fire(muzzle, &stage.creeps[creep], tower::DAMAGE);
            stage.bolts.push(bolt);
            stage.towers[index].fired(now);
            stage.shots += 1;
        }
    }

    // 6. Win at the end of the table, lose at zero lives.
    if stage.lives == 0 {
        stage.outcome = Outcome::Lost;
        stage.ended_at = stage.elapsed;
    } else if stage.waves.is_exhausted() && stage.creeps.is_empty() {
        stage.outcome = Outcome::Won;
        stage.ended_at = stage.elapsed;
    }

    stage.ticks += 1;
    stage.elapsed += dt;
}

// ---------------------------------------------------------------------------
// The module
// ---------------------------------------------------------------------------

/// The stage, as the server hosts it.
///
/// `register` is empty for the same reason `apps/breach`'s is: the whole
/// simulation is the [`Stage`] behind the shared cell, and there is no ECS
/// system to register.
struct TowersModule {
    shared: Arc<Mutex<Stage>>,
}

impl std::fmt::Debug for TowersModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TowersModule").finish_non_exhaustive()
    }
}

impl GameModule for TowersModule {
    fn name(&self) -> &str {
        "towers"
    }

    fn register(&self, _world: &mut World) {}

    fn tick(&mut self, world: &mut World, inputs: ClientInputs<'_>) {
        let dt = world.tick_dt();
        let mut stage = lock(&self.shared);
        run_tick(&mut stage, Intent::from_inputs(inputs), dt);
    }
}

/// The shared stage, with a poisoned lock treated as the stage it was left in.
///
/// A panic inside the tick is a bug this sample would rather report through its
/// own numbers than through a second panic in the frame that reads them.
fn lock(shared: &Arc<Mutex<Stage>>) -> MutexGuard<'_, Stage> {
    shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

// ---------------------------------------------------------------------------
// What a frame reads
// ---------------------------------------------------------------------------

/// Everything the frame draws, snapshotted once per draw.
///
/// A plain `Copy` struct rather than a borrow of the stage: the frame runs on
/// the frame's thread and the stage is behind a mutex the tick holds, and a
/// frame that read through the lock would be holding it for the length of a
/// draw. The pools are fixed-size for the same reason — a heap allocation here
/// would be one per draw.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderState {
    /// Every creep on the field, the first [`RenderState::creeps_alive`] of
    /// them live.
    pub creeps: [CreepView; MAX_CREEPS],
    pub creeps_alive: usize,
    /// One entry per plot: `None` for an empty plot, `Some(firing)` for a built
    /// tower.
    pub towers: [Option<bool>; PLOTS.len()],
    /// Every bolt in the air, the first [`RenderState::bolts_flying`] of them
    /// live. Bolts past the pool are simulated and not drawn — see
    /// [`crate::map::MAX_BOLTS`].
    pub bolts: [DVec3; MAX_BOLTS],
    pub bolts_flying: usize,
    pub gold: u32,
    pub lives: u32,
    /// How many waves have been started, out of [`crate::wave::WAVES`].
    pub wave: usize,
    pub kills: u64,
    pub leaks: u64,
    pub outcome: Outcome,
    /// How long until the next wave starts, in seconds, or `None` while one is
    /// releasing or the table is spent.
    pub next_wave_in: Option<f64>,
}

/// The stage's numbers, for the debug overlay and the `[HUD]` line.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stats {
    pub ticks: u64,
    pub gold: u32,
    pub lives: u32,
    pub wave: usize,
    pub creeps: usize,
    pub towers: usize,
    pub bolts: usize,
    pub kills: u64,
    pub leaks: u64,
    pub shots: u64,
    pub built: u64,
    /// How many commands the server turned down — see `Stage::refused`.
    pub refused: u64,
    pub outcome: Outcome,
    pub runs: u64,
    pub next_wave_in: Option<f64>,
}

impl crcbl::ui::DebugModule for Stats {
    fn debug_section(&self, section: &mut crcbl::ui::DebugSection) {
        section.set_title("towers");
        section.row("tick", format_args!("{}", self.ticks));
        section.row("gold", format_args!("{}", self.gold));
        section.row(
            "lives",
            format_args!("{}/{}", self.lives, crate::wave::STARTING_LIVES),
        );
        section.row(
            "wave",
            format_args!("{}/{}", self.wave, crate::wave::WAVES.len()),
        );
        match self.next_wave_in {
            Some(seconds) => section.row("next", format_args!("{seconds:.1} s")),
            None => section.row_str("next", "--"),
        }
        section.row("creeps", format_args!("{}", self.creeps));
        section.row("towers", format_args!("{}/{}", self.towers, PLOTS.len()));
        section.row("bolts", format_args!("{}", self.bolts));
        section.row("kills", format_args!("{}", self.kills));
        section.row("leaks", format_args!("{}", self.leaks));
        section.row("shots", format_args!("{}", self.shots));
        section.row("built", format_args!("{}", self.built));
        // The one row that says the server is validating rather than obeying.
        section.row("refused", format_args!("{}", self.refused));
        section.row_str("outcome", self.outcome.label());
        section.row("runs", format_args!("{}", self.runs));
    }
}

// ---------------------------------------------------------------------------
// The facade
// ---------------------------------------------------------------------------

/// What can stop towers before it starts.
#[derive(Debug)]
pub enum GameError {
    /// The operating system would not seed the server's resume credential.
    Server(String),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Server(message) => write!(f, "server creation failed: {message}"),
        }
    }
}

impl std::error::Error for GameError {}

/// The stage, its server, its client, and the clock that drives all three.
pub struct Game {
    session: Loopback,
    shared: Arc<Mutex<Stage>>,
    /// Exactly one tick period per [`Game::tick`], so the server's accumulator
    /// yields exactly one tick per call.
    tick_period: Duration,
    sim_time: Duration,
    ticks_run: u64,
    /// What the player asked for, sent on the next tick.
    pending: Intent,
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("ticks_run", &self.ticks_run)
            .finish_non_exhaustive()
    }
}

impl Game {
    /// Builds the server, its client and the stage between them.
    ///
    /// # Errors
    ///
    /// [`GameError::Server`] if the operating system would not give the server
    /// the entropy for a resume credential, or if the loopback session did not
    /// come up.
    ///
    /// # Panics
    ///
    /// If `tick_hz` is zero.
    pub fn new(tick_hz: u32) -> Result<Self, GameError> {
        assert!(tick_hz > 0, "tick rate must be positive");
        let shared = Arc::new(Mutex::new(Stage::new()));

        // An empty world, and that is the honest shape: this sample has no
        // entity and no ECS system. What the server hosts is the module, and
        // what the module owns is the stage.
        let session = Loopback::new(
            World::new(),
            Box::new(TowersModule {
                shared: Arc::clone(&shared),
            }),
            tick_hz,
            COMPATIBILITY,
        )
        .map_err(|error| GameError::Server(error.to_string()))?;

        let tick_period = session.tick_period();
        let mut game = Self {
            session,
            shared,
            tick_period,
            sim_time: Duration::ZERO,
            ticks_run: 0,
            pending: Intent::default(),
        };

        // **One tick spent on the handshake, before the first command.**
        // `Server::update` drains the transport inside `tick`, so the client's
        // hello is not read until a tick runs, and until the session is up the
        // client drops every input frame it is asked to send. Spending it here
        // is what makes the player's first key the first the simulation sees.
        game.sim_time = tick_period;
        game.session.client_mut().update(game.sim_time);
        game.session.server_mut().update(game.sim_time);
        game.session.client_mut().update(game.sim_time);
        if game.session.server().session_state() != crcbl::net::SessionState::Connected {
            return Err(GameError::Server(
                "the loopback session did not come up in its first tick".into(),
            ));
        }

        crcbl::log::info!(
            "sim: {tick_hz} Hz, {:.3} ms per tick, {} waves of up to {MAX_CREEPS} creeps over a \
             {:.1} m path, {} lives and {} gold",
            tick_period.as_secs_f64() * 1e3,
            wave::WAVES.len(),
            crate::path::length(),
            STARTING_LIVES,
            STARTING_GOLD,
        );
        Ok(game)
    }

    /// Records what the player asked for, to be sent on the next tick.
    pub fn set_controls(&mut self, controls: Controls) {
        self.pending = Intent {
            place: controls.place,
            start_wave: controls.start_wave,
            restart: controls.restart,
        };
    }

    /// Advances the server, and with it the stage, by exactly one tick.
    pub fn tick(&mut self) {
        self.sim_time += self.tick_period;
        let (server, client) = self.session.both_mut();

        // The bytes are the whole command path: the client seals them, the
        // transport carries them and the module decodes them, exactly as a
        // remote client's would be.
        client.set_input(core::mem::take(&mut self.pending).to_wire());

        // Send, simulate, then receive — and the send has to come first.
        // `Client::update` is the only thing that puts input on the wire and
        // the server drains the wire at the top of its tick, so a client
        // updated only after the server posts this tick's commands to the next
        // one.
        client.update(self.sim_time);
        let server_ticks = server.update(self.sim_time);
        debug_assert_eq!(
            server_ticks, 1,
            "one tick period in must be exactly one server tick out",
        );
        // Consumes no tick — the clock has not moved between the two — and is
        // there to take the snapshot this tick produced.
        client.update(self.sim_time);
        self.ticks_run += 1;
    }

    /// How many times [`Game::tick`] has been called.
    #[must_use]
    pub const fn ticks_run(&self) -> u64 {
        self.ticks_run
    }

    /// What the frame should draw.
    #[must_use]
    pub fn render_state(&self) -> RenderState {
        let stage = lock(&self.shared);
        let mut creeps = [CreepView::default(); MAX_CREEPS];
        for (slot, creep) in creeps.iter_mut().zip(stage.creeps.iter()) {
            *slot = creep.view();
        }
        let mut bolts = [DVec3::ZERO; MAX_BOLTS];
        for (slot, bolt) in bolts.iter_mut().zip(stage.bolts.iter()) {
            *slot = bolt.at();
        }
        let now = stage.elapsed;
        RenderState {
            creeps,
            creeps_alive: stage.creeps.len().min(MAX_CREEPS),
            towers: core::array::from_fn(|plot| {
                stage
                    .towers
                    .iter()
                    .find(|tower| tower.plot() == plot)
                    .map(|tower| tower.is_firing(now))
            }),
            bolts,
            bolts_flying: stage.bolts.len().min(MAX_BOLTS),
            gold: stage.gold,
            lives: stage.lives,
            wave: stage.waves.started(),
            kills: stage.kills,
            leaks: stage.leaks,
            outcome: stage.outcome,
            next_wave_in: stage.waves.next_in(now),
        }
    }

    /// The stage's numbers for the debug panel and the `[HUD]` line.
    #[must_use]
    pub fn stats(&self) -> Stats {
        let stage = lock(&self.shared);
        Stats {
            ticks: stage.ticks,
            gold: stage.gold,
            lives: stage.lives,
            wave: stage.waves.started(),
            creeps: stage.creeps.len(),
            towers: stage.towers.len(),
            bolts: stage.bolts.len(),
            kills: stage.kills,
            leaks: stage.leaks,
            shots: stage.shots,
            built: stage.built,
            refused: stage.refused,
            outcome: stage.outcome,
            runs: stage.runs,
            next_wave_in: stage.waves.next_in(stage.elapsed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wave::WAVES;

    /// One tick at the default rate.
    const DT: f64 = 1.0 / DEFAULT_TICK_HZ as f64;

    /// Runs the stage for `seconds`, asking for nothing.
    fn idle(stage: &mut Stage, seconds: f64) {
        for _ in 0..(seconds / DT).round() as u64 {
            run_tick(stage, Intent::default(), DT);
        }
    }

    /// One command, on its own tick.
    fn command(stage: &mut Stage, intent: Intent) {
        run_tick(stage, intent, DT);
    }

    /// Runs the stage until the run ends, for at most `seconds`.
    ///
    /// A run rather than a fixed number of ticks, because a finished run
    /// **starts itself again** — see [`RESTART_S`] — so a test that idled for a
    /// round number of seconds would be asking about whichever run happened to
    /// be in progress when it stopped counting.
    fn until_over(stage: &mut Stage, seconds: f64) -> Outcome {
        for _ in 0..(seconds / DT).round() as u64 {
            run_tick(stage, Intent::default(), DT);
            if stage.outcome.is_over() {
                return stage.outcome;
            }
        }
        stage.outcome
    }

    /// The command that builds on `plot`.
    const fn build(plot: u8) -> Intent {
        Intent {
            place: Some(plot),
            start_wave: false,
            restart: false,
        }
    }

    /// **A command survives the wire, and a frame this build did not write is
    /// refused rather than read as an empty command.**
    ///
    /// The refusal matters because an unreadable frame taken as `default()`
    /// would be indistinguishable from a player asking for nothing, and the
    /// merge below would then treat it as one.
    #[test]
    fn a_command_survives_the_wire_and_nonsense_does_not() {
        for intent in [
            Intent::default(),
            Intent {
                place: Some(3),
                start_wave: true,
                restart: false,
            },
            Intent {
                place: None,
                start_wave: false,
                restart: true,
            },
        ] {
            let wire = intent.to_wire();
            assert_eq!(wire.len(), INTENT_BYTES);
            assert_eq!(Intent::from_wire(&wire), Some(intent));
        }

        assert_eq!(Intent::from_wire(&[]), None, "an empty frame was read");
        assert_eq!(Intent::from_wire(&[0, 0, 0]), None, "a long frame was read");
        assert_eq!(
            Intent::from_wire(&[0b1000_0000, PLOT_NONE]),
            None,
            "a flag this build never sets was read",
        );
    }

    /// **The sentinel is not a plot**, which is what lets one byte carry both
    /// "build here" and "build nothing".
    #[test]
    fn the_no_plot_sentinel_is_not_a_plot() {
        assert!(
            (PLOT_NONE as usize) >= PLOTS.len(),
            "the sentinel names plot {PLOT_NONE}",
        );
    }

    /// **A tower is built only when the plot is free, the plot is a plot and
    /// the gold is there** — and every refusal is counted.
    ///
    /// Four refusals and one success, because the four are what the server is
    /// for: a build that trusted its client passes the success and leaves
    /// `refused` at zero.
    #[test]
    fn a_tower_is_built_only_when_the_rules_allow_it() {
        let mut stage = Stage::new();
        command(&mut stage, build(0));
        assert_eq!(stage.towers.len(), 1, "the first build was refused");
        assert_eq!(stage.gold, STARTING_GOLD - tower::COST);
        assert_eq!(stage.refused, 0);

        // The same plot again.
        command(&mut stage, build(0));
        assert_eq!(stage.towers.len(), 1, "it built twice on one plot");
        assert_eq!(stage.refused, 1);

        // A plot that is not a plot — the byte the wire carried intact.
        command(&mut stage, build(200));
        assert_eq!(stage.towers.len(), 1, "it built on plot 200");
        assert_eq!(stage.refused, 2);

        // Spend down to nothing, then ask again.
        let affordable = stage.gold / tower::COST;
        for plot in 1..=affordable as u8 {
            command(&mut stage, build(plot));
        }
        assert!(stage.gold < tower::COST, "the purse is not empty");
        let built = stage.towers.len();
        command(&mut stage, build(built as u8));
        assert_eq!(stage.towers.len(), built, "it built with no gold");
        assert_eq!(stage.refused, 3);
    }

    /// **A creep that reaches the exit costs a life**, and the field is left
    /// without it.
    #[test]
    fn a_creep_that_reaches_the_exit_costs_a_life() {
        let mut stage = Stage::new();
        command(
            &mut stage,
            Intent {
                place: None,
                start_wave: true,
                restart: false,
            },
        );
        assert_eq!(stage.refused, 0, "the first wave refused to start");

        // Long enough for the whole first wave to walk the path and no longer:
        // the second wave's own creeps must not be what this counts.
        let walk = crate::path::length() / WAVES[0].speed;
        idle(
            &mut stage,
            walk + f64::from(WAVES[0].creeps) * WAVES[0].spacing_s + 1.0,
        );
        assert_eq!(
            stage.leaks,
            u64::from(WAVES[0].creeps),
            "an empty field let {} of {} through",
            stage.leaks,
            WAVES[0].creeps,
        );
        assert_eq!(stage.lives, STARTING_LIVES - WAVES[0].creeps);
        assert_eq!(stage.kills, 0, "an empty field killed something");
    }

    /// **A field with no towers on it loses the run**, which is what makes
    /// building the game. The table releases more creeps than the team has
    /// lives — `crate::wave` asserts that inequality — and this is the outcome
    /// it produces.
    #[test]
    fn a_field_with_no_towers_on_it_loses_the_run() {
        let mut stage = Stage::new();
        // Well past the whole table plus the walk, so a run that never ends
        // fails here rather than running for ever.
        assert_eq!(
            until_over(&mut stage, 120.0),
            Outcome::Lost,
            "an empty field survived: {} leaks and {} lives left",
            stage.leaks,
            stage.lives,
        );
        assert_eq!(stage.lives, 0);
        assert_eq!(stage.kills, 0, "an empty field killed something");
        assert_eq!(stage.leaks, u64::from(STARTING_LIVES));
        assert!(
            stage.waves.started() < wave::WAVES.len() || !stage.creeps.is_empty(),
            "it lost only once the table was spent, so the lives outlasted the waves",
        );
    }

    /// **Towers clear the table and win the run**, and the kills pay for
    /// themselves: the run starts with three towers' worth of gold and buys the
    /// other two out of bounties.
    ///
    /// The build command is issued every tick and refused until the purse
    /// catches up, which is the same validation
    /// `a_tower_is_built_only_when_the_rules_allow_it` asserts, driven by the
    /// game rather than by a test.
    #[test]
    fn towers_clear_the_table_and_win_the_run() {
        let mut stage = Stage::new();
        let mut plot = 0_u8;
        let mut ticks = 0_u64;
        // The most bolts that were ever in the air at once, which is what
        // `crate::map::MAX_BOLTS` has to cover — see that constant.
        let mut peak_bolts = 0;
        // Well past the table's own length, so a run that stalls fails here
        // rather than running for ever.
        while ticks < (150.0 / DT) as u64 && stage.outcome == Outcome::Playing {
            let intent = if (plot as usize) < PLOTS.len() {
                build(plot)
            } else {
                Intent::default()
            };
            run_tick(&mut stage, intent, DT);
            peak_bolts = peak_bolts.max(stage.bolts.len());
            if stage.towers.len() > plot as usize {
                plot += 1;
            }
            ticks += 1;
        }
        assert_eq!(
            stage.outcome,
            Outcome::Won,
            "five towers lost the table: {} kills, {} leaks, {} lives left",
            stage.kills,
            stage.leaks,
            stage.lives,
        );
        assert_eq!(stage.towers.len(), PLOTS.len(), "not every plot was built");
        assert!(
            stage.built as u32 * tower::COST > STARTING_GOLD,
            "the opening purse paid for every tower, so nothing was earned",
        );
        assert_eq!(
            stage.kills,
            MAX_CREEPS as u64 - stage.leaks,
            "the kills and the leaks do not account for every creep",
        );
        // **The draw pool covers what a full field puts in the air.** Measured
        // over the whole run rather than argued from the reload, because what
        // decides it is how long a bolt lives, and that is the map's geometry.
        assert!(
            peak_bolts > 0 && peak_bolts <= map::MAX_BOLTS,
            "{peak_bolts} bolts were in the air at once, against a pool of {}",
            map::MAX_BOLTS,
        );
    }

    /// **A finished run plays itself again**, so a demo nobody is watching is
    /// never a still picture — see [`RESTART_S`].
    #[test]
    fn a_finished_run_starts_itself_again() {
        let mut stage = Stage::new();
        assert_eq!(until_over(&mut stage, 120.0), Outcome::Lost);
        assert_eq!(stage.runs, 1);

        idle(&mut stage, RESTART_S + 1.0);
        assert_eq!(stage.outcome, Outcome::Playing, "the run never restarted");
        assert_eq!(stage.runs, 2, "the restart was not counted");
        assert_eq!(stage.lives, STARTING_LIVES, "the lives did not come back");
        assert_eq!(stage.gold, STARTING_GOLD);
        assert_eq!(stage.waves.started(), 0);
    }

    /// **`R` throws the run away at once**, without waiting for it to end.
    #[test]
    fn the_restart_command_starts_the_run_over() {
        let mut stage = Stage::new();
        command(&mut stage, build(0));
        idle(&mut stage, 5.0);
        assert!(stage.waves.started() > 0 && stage.towers.len() == 1);

        command(
            &mut stage,
            Intent {
                place: None,
                start_wave: false,
                restart: true,
            },
        );
        assert!(stage.towers.is_empty(), "the towers survived the restart");
        assert!(stage.creeps.is_empty(), "the creeps survived the restart");
        assert_eq!(stage.gold, STARTING_GOLD);
        assert_eq!(stage.runs, 2);
    }

    /// **A kill pays its wave's bounty**, which is the whole of the economy.
    #[test]
    fn a_kill_pays_its_bounty() {
        let mut stage = Stage::new();
        command(&mut stage, build(0));
        let purse = stage.gold;
        command(
            &mut stage,
            Intent {
                place: None,
                start_wave: true,
                restart: false,
            },
        );
        // One creep's worth of walking past one tower.
        idle(&mut stage, 6.0);
        assert!(stage.kills > 0, "one tower killed nothing in six seconds");
        assert_eq!(
            stage.gold,
            purse + stage.kills as u32 * WAVES[0].bounty,
            "the purse does not match {} kills at {} gold",
            stage.kills,
            WAVES[0].bounty,
        );
    }
}
