//! `WaveSystem` and `EconomySystem`: the scripted table, the clock that
//! releases it, and the two numbers a team shares.
//!
//! ```text
//!   idle ──(gap elapsed, or StartWave)──▶ releasing ──(table row spent)──▶ idle
//!     │                                                                     │
//!     └────────────────────(every row spent)──▶ exhausted ──▶ Outcome::Won ─┘
//! ```
//!
//! # A table, not a formula
//!
//! [`WAVES`] is written out row by row. A generated curve is what a game does
//! once it has been played enough to know what the curve should be, and this
//! sample has been played for one slice; a table is also the thing a reviewer
//! can read and disagree with. `docs/plan/sample/07-towers.md` asks for ten
//! waves and this is the first three of them, which is what makes the loop
//! finishable in a test rather than a soak.
//!
//! # Doing nothing loses, and that is arithmetic rather than a hope
//!
//! [`STARTING_LIVES`] is deliberately under [`MAX_CREEPS`], the number of
//! creeps the whole table releases — so a run that builds nothing leaks its way
//! to [`Outcome::Lost`] before the table is spent.
//! `a_team_that_builds_nothing_cannot_survive_the_table` asserts the
//! inequality here, and `crate::game`'s
//! `a_field_with_no_towers_on_it_loses_the_run` asserts the outcome.
//!
//! # `StartWave` is a real command with a real refusal
//!
//! [`Waves::start_now`] is what the client's `StartWave` intent reaches, and it
//! answers `false` for the two cases the server must refuse: a wave already
//! releasing, and a table already spent. Waves also start **on their own**
//! after [`GAP_S`], so a field nobody is playing still plays — the command
//! brings the next one forward rather than being the only way to see one.

/// One row of the scripted table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Wave {
    /// What the overlay and a failing test call it.
    pub label: &'static str,
    /// How many creeps it releases.
    pub creeps: u32,
    /// How long between one release and the next, in seconds.
    pub spacing_s: f64,
    /// What each of them has, in hit points.
    pub health: u32,
    /// How fast each of them walks, in metres a second.
    pub speed: f64,
    /// What killing one pays, in gold.
    pub bounty: u32,
}

/// The three waves, in the order they are released.
///
/// Each row moves one thing at a time — more creeps, then tougher ones, then
/// tougher and quicker — so a run that fails says which of the three it failed
/// on.
pub const WAVES: [Wave; 3] = [
    Wave {
        label: "first",
        creeps: 4,
        spacing_s: 1.0,
        health: 40,
        speed: 6.0,
        bounty: 12,
    },
    Wave {
        label: "second",
        creeps: 6,
        spacing_s: 0.9,
        health: 60,
        speed: 6.0,
        bounty: 14,
    },
    Wave {
        label: "third",
        creeps: 8,
        spacing_s: 0.8,
        health: 90,
        speed: 6.5,
        bounty: 16,
    },
];

/// How many creeps the whole table releases.
///
/// The size of `crate::map`'s creep instance pool, and therefore the number of
/// creeps that can be on the field at once — the gap between waves is measured
/// from the last **release** rather than from the field clearing, so a wave can
/// still be walking when the next one starts.
pub const MAX_CREEPS: usize = {
    let mut total = 0;
    let mut row = 0;
    while row < WAVES.len() {
        total += WAVES[row].creeps as usize;
        row += 1;
    }
    total
};

/// How long the build phase is, in seconds: before the first wave, and between
/// one wave's last release and the next wave's first.
pub const GAP_S: f64 = 2.5;

/// What a team starts with, in gold. Three towers' worth — see
/// [`crate::tower::COST`].
pub const STARTING_GOLD: u32 = 120;

/// How many creeps a team can let through before it loses.
pub const STARTING_LIVES: u32 = 12;

/// How the run ended, or that it has not.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Outcome {
    /// Still being played.
    #[default]
    Playing,
    /// Every wave cleared with lives to spare.
    Won,
    /// The last life went through the exit.
    Lost,
}

impl Outcome {
    /// What the overlay, the debug panel and the `[HUD]` line call it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Playing => "playing",
            Self::Won => "won",
            Self::Lost => "lost",
        }
    }

    /// Whether the run is over.
    #[must_use]
    pub const fn is_over(self) -> bool {
        !matches!(self, Self::Playing)
    }
}

