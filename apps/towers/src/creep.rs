//! `CreepSystem`: one archetype of creep, walking [`crate::map::PATH`] as a
//! kinematic body in the broadphase.
//!
//! ```text
//!   along += speed × dt ──▶ path::point_at ──▶ PhysicsWorld::set_sphere
//!                                                      │
//!                              exit trigger ◀── overlap_sphere ──▶ a life
//! ```
//!
//! # A creep is a sphere the game moves, not a body physics moves
//!
//! There is no integrator and no controller here. A creep's whole state is the
//! metres it has walked; every tick that number grows and the sphere is written
//! to wherever [`crate::path`] puts it. That is what
//! `docs/plan/sample/07-towers.md` means by "kinematic spline-followers in the
//! broadphase" — the body is in the world so the towers' queries can find it and
//! the bolts' sweeps can hit it, and nothing in `crcbl-phys` decides where it
//! goes.
//!
//! # Reaching the exit is an overlap and not a distance
//!
//! [`has_reached_the_exit`] asks the world whether the creep's sphere touches
//! [`crate::map::exit_collider`], rather than comparing `along` against
//! [`crate::path::length`]. The two answers are genuinely different — the
//! volume is two metres across, so it catches a creep about a metre and a half
//! before the last waypoint — and the overlap is the one the sample is for:
//! a **trigger volume** is `docs/plan/05-physics.md`'s L0 feature this map
//! exists to drive, and a distance check would be the game doing the physics'
//! job. `a_creep_is_taken_by_the_volume_before_the_path_runs_out` is what holds
//! the two apart.
//!
//! # One archetype, and the plan asks for three
//!
//! This is the "fast" creep of that document's scope line. Tanky and swarm are
//! owed, and so are the health bars over their heads; the wave table's rows
//! carry health and speed per wave, which is where a second archetype will
//! attach.

use crcbl::math::DVec3;
use crcbl::phys::{ColliderId, PhysicsWorld, Sphere};

use crate::map::CREEP_RADIUS;
use crate::path;
use crate::wave::Wave;

/// One creep.
#[derive(Debug)]
pub struct Creep {
    /// Its sphere in the world — what a tower's overlap finds and what a bolt's
    /// sweep hits.
    body: ColliderId,
    /// How far it has walked, in metres along [`crate::map::PATH`]. **The whole
    /// of its position**: see [`crate::path`].
    along: f64,
    /// How fast it walks, in metres a second, off its wave's row.
    speed: f64,
    /// What it has left, and what it started with.
    health: u32,
    max_health: u32,
    /// What killing it pays.
    bounty: u32,
}

/// Where a creep with `along` metres behind it has its centre.
///
/// A free function because the frame wants it for a creep it does not hold and
/// [`Creep::centre`] wants it for one it does.
#[must_use]
pub fn centre_at(along: f64) -> DVec3 {
    path::point_at(along) + DVec3::Y * CREEP_RADIUS
}

impl Creep {
    /// Puts a creep on the first waypoint, with its sphere in `world`.
    #[must_use]
    pub fn spawn(world: &mut PhysicsWorld, wave: &Wave) -> Self {
        let body = world.add_sphere(Sphere::new(centre_at(0.0), CREEP_RADIUS));
        Self {
            body,
            along: 0.0,
            speed: wave.speed,
            health: wave.health,
            max_health: wave.health,
            bounty: wave.bounty,
        }
    }

    /// Its sphere, for a query's answer to be matched against.
    #[must_use]
    pub const fn body(&self) -> ColliderId {
        self.body
    }

    /// How far it has walked, in metres.
    ///
    /// **What a tower picks its target by**: the creep nearest the exit is the
    /// one with the most to lose, which is the rule every tower defense uses.
    #[must_use]
    pub const fn along(&self) -> f64 {
        self.along
    }

    /// What killing it pays.
    #[must_use]
    pub const fn bounty(&self) -> u32 {
        self.bounty
    }

    /// What it has left.
    #[must_use]
    pub const fn health(&self) -> u32 {
        self.health
    }

    /// Where its centre is, in metres.
    #[must_use]
    pub fn centre(&self) -> DVec3 {
        centre_at(self.along)
    }

    /// Walks one tick and writes the sphere where the walk left it.
    ///
    /// The write is not optional and not deferred: the towers' queries and the
    /// bolts' sweeps run later in the same tick against this world, and a body
    /// left at last tick's place is a target that cannot be hit where it is
    /// drawn.
    pub fn advance(&mut self, world: &mut PhysicsWorld, dt: f64) {
        self.along += self.speed * dt;
        world.set_sphere(self.body, Sphere::new(self.centre(), CREEP_RADIUS));
    }

    /// Takes `damage` off it. Answers whether that killed it.
    pub fn wounded(&mut self, damage: u32) -> bool {
        self.health = self.health.saturating_sub(damage);
        self.health == 0
    }

    /// What the frame draws of it.
    #[must_use]
    pub fn view(&self) -> CreepView {
        CreepView {
            centre: self.centre(),
            facing: {
                #[allow(clippy::cast_possible_truncation)]
                let facing = path::heading_at(self.along) as f32;
                facing
            },
            hurt: 2 * self.health <= self.max_health,
        }
    }

