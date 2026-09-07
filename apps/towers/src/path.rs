//! `PathSystem`: where a creep is, `s` metres into [`crate::map::PATH`].
//!
//! ```text
//!   along ──▶ point_at ──▶ DVec3   (where the sphere is written)
//!         └─▶ heading_at ─▶ yaw    (which way it is walking)
//! ```
//!
//! # This is a polyline follower and not a spline follower
//!
//! `docs/plan/sample/07-towers.md` asks for creeps that "walk spline", and the
//! engine has no spline type to walk: `crcbl-phys` and `crcbl-scene` offer
//! none, and the only splines in the workspace are `crcbl-anim`'s clip
//! interpolation and the glTF importer's, neither of which is a curve a game
//! can put a body on. So this module measures **straight legs between
//! waypoints**, which is sample code over kinematic bodies exactly as that
//! document predicted. The gap is recorded in `docs/backlog.md` rather than
//! papered over here; what a spline would change is the corners, where a creep
//! currently turns in one tick.
//!
//! # Distance is the state, not a leg index and a fraction
//!
//! A creep carries one number, the metres it has walked, and every reading is a
//! function of it. That is what makes a creep's position a pure function of its
//! own clock — two runs that spawned the same creep at the same tick put it in
//! the same place — and it is why [`point_at`] is a free function rather than a
//! method on anything.

use crcbl::math::DVec3;

use crate::map::{LEGS, PATH};

/// How long one leg is, in metres.
///
/// # Panics
///
/// If `leg` is not a leg. Every caller here iterates `0..LEGS`.
#[must_use]
pub fn leg_length(leg: usize) -> f64 {
    (PATH[leg + 1] - PATH[leg]).length()
}

/// How long the whole path is, in metres.
#[must_use]
pub fn length() -> f64 {
    (0..LEGS).map(leg_length).sum()
}

/// Which leg `s` metres in falls on, and how far along that leg it is.
///
/// Clamped at both ends: before the start is the first leg at zero, past the
/// finish is the last leg at its full length. A creep is taken off the field by
/// the exit volume rather than by running out of path — see
/// [`crate::creep::has_reached_the_exit`] — so the far clamp is a guard rather
/// than a state a run reaches.
fn leg_of(s: f64) -> (usize, f64) {
    let mut left = s.max(0.0);
    for leg in 0..LEGS {
        let span = leg_length(leg);
        if left <= span || leg + 1 == LEGS {
            return (leg, left.min(span));
        }
        left -= span;
    }
    // `LEGS` is `PATH.len() - 1` and `PATH` has at least two waypoints, so the
    // loop above always returns.
    unreachable!("the path has no legs")
}

/// Where a creep `s` metres along the path stands, on the ground.
#[must_use]
pub fn point_at(s: f64) -> DVec3 {
    let (leg, along) = leg_of(s);
    let step = PATH[leg + 1] - PATH[leg];
    PATH[leg] + step.normalize_or_zero() * along
}

/// Which way a creep `s` metres along the path is walking, as a yaw in
/// `apps/breach::camera::forward`'s measure — zero looks down `-Z` and a rising
/// yaw swings toward `+X`.
///
/// Read by the frame and by nothing in the simulation: a creep on a polyline
/// has no steering to do.
#[must_use]
pub fn heading_at(s: f64) -> f64 {
    let (leg, _) = leg_of(s);
    let step = PATH[leg + 1] - PATH[leg];
    step.x.atan2(-step.z)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One centimetre, which is the tolerance every comparison here is made to.
    const EPS: f64 = 1e-2;

    /// **The two ends of the path are the two ends of the waypoint list**, and
    /// the length is the legs added up. The claim every other reading rests on.
    #[test]
    fn the_ends_of_the_walk_are_the_ends_of_the_list() {
        assert!((point_at(0.0) - PATH[0]).length() < EPS);
        assert!((point_at(length()) - PATH[PATH.len() - 1]).length() < EPS);
        // Clamped rather than extrapolated at both ends.
        assert!((point_at(-5.0) - PATH[0]).length() < EPS);
        assert!((point_at(length() + 50.0) - PATH[PATH.len() - 1]).length() < EPS);

        let legs: f64 = (0..LEGS).map(leg_length).sum();
        assert!((length() - legs).abs() < EPS);
    }

    /// **Every waypoint is somewhere the walk actually passes through**, at the
    /// distance the legs before it add up to. A follower that skipped a corner
    /// — the failure a leg index off by one produces — puts a creep on the
    /// diagonal between two legs and passes an end-to-end test unchanged.
    #[test]
    fn the_walk_passes_through_every_waypoint() {
        let mut so_far = 0.0;
        for (leg, waypoint) in PATH.iter().enumerate() {
            assert!(
                (point_at(so_far) - *waypoint).length() < EPS,
                "waypoint {leg} is not {so_far:.2} m in",
            );
            if leg < LEGS {
                so_far += leg_length(leg);
            }
        }
        assert!(
            (so_far - length()).abs() < EPS,
            "the waypoints do not add up to the path",
        );
    }

    /// **Walking is monotone and at the speed asked for**: a step of `d` metres
    /// moves a creep `d` metres, except across a corner, where the polyline
    /// turns and the straight-line distance is shorter.
    ///
    /// Asserted over the whole path in ten-centimetre steps, so the corners are
    /// in the sample rather than avoided by it.
    #[test]
    fn a_step_along_the_path_covers_the_distance_it_asks_for() {
        const STEP: f64 = 0.1;
        let mut travelled = 0.0;
        let mut s = 0.0;
        while s + STEP <= length() {
            let moved = (point_at(s + STEP) - point_at(s)).length();
            assert!(
                moved <= STEP + EPS,
                "a {STEP} m step covered {moved:.4} m at s = {s:.2}",
            );
            travelled += moved;
            s += STEP;
        }
        // The corners are the only place the two disagree, and there are `LEGS
        // - 1` of them, each losing under one step.
        assert!(
            travelled > length() - (LEGS as f64) * STEP - EPS,
            "walking the path in {STEP} m steps covered {travelled:.2} m of {:.2}",
            length(),
        );
    }

    /// **The heading is the leg's own direction**, which is what the frame
    /// turns a creep by. One reading per leg, taken at its middle so a corner
    /// cannot answer for its neighbour.
    #[test]
    fn the_heading_is_the_leg_the_creep_is_on() {
        let mut so_far = 0.0;
        for (leg, pair) in PATH.windows(2).enumerate() {
            let span = leg_length(leg);
            let step = pair[1] - pair[0];
            let expected = step.x.atan2(-step.z);
            let read = heading_at(so_far + 0.5 * span);
            assert!(
                (read - expected).abs() < 1e-9,
                "leg {leg} reads {read} and runs at {expected}",
            );
            so_far += span;
        }
    }
}
