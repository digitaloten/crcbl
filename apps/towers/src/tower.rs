//! `TowerSystem` and `ProjectileSystem`: acquisition by sphere overlap, and a
//! swept bolt against a creep that is moving in the same tick.
//!
//! ```text
//!   muzzle ──▶ PhysicsWorld::overlap_sphere ──▶ ids ──filter──▶ the creep
//!                                                     nearest the exit
//!                                                          │
//!   Bolt ── Segment(at, at + heading × speed × dt) ──▶ sweep_sphere ──▶ damage
//! ```
//!
//! # Both halves are `crcbl-phys` queries, and rule 9 has no exemption
//!
//! Nothing in this file works out whether two things are touching. Acquisition
//! is [`PhysicsWorld::overlap_sphere`] and a hit is
//! [`PhysicsWorld::sweep_sphere`]; what this module decides is **where from**,
//! **which way** and **what an answer means**.
//!
//! # An overlap answers with everything, which is the point of the filter
//!
//! [`PhysicsWorld::overlap_sphere`] reports triggers as well as solids — that is
//! the query they exist for — and the ground slab is inside every tower's
//! range. So [`acquire`] hands back a creep index or nothing, and a build that
//! took the query's first answer would have every tower shooting at the floor.
//! `apps/horde` carries the same note about the same query, and
//! `a_tower_ignores_everything_in_range_that_is_not_a_creep` asserts the
//! overlap really does return the other things.
//!
//! # The sweep is the whole reason a bolt is a bolt
//!
//! [`BOLT_SPEED`] is fast enough that one tick's travel is longer than a creep
//! is wide: a test that asked "is the bolt inside a creep?" at the start of the
//! tick and again at the end would answer no both times, on a tick the bolt
//! passed clean through one. `docs/plan/05-physics.md`'s CCD slice is what
//! towers drives, and
//! `a_bolt_hits_a_creep_that_a_test_at_either_end_of_the_tick_would_miss` is
//! that claim made against a creep that is **also moving** — the creep's sphere
//! is written by [`crate::creep::Creep::advance`] earlier in the same tick, so
//! the sweep is against where the target is now rather than where it was.
//!
//! # A bolt homes, and that is a design decision rather than a shortcut
//!
//! A tower defense's single-target tower hits what it shot at; leading a target
//! is a skill a player has and a tower does not. So [`Bolt::step`] turns toward
//! its target's current centre every tick.
//!
//! **A bolt whose target died before it landed keeps its last heading and stops
//! in the ground**, which is [`BoltOutcome::Spent`]. There is no separate
//! lifetime on a bolt and it does not need one: [`MUZZLE_Y`] stands above a
//! creep's centre, so every bolt this sample fires is descending and the ground
//! slab is always in front of it. A timer beside that would be a guard nothing
//! could ever trip.

use crcbl::math::DVec3;
use crcbl::phys::{ColliderId, PhysicsWorld, Segment};

use crate::creep::Creep;
use crate::map::{BOLT_RADIUS, MUZZLE_Y, PLOTS};

/// How far a tower reaches, in metres, measured from its muzzle.
pub const RANGE_M: f64 = 7.0;

/// What one bolt takes off a creep, in hit points.
pub const DAMAGE: u32 = 18;

/// How long between one shot and the next, in seconds.
pub const RELOAD_S: f64 = 0.35;

/// What building a tower costs, in gold.
pub const COST: u32 = 40;

/// How long a tower is drawn hot after firing, in seconds.
///
/// Long enough to be seen at sixty frames a second and short enough that a
/// tower between shots is plainly between shots — the picture is how a reviewer
/// tells a tower that is firing from one that has nothing in range.
pub const FLASH_S: f64 = 0.12;

/// How fast a bolt travels, in metres a second.
///
/// **Chosen so that one tick's travel is longer than a creep is wide**, which
/// is what makes the sweep load-bearing rather than decorative — see the module
/// docs, and `a_tick_of_a_bolt_is_longer_than_a_creep_is_wide`, which asserts
/// the inequality against the simulation rate.
pub const BOLT_SPEED: f64 = 120.0;