/// Where the table has got to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Waves {
    /// How many waves have been started, `0..=WAVES.len()`.
    started: usize,
    /// How many creeps the wave in progress has left to release.
    remaining: u32,
    /// When the next release — or, while idle, the next wave — is due, in the
    /// stage's elapsed seconds.
    due_at: f64,
    /// Whether a wave is releasing creeps.
    releasing: bool,
}

impl Default for Waves {
    fn default() -> Self {
        Self::new()
    }
}

impl Waves {
    /// A table nothing has been released from, with the first build phase
    /// already running.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            started: 0,
            remaining: 0,
            due_at: GAP_S,
            releasing: false,
        }
    }

    /// How many waves have been started.
    #[must_use]
    pub const fn started(&self) -> usize {
        self.started
    }

    /// Whether a wave is releasing creeps this instant.
    #[must_use]
    pub const fn is_releasing(&self) -> bool {
        self.releasing
    }

    /// Whether every row has been released.
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.started == WAVES.len() && !self.releasing
    }

    /// How long until the next wave starts, in seconds, or `None` while one is
    /// releasing or the table is spent.
    #[must_use]
    pub fn next_in(&self, now: f64) -> Option<f64> {
        (!self.releasing && self.started < WAVES.len()).then(|| (self.due_at - now).max(0.0))
    }

    /// Brings the next wave forward to `now`. Answers whether it was allowed.
    ///
    /// **The server's half of the `StartWave` command.** Refused while a wave
    /// is releasing — there is no "next" to bring forward — and once the table
    /// is spent. `crate::game` counts a refusal, so a rejected command is
    /// something a run reports rather than something it swallows.
    pub fn start_now(&mut self, now: f64) -> bool {
        if self.releasing || self.started >= WAVES.len() {
            return false;
        }
        self.due_at = now;
        true
    }

    /// The wave a creep released on this tick belongs to, or `None` on a tick
    /// that releases none.
    ///
    /// At most one creep a tick: the shortest [`Wave::spacing_s`] in the table
    /// is far longer than a tick, which
    /// `the_table_never_asks_for_two_creeps_in_one_tick` asserts against the
    /// simulation rate.
    pub fn step(&mut self, now: f64) -> Option<Wave> {
        if !self.releasing {
            if self.started >= WAVES.len() || now < self.due_at {
                return None;
            }
            self.started += 1;
            self.remaining = WAVES[self.started - 1].creeps;
            self.releasing = true;
        }
        if now < self.due_at {
            return None;
        }
        let wave = WAVES[self.started - 1];
        self.remaining -= 1;
        if self.remaining == 0 {
            self.releasing = false;
            self.due_at = now + GAP_S;
        } else {
            self.due_at = now + wave.spacing_s;
        }
        Some(wave)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One tick at the sample's own rate.
    const DT: f64 = 1.0 / crate::game::DEFAULT_TICK_HZ as f64;

    /// Runs the table forward for `seconds`, answering every creep it released.
    fn released(waves: &mut Waves, seconds: f64) -> Vec<(f64, Wave)> {
        let mut out = Vec::new();
        let mut now = 0.0;
        while now < seconds {
            now += DT;
            if let Some(wave) = waves.step(now) {
                out.push((now, wave));
            }
        }
        out
    }

    /// **A table nobody touches releases exactly what it says**, in order, at
    /// the spacing each row asks for, and then stops.
    ///
    /// The "and then stops" is the half worth having: a schedule that wrapped
    /// round would look identical for the first three waves and never let a run
    /// be won.
    #[test]
    fn the_table_releases_every_row_once_and_then_stops() {
        let mut waves = Waves::new();
        // Long enough for every gap and every row, with room to spare.
        let all = released(&mut waves, 120.0);
        assert_eq!(
            all.len(),
            MAX_CREEPS,
            "the table released {} creeps of {MAX_CREEPS}",
            all.len(),
        );
        assert!(waves.is_exhausted(), "the table is not spent");
        assert_eq!(waves.started(), WAVES.len());

        let mut seen = 0;
        for (row, wave) in WAVES.iter().enumerate() {
            for index in 0..wave.creeps as usize {
                let (at, released) = all[seen + index];
                assert_eq!(&released, wave, "creep {seen} came from the wrong row");
                if index > 0 {
                    let gap = at - all[seen + index - 1].0;
                    assert!(
                        (gap - wave.spacing_s).abs() < 2.0 * DT,
                        "wave {row} released two creeps {gap:.3} s apart, not {}",
                        wave.spacing_s,
                    );
                }
            }
            seen += wave.creeps as usize;
        }

        // The first wave waits out the build phase, and the second waits out
        // another one after the first row is spent.
        assert!(
            (all[0].0 - GAP_S).abs() < 2.0 * DT,
            "the first creep arrived at {:.3} s, not after the {GAP_S} s build phase",
            all[0].0,
        );
        let first_row = WAVES[0].creeps as usize;
        let between = all[first_row].0 - all[first_row - 1].0;
        assert!(
            (between - GAP_S).abs() < 2.0 * DT,
            "the second wave started {between:.3} s after the first ended, not {GAP_S} s",
        );
    }

    /// **`StartWave` brings the next wave forward, and is refused when there is
    /// no next wave to bring.** The refusal is the half the server exists for:
    /// a command accepted while a wave is already releasing would start the
    /// same row twice.
    #[test]
    fn start_wave_is_accepted_when_idle_and_refused_otherwise() {
        let mut waves = Waves::new();
        assert!(waves.start_now(0.0), "an idle table refused the command");
        assert!(
            waves.step(0.0).is_some(),
            "the wave it brought forward did not start",
        );
        assert!(waves.is_releasing());
        assert!(
            !waves.start_now(0.0),
            "a releasing table accepted a second start",
        );

        // Spend the table, then ask again.
        let mut waves = Waves::new();
        released(&mut waves, 120.0);
        assert!(waves.is_exhausted());
        assert!(
            !waves.start_now(200.0),
            "a spent table accepted another wave",
        );
        assert_eq!(waves.started(), WAVES.len(), "it started a fourth wave");
    }

    /// **The command is worth issuing**: a run that starts every wave the
    /// instant it may finishes the table sooner than one that waits out every
    /// build phase. Without this, `start_now` could return `true` and change
    /// nothing.
    #[test]
    fn starting_waves_early_finishes_the_table_sooner() {
        let mut patient = Waves::new();
        let waited = released(&mut patient, 120.0);

        let mut eager = Waves::new();
        let mut hurried = Vec::new();
        let mut now = 0.0;
        while now < 120.0 {
            now += DT;
            eager.start_now(now);
            if let Some(wave) = eager.step(now) {
                hurried.push((now, wave));
            }
        }
        assert_eq!(hurried.len(), waited.len(), "a different number of creeps");
        let saved = waited[waited.len() - 1].0 - hurried[hurried.len() - 1].0;
        assert!(
            saved > 2.0 * GAP_S - 3.0 * DT,
            "starting early saved {saved:.2} s over {} build phases",
            WAVES.len() - 1,
        );
    }

    /// **Every row is at least as hard as the one before it**, which is what a
    /// scripted table is for — and the pool is sized to the whole of it.
    #[test]
    fn the_table_gets_harder_and_the_pool_holds_all_of_it() {
        for pair in WAVES.windows(2) {
            let (before, after) = (pair[0], pair[1]);
            assert!(
                after.creeps >= before.creeps
                    && after.health >= before.health
                    && after.speed >= before.speed
                    && after.bounty >= before.bounty,
                "the {} wave is easier than the {} one",
                after.label,
                before.label,
            );
        }
        let total: u32 = WAVES.iter().map(|wave| wave.creeps).sum();
        assert_eq!(MAX_CREEPS, total as usize);
    }

    /// **The table never asks for two creeps in one tick**, which is what lets
    /// [`Waves::step`] release at most one per call.
    #[test]
    fn the_table_never_asks_for_two_creeps_in_one_tick() {
        for wave in WAVES {
            assert!(
                wave.spacing_s > DT,
                "the {} wave releases every {} s, inside one {DT} s tick",
                wave.label,
                wave.spacing_s,
            );
        }
    }

    /// **A team that builds nothing cannot survive the table**, which is what
    /// makes building the game rather than the decoration. The outcome is
    /// asserted in `crate::game`; this is the arithmetic it rests on.
    #[test]
    fn a_team_that_builds_nothing_cannot_survive_the_table() {
        assert!(
            (STARTING_LIVES as usize) < MAX_CREEPS,
            "{STARTING_LIVES} lives outlast the {MAX_CREEPS} creeps the table releases",
        );
        assert_eq!(
            STARTING_GOLD / crate::tower::COST,
            3,
            "the opening gold is not three towers",
        );
    }
}