    /// Takes its sphere out of the world.
    ///
    /// Consuming, because a creep without a body is not a creep: the id would
    /// go on resolving to whatever collider recycled its slot, which is the one
    /// mistake `ColliderId`'s generation exists to make impossible.
    pub fn despawn(self, world: &mut PhysicsWorld) {
        world.remove(self.body);
    }
}

/// Where one creep is drawn, and how.
///
/// The frame's copy of a [`Creep`], snapshotted with the rest of
/// [`crate::game::RenderState`] so a draw never reads through the tick's lock.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CreepView {
    /// Where its centre is, in metres.
    pub centre: DVec3,
    /// Which way it is walking, in [`crate::path::heading_at`]'s measure.
    pub facing: f32,
    /// Whether it is down to half its health or less — the one piece of a
    /// creep's state the picture carries.
    pub hurt: bool,
}

/// Whether `creep` is standing in the exit volume.
///
/// `scratch` is the caller's so the query allocates nothing: this runs once per
/// creep per tick, which is `apps/horde`'s reason for hoisting the same buffer
/// out of the same loop.
#[must_use]
pub fn has_reached_the_exit(
    world: &mut PhysicsWorld,
    creep: &Creep,
    exit: ColliderId,
    scratch: &mut Vec<ColliderId>,
) -> bool {
    world.overlap_sphere_into(creep.centre(), CREEP_RADIUS, scratch);
    scratch.contains(&exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;
    use crate::wave::WAVES;

    /// One tick at sixty a second.
    const DT: f64 = 1.0 / 60.0;

    /// **A creep walks at its wave's speed and drags its collider with it.**
    /// The second half is the one worth asserting: a follower that moved the
    /// number and not the sphere leaves every tower shooting at where the creep
    /// was on the tick it spawned.
    #[test]
    fn a_creep_walks_its_speed_and_its_body_follows() {
        let (mut world, _) = map::world();
        let mut creep = Creep::spawn(&mut world, &WAVES[0]);
        let start = creep.centre();

        for _ in 0..60 {
            creep.advance(&mut world, DT);
        }
        let walked = (creep.centre() - start).length();
        assert!(
            (walked - WAVES[0].speed).abs() < 0.05,
            "a second of walking covered {walked:.2} m at {} m/s",
            WAVES[0].speed,
        );
        assert!(
            world
                .overlap_sphere(creep.centre(), 0.01)
                .contains(&creep.body()),
            "the collider was left behind at the spawn",
        );
        assert!(
            !world.overlap_sphere(start, 0.01).contains(&creep.body()),
            "the collider is in two places, so `set_sphere` added rather than moved",
        );
    }

    /// **The exit volume is what takes a creep, and it takes it before the path
    /// runs out.** The margin is the whole point: a build that compared `along`
    /// against `path::length()` would agree with this test at the last waypoint
    /// and disagree everywhere the volume reaches, which is a metre and a half
    /// of the last leg.
    #[test]
    fn a_creep_is_taken_by_the_volume_before_the_path_runs_out() {
        let (mut world, exit) = map::world();
        let mut creep = Creep::spawn(&mut world, &WAVES[0]);
        let mut scratch = Vec::new();

        assert!(
            !has_reached_the_exit(&mut world, &creep, exit, &mut scratch),
            "a creep on the spawn is already in the exit",
        );

        let mut caught = None;
        for _ in 0..(60 * 60) {
            creep.advance(&mut world, DT);
            if has_reached_the_exit(&mut world, &creep, exit, &mut scratch) {
                caught = Some(creep.along());
                break;
            }
        }
        let caught = caught.expect("a creep walking the whole path never reached the exit");
        let left = path::length() - caught;
        assert!(
            left > 0.5,
            "the volume caught it {left:.2} m from the end, which is the end rather than the \
             volume",
        );
        assert!(
            left < map::EXIT_HALF.x + CREEP_RADIUS + 0.2,
            "it was caught {left:.2} m out, further than the volume reaches",
        );
    }

    /// **Damage runs out at zero and not below it**, and a creep is drawn hurt
    /// once it is down to half.
    #[test]
    fn a_creep_dies_when_its_health_runs_out() {
        let (mut world, _) = map::world();
        let mut creep = Creep::spawn(&mut world, &WAVES[0]);
        let half = WAVES[0].health / 2;

        assert!(!creep.view().hurt, "a creep spawns hurt");
        assert!(!creep.wounded(half), "half the health killed it");
        assert!(creep.view().hurt, "half the health is not drawn hurt");
        assert!(
            creep.wounded(WAVES[0].health * 4),
            "an overkill did not kill it",
        );
        assert_eq!(creep.health(), 0, "health went below zero");
    }

    /// **A despawned creep's body leaves the world**, which is what stops a
    /// dead creep going on being a thing towers acquire and bolts stop at.
    #[test]
    fn a_despawned_creep_is_no_longer_in_the_world() {
        let (mut world, _) = map::world();
        let creep = Creep::spawn(&mut world, &WAVES[0]);
        let (body, centre) = (creep.body(), creep.centre());
        assert!(world.overlap_sphere(centre, 0.01).contains(&body));

        creep.despawn(&mut world);
        assert!(
            !world.overlap_sphere(centre, 0.01).contains(&body),
            "the collider outlived the creep",
        );
    }
}