// ---------------------------------------------------------------------------
// The tower
// ---------------------------------------------------------------------------

/// One built tower.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tower {
    /// Which of [`PLOTS`] it stands on.
    plot: usize,
    /// When it may fire again, in the stage's elapsed seconds.
    ready_at: f64,
    /// When it last fired, in the same seconds — read by the frame and by
    /// nothing in the simulation.
    fired_at: f64,
}

impl Tower {
    /// Builds a tower on `plot`, able to fire at once.
    #[must_use]
    pub const fn new(plot: usize) -> Self {
        Self {
            plot,
            ready_at: 0.0,
            fired_at: f64::NEG_INFINITY,
        }
    }

    /// Which plot it stands on.
    #[must_use]
    pub const fn plot(&self) -> usize {
        self.plot
    }

    /// Where its bolts start, in metres.
    ///
    /// # Panics
    ///
    /// If its plot is not a plot, which `crate::game`'s validation makes
    /// unreachable: a `PlaceTower` naming a plot outside [`PLOTS`] is refused
    /// before a tower is built.
    #[must_use]
    pub fn muzzle(&self) -> DVec3 {
        let at = PLOTS[self.plot];
        DVec3::new(at.x, MUZZLE_Y, at.z)
    }

    /// Whether its reload has finished.
    #[must_use]
    pub fn is_ready(&self, now: f64) -> bool {
        now >= self.ready_at
    }

    /// Records a shot and starts the reload.
    pub fn fired(&mut self, now: f64) {
        self.ready_at = now + RELOAD_S;
        self.fired_at = now;
    }

    /// Whether the frame draws it hot.
    #[must_use]
    pub fn is_firing(&self, now: f64) -> bool {
        now - self.fired_at < FLASH_S
    }
}

/// Which creep a tower at `from` shoots, or `None` for a tower with nothing in
/// range.
///
/// **The creep nearest the exit**, which is every tower defense's rule: the one
/// with the least path left is the one about to cost a life.
/// [`Creep::along`] is that ordering, and it is a total one over the creeps on
/// the field, so two runs pick the same target.
///
/// `scratch` is the caller's so the query allocates nothing — this runs once
/// per tower per tick.
#[must_use]
pub fn acquire(
    world: &mut PhysicsWorld,
    creeps: &[Creep],
    from: DVec3,
    scratch: &mut Vec<ColliderId>,
) -> Option<usize> {
    world.overlap_sphere_into(from, RANGE_M, scratch);
    let mut best: Option<(usize, f64)> = None;
    for id in scratch.iter() {
        // The ground and the exit volume are in range of every tower, and both
        // come back from the overlap. See the module docs.
        let Some(index) = creeps.iter().position(|creep| creep.body() == *id) else {
            continue;
        };
        let along = creeps[index].along();
        if best.is_none_or(|(_, furthest)| along > furthest) {
            best = Some((index, along));
        }
    }
    best.map(|(index, _)| index)
}

// ---------------------------------------------------------------------------
// The bolt
// ---------------------------------------------------------------------------

/// What became of a bolt over one tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoltOutcome {
    /// Still in the air.
    Flying,
    /// It struck the creep whose body this is.
    Hit(ColliderId),
    /// It struck the map — in practice the ground, which every bolt is
    /// descending toward. It is gone and nothing was damaged.
    Spent,
}

/// One bolt in the air.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bolt {
    /// Where its centre is, in metres.
    at: DVec3,
    /// The unit direction it is travelling in, kept so a bolt whose target died
    /// still has somewhere to go.
    heading: DVec3,
    /// The body it was fired at. A [`ColliderId`] rather than an index into the
    /// creep list, because that list is swap-removed as creeps die: an index
    /// would silently come to mean a different creep, and a stale
    /// [`ColliderId`] resolves to nothing instead.
    target: ColliderId,
    /// What it takes off whatever it hits.
    damage: u32,
}

impl Bolt {
    /// Fires a bolt from `from` at `target`.
    #[must_use]
    pub fn fire(from: DVec3, target: &Creep, damage: u32) -> Self {
        Self {
            at: from,
            heading: (target.centre() - from).normalize_or_zero(),
            target: target.body(),
            damage,
        }
    }

    /// Where it is, for the frame to draw it.
    #[must_use]
    pub const fn at(&self) -> DVec3 {
        self.at
    }

    /// What it takes off what it hits.
    #[must_use]
    pub const fn damage(&self) -> u32 {
        self.damage
    }

    /// Flies one tick, and says what it met on the way.
    ///
    /// The sweep is the whole of the collision — see the module docs. A hit on
    /// anything that is not a creep is [`BoltOutcome::Spent`]: the map stops a
    /// bolt, it does not bounce one.
    pub fn step(&mut self, world: &mut PhysicsWorld, creeps: &[Creep], dt: f64) -> BoltOutcome {
        if let Some(creep) = creeps.iter().find(|creep| creep.body() == self.target) {
            let toward = creep.centre() - self.at;
            if toward.length_squared() > 0.0 {
                self.heading = toward.normalize();
            }
        }
        let to = self.at + self.heading * BOLT_SPEED * dt;
        if let Some((id, hit)) = world.sweep_sphere(&Segment::new(self.at, to), BOLT_RADIUS) {
            // Left where it struck rather than at the end of the segment, so
            // the last place a bolt was is a place on a surface.
            self.at = hit.point;
            return if creeps.iter().any(|creep| creep.body() == id) {
                BoltOutcome::Hit(id)
            } else {
                BoltOutcome::Spent
            };
        }
        self.at = to;
        BoltOutcome::Flying
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creep::Creep;
    use crate::map;
    use crate::wave::WAVES;

    /// One tick at the sample's own rate.
    const DT: f64 = 1.0 / crate::game::DEFAULT_TICK_HZ as f64;

    /// A creep `along` metres into the path, in a world with the map in it.
    fn creep_at(world: &mut PhysicsWorld, along: f64) -> Creep {
        let mut creep = Creep::spawn(world, &WAVES[0]);
        // Walked rather than placed, because `advance` is what writes the
        // sphere: a creep whose number moved and whose body did not is exactly
        // the failure these queries would then fail to see.
        let ticks = (along / (WAVES[0].speed * DT)).round() as u64;
        for _ in 0..ticks {
            creep.advance(world, DT);
        }
        creep
    }

    /// **One tick of a bolt is longer than a creep is wide.** The inequality
    /// the sweep exists for, asserted against the constants rather than assumed
    /// by the test below it: without it that test would pass on a build with no
    /// sweep in it at all.
    #[test]
    fn a_tick_of_a_bolt_is_longer_than_a_creep_is_wide() {
        let step = BOLT_SPEED * DT;
        let shadow = 2.0 * (map::CREEP_RADIUS + BOLT_RADIUS);
        assert!(
            step > shadow,
            "a bolt covers {step:.2} m a tick and a creep casts a {shadow:.2} m shadow, so a \
             test at the two ends of a tick would catch it",
        );
    }

    /// **A bolt hits a creep that a test at either end of the tick would
    /// miss** — and the creep is moving while it happens.
    ///
    /// The two `overlap_sphere` readings are the control: they are the check a
    /// build without continuous collision would be making, and both of them
    /// answer no on the very tick the bolt goes through the creep. What sees it
    /// is [`PhysicsWorld::sweep_sphere`] over the segment between them.
    #[test]
    fn a_bolt_hits_a_creep_that_a_test_at_either_end_of_the_tick_would_miss() {
        let (mut world, _) = map::world();
        let mut creep = creep_at(&mut world, 6.0);

        // Half a tick's travel short of the creep, aimed straight at it: one
        // step carries the bolt the same distance out the far side.
        let half_step = 0.5 * BOLT_SPEED * DT;
        let approach = DVec3::new(0.0, 0.0, 1.0);
        let from = creep.centre() + approach * half_step;
        let mut bolt = Bolt::fire(from, &creep, DAMAGE);

        assert!(
            !world
                .overlap_sphere(from, BOLT_RADIUS)
                .contains(&creep.body()),
            "the bolt starts the tick already inside the creep, so this proves nothing",
        );

        // The creep walks first, exactly as the tick order has it.
        creep.advance(&mut world, DT);
        let creeps = [creep];
        // Where the tick's segment ends, worked out the way `Bolt::step` works
        // it out — this is the *other* place a static test would look, and it
        // is read before the step because a hit leaves the bolt on the contact
        // point rather than at the end of its segment.
        let heading = (creeps[0].centre() - from).normalize();
        let tick_end = from + heading * BOLT_SPEED * DT;
        assert!(
            !world
                .overlap_sphere(tick_end, BOLT_RADIUS)
                .contains(&creeps[0].body()),
            "the tick ends with the bolt inside the creep, so a static test would have caught it",
        );

        let outcome = bolt.step(&mut world, &creeps, DT);
        assert_eq!(
            outcome,
            BoltOutcome::Hit(creeps[0].body()),
            "the sweep did not find the creep it flew through",
        );
    }

    /// **A tower ignores everything in range that is not a creep**, and the
    /// query really does hand it those things.
    ///
    /// The second assertion is what makes the first one mean something: if the
    /// overlap returned creeps alone, a build with no filter at all would pass.
    #[test]
    fn a_tower_ignores_everything_in_range_that_is_not_a_creep() {
        let (mut world, exit) = map::world();
        let mut scratch = Vec::new();
        // The gate plot, which is the one the exit volume stands beside.
        let gate = PLOTS
            .iter()
            .position(|plot| plot.label == "gate")
            .expect("the map has a gate plot");
        let muzzle = Tower::new(gate).muzzle();

        let in_range = world.overlap_sphere(muzzle, RANGE_M);
        assert!(
            in_range.contains(&exit),
            "the gate tower does not have the exit volume in range, so this proves nothing",
        );
        assert!(
            in_range.len() > 1,
            "only one thing is in range of the gate tower",
        );
        assert_eq!(
            acquire(&mut world, &[], muzzle, &mut scratch),
            None,
            "a tower with no creeps on the field acquired something",
        );
    }

    /// **A tower shoots the creep nearest the exit**, and shoots nothing at all
    /// once its target walks out of range.
    #[test]
    fn a_tower_shoots_the_creep_nearest_the_exit_and_nothing_out_of_range() {
        let (mut world, _) = map::world();
        let mut scratch = Vec::new();
        let entry = PLOTS
            .iter()
            .position(|plot| plot.label == "entry")
            .expect("the map has an entry plot");
        let muzzle = Tower::new(entry).muzzle();

        // Two creeps on the first leg, one further along than the other, both
        // inside the entry tower's reach.
        let behind = creep_at(&mut world, 6.0);
        let ahead = creep_at(&mut world, 9.0);
        for creep in [&behind, &ahead] {
            assert!(
                (creep.centre() - muzzle).length() < RANGE_M,
                "a creep {:.2} m out is not in the entry tower's reach",
                (creep.centre() - muzzle).length(),
            );
        }
        let creeps = [behind, ahead];
        assert_eq!(
            acquire(&mut world, &creeps, muzzle, &mut scratch),
            Some(1),
            "it shot the creep with more path left",
        );

        // And one that has walked away down the far leg is out of reach.
        let (mut world, _) = map::world();
        let gone = creep_at(&mut world, crate::path::length() - 2.0);
        let creeps = [gone];
        assert!(
            (creeps[0].centre() - muzzle).length() > RANGE_M,
            "the far end of the path is inside the entry tower's reach",
        );
        assert_eq!(
            acquire(&mut world, &creeps, muzzle, &mut scratch),
            None,
            "it acquired a creep outside its range",
        );
    }

    /// **A bolt whose target is gone stops in the ground.** Not a hit, and not
    /// a bolt that lives for ever: a creep killed by another tower's shot
    /// leaves this one's in the air with nothing to home on.
    ///
    /// **Where it stopped is the assertion that means something.** A bolt that
    /// merely reported [`BoltOutcome::Spent`] could have expired on a timer, or
    /// have been let through the ground and stopped by the loop's own patience;
    /// a bolt lying on `y = 0` struck the slab. That the slab is always there
    /// to be struck is [`MUZZLE_Y`]'s doing, asserted first.
    #[test]
    fn a_bolt_whose_target_is_gone_stops_in_the_ground() {
        const {
            assert!(
                MUZZLE_Y > map::CREEP_RADIUS,
                "a bolt fired at a creep would not be descending",
            );
        }
        let (mut world, _) = map::world();
        let creep = creep_at(&mut world, 6.0);
        let entry = PLOTS
            .iter()
            .position(|plot| plot.label == "entry")
            .expect("the map has an entry plot");
        let mut bolt = Bolt::fire(Tower::new(entry).muzzle(), &creep, DAMAGE);
        creep.despawn(&mut world);

        let mut outcome = BoltOutcome::Flying;
        for _ in 0..600 {
            outcome = bolt.step(&mut world, &[], DT);
            if outcome != BoltOutcome::Flying {
                break;
            }
        }
        assert_eq!(
            outcome,
            BoltOutcome::Spent,
            "a bolt with no target left is still flying",
        );
        assert!(
            bolt.at().y.abs() < BOLT_RADIUS,
            "it stopped at y = {:.2}, which is not the ground",
            bolt.at().y,
        );
        assert!(
            bolt.at().x.abs() < map::HALF_WIDTH && bolt.at().z.abs() < map::HALF_DEPTH,
            "it stopped at {:?}, off the slab",
            bolt.at(),
        );
    }

    /// **A bolt is gone before its tower can fire again.** The claim
    /// [`crate::map::MAX_BOLTS`] rests on: a tower with a bolt still in the
    /// air has not reloaded, so a full field has one bolt per plot and never
    /// more. Measured on the longest flight there is — a bolt fired at the
    /// edge of a tower's range whose target then vanished, which glides on its
    /// last heading until the ground stops it instead of ending in a creep.
    #[test]
    fn a_bolt_lands_long_before_its_tower_reloads() {
        let (mut world, _) = map::world();
        let entry = PLOTS
            .iter()
            .position(|plot| plot.label == "entry")
            .expect("the map has an entry plot");
        let muzzle = Tower::new(entry).muzzle();

        let mut creep = Creep::spawn(&mut world, &WAVES[0]);
        let mut walked = 0_u64;
        while (creep.centre() - muzzle).length() > RANGE_M {
            creep.advance(&mut world, DT);
            walked += 1;
            assert!(walked < 10_000, "the entry plot never had a creep in range");
        }

        let mut bolt = Bolt::fire(muzzle, &creep, DAMAGE);
        creep.despawn(&mut world);
        let mut ticks = 0_u64;
        while bolt.step(&mut world, &[], DT) == BoltOutcome::Flying {
            ticks += 1;
            assert!(ticks < 10_000, "the bolt never came down");
        }
        let flight = (ticks + 1) as f64 * DT;
        assert!(
            flight < RELOAD_S,
            "a bolt fired at the edge of the range was up for {flight:.3} s \
             against a {RELOAD_S} s reload",
        );
    }

    /// **A tower fires on its reload and not on the tick rate.** One shot per
    /// [`RELOAD_S`], however many ticks that is.
    #[test]
    fn a_tower_fires_on_its_reload_rather_than_every_tick() {
        let mut tower = Tower::new(0);
        let mut shots = 0_u32;
        let seconds = 6.0;
        for tick in 0..(seconds / DT).round() as u64 {
            let now = tick as f64 * DT;
            if tower.is_ready(now) {
                shots += 1;
                tower.fired(now);
                assert!(tower.is_firing(now), "a tower that fired is not drawn hot");
            }
        }
        let expected = (seconds / RELOAD_S).floor() as u32 + 1;
        assert!(
            shots.abs_diff(expected) <= 1,
            "it fired {shots} times over {seconds} s at one per {RELOAD_S} s",
        );
        assert!(
            !tower.is_firing(seconds + 1.0),
            "a tower is still drawn hot a second after its last shot",
        );
    }
}
