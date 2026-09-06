//! What a character has learned, and what a level is worth to them.
//!
//! ```text
//!   a foe falls ──▶ Kind::experience ──┐
//!                                      ├──▶ Stage::experience ──▶ level_for
//!   a stack is taken ──▶ Rarity::experience ──┘                       │
//!                                                                     ▼
//!                                                              health_max
//! ```
//!
//! # A table, not a curve
//!
//! [`THRESHOLDS`] is the whole of the progression: the total experience each
//! level begins at, written out. A formula would be shorter and nobody could
//! read the second level off it — and a sample whose difficulty curve is a
//! polynomial is a sample whose curve is only known by running it. Every level
//! this zone has is a row a reviewer can look at.
//!
//! # The zone is worth exactly enough
//!
//! There is no respawn, so the experience this zone can ever pay out is
//! bounded: one kill per post, one find per drop. The **least** a full clear
//! yields — every foe felled and every stack taken at the commonest tier — is
//! what the last threshold is set against, so the top level is reachable on
//! every seed rather than on the lucky ones.
//! `clearing_the_zone_reaches_the_top_level_on_any_seed` is what holds that to
//! the numbers rather than to this sentence, and [`EXPERIENCE_MAX`] is the
//! other end of the same bound: the most any session can hold, which is what
//! [`crate::save`]'s decoder refuses a payload against.
//!
//! # What a level does
//!
//! It deepens the pool: [`health_max`] is [`crate::foe::HEALTH_MAX`] at the
//! first level and [`HEALTH_PER_LEVEL`] more at each one after it. **It does
//! not heal a wound.** A character who levels mid-fight keeps the health they
//! had and gains the room to take more, and it is a return to the spawn —
//! `crate::game`'s down — that fills the pool to whatever the level now allows.
//! One rule for the ceiling and one for the refill, and between them the health
//! a character is carrying is never above the maximum being drawn beside it.

use crate::foe;
use crate::loot::Rarity;

/// The total experience each level begins at.
///
/// The first is zero, because that is what a character opens with, and each is
/// past the one before it — `the_table_is_a_progression` refuses a row that is
/// not. The last is inside what a full clear of this zone pays out on its
/// unluckiest seed; see the module docs.
pub const THRESHOLDS: [u64; 4] = [0, 30, 75, 130];

/// The highest level [`THRESHOLDS`] describes.
pub const MAX_LEVEL: u32 = THRESHOLDS.len() as u32;

/// How much deeper each level makes the character's pool.
pub const HEALTH_PER_LEVEL: u32 = 25;

/// The most experience a session can hold: every foe felled and every stack
/// taken at the richest tier.
///
/// A **bound rather than a guess** — the roster is fixed, nothing respawns and
/// a stack is only ever minted by a foe falling, so this is arithmetic over
/// [`crate::foe::POSTS`] rather than a number somebody chose. It is what
/// [`crate::save`]'s decoder measures a payload's experience against, and it
/// is why a level read off a save cannot be one no session could have reached.
pub const EXPERIENCE_MAX: u64 = ceiling();

/// [`EXPERIENCE_MAX`], as arithmetic over the roster.
const fn ceiling() -> u64 {
    let mut total = 0;
    let mut index = 0;
    while index < foe::FOES {
        total += foe::POSTS[index].kind.experience() as u64 + Rarity::Rare.experience() as u64;
        index += 1;
    }
    total
}

/// Which level `experience` has reached.
///
/// Never below the first: [`THRESHOLDS`] opens at zero, so a character who has
/// learned nothing is still a character.
#[must_use]
pub const fn level_for(experience: u64) -> u32 {
    let mut level = 1;
    while (level as usize) < THRESHOLDS.len() && experience >= THRESHOLDS[level as usize] {
        level += 1;
    }
    level
}

/// How deep the pool is at `level`.
///
/// A level past [`MAX_LEVEL`] answers with the top level's pool rather than
/// with a deeper one, so a number that did not come from [`level_for`] cannot
/// widen the ceiling [`crate::save`] checks a health against.
#[must_use]
pub const fn health_max(level: u32) -> u32 {
    let capped = if level > MAX_LEVEL { MAX_LEVEL } else { level };
    let steps = capped.saturating_sub(1);
    foe::HEALTH_MAX + steps * HEALTH_PER_LEVEL
}

/// How much of the current level `experience` has covered.
#[must_use]
pub const fn into_level(experience: u64) -> u64 {
    experience - THRESHOLDS[level_for(experience) as usize - 1]
}

/// How much the current level spans, or `None` for a character at the top.
///
/// The denominator the overlay draws [`into_level`] over, and `None` is what
/// makes "there is nothing further" a state the reading can have rather than a
/// division nobody can do.
#[must_use]
pub const fn level_span(experience: u64) -> Option<u64> {
    let level = level_for(experience) as usize;
    if level >= THRESHOLDS.len() {
        return None;
    }
    Some(THRESHOLDS[level] - THRESHOLDS[level - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The least a full clear of this zone pays out: every foe felled and every
    /// stack taken at the commonest tier.
    fn poorest_clear() -> u64 {
        foe::POSTS
            .iter()
            .map(|post| u64::from(post.kind.experience()) + u64::from(Rarity::Common.experience()))
            .sum()
    }

    /// **The table is a progression.** It opens at nothing and every row is
    /// past the one before it, which is what makes [`level_for`]'s walk a walk
    /// rather than a search — and a table with a repeated or a falling row
    /// would give one level two entries.
    #[test]
    fn the_table_is_a_progression() {
        assert_eq!(THRESHOLDS[0], 0, "a character opens below the first level");
        for pair in THRESHOLDS.windows(2) {
            assert!(
                pair[1] > pair[0],
                "{} does not come after {} in the table",
                pair[1],
                pair[0],
            );
        }
        assert_eq!(MAX_LEVEL as usize, THRESHOLDS.len());
    }

    /// **A level begins exactly at its own row, and one experience short of it
    /// is the level below.** The boundary, from both sides — a build that
    /// compared with `>` rather than `>=` passes every other check here and
    /// makes the last point of every level unreachable.
    #[test]
    fn a_level_begins_at_the_value_the_table_gives_it() {
        assert_eq!(level_for(0), 1, "a character who has learned nothing");
        for (index, threshold) in THRESHOLDS.iter().enumerate().skip(1) {
            let level = index as u32 + 1;
            assert_eq!(
                level_for(*threshold),
                level,
                "{threshold} experience is where level {level} begins",
            );
            assert_eq!(
                level_for(threshold - 1),
                level - 1,
                "one short of {threshold} is still level {}",
                level - 1,
            );
        }
        // …and past the last row it stops rather than running on.
        assert_eq!(level_for(EXPERIENCE_MAX), MAX_LEVEL);
        assert_eq!(level_for(u64::MAX), MAX_LEVEL);
    }

    /// **The first level's pool is the one the character starts with**, and
    /// every level after it is deeper by the same step.
    ///
    /// The first half is what lets `crate::game`'s fresh stage open at
    /// [`crate::foe::HEALTH_MAX`] without naming a level: a build that moved
    /// the two apart would open a character below or above their own maximum.
    #[test]
    fn the_first_level_is_the_pool_the_character_starts_with() {
        assert_eq!(health_max(1), foe::HEALTH_MAX);
        for level in 2..=MAX_LEVEL {
            // The strict inequality comes first, and it is the one that
            // matters: the equality under it compares the step with itself, so
            // on a build where a level buys nothing at all it holds and this
            // does not.
            assert!(
                health_max(level) > health_max(level - 1),
                "level {level} is no deeper than the one below it",
            );
            assert_eq!(
                health_max(level),
                health_max(level - 1) + HEALTH_PER_LEVEL,
                "level {level} is not one step deeper than the one below it",
            );
        }
        // A level nobody can reach does not widen the pool, which is the half
        // `crate::save`'s ceiling rests on.
        assert_eq!(health_max(MAX_LEVEL + 1), health_max(MAX_LEVEL));
        assert_eq!(health_max(0), foe::HEALTH_MAX);
    }

    /// **Clearing the zone reaches the top level however the rolls went.** The
    /// claim the last row of the table is set against: the least a full clear
    /// pays out is still inside it, so the top level is a level every seed can
    /// reach rather than one only a lucky haul does.
    ///
    /// The other end is the control: the *most* a session can hold is
    /// [`EXPERIENCE_MAX`], and a decoder that let a payload past it would let a
    /// save claim a level this zone cannot pay for.
    #[test]
    fn clearing_the_zone_reaches_the_top_level_on_any_seed() {
        let least = poorest_clear();
        assert!(
            least >= THRESHOLDS[THRESHOLDS.len() - 1],
            "the poorest clear is worth {least} against a top level at {}",
            THRESHOLDS[THRESHOLDS.len() - 1],
        );
        assert_eq!(level_for(least), MAX_LEVEL);
        assert!(
            EXPERIENCE_MAX >= least,
            "the richest clear is worth less than the poorest one",
        );
        // …and no tier is worth more than the one the ceiling is computed from.
        for tier in Rarity::ALL {
            assert!(
                tier.experience() <= Rarity::Rare.experience(),
                "{} is worth more than the tier EXPERIENCE_MAX is built on",
                tier.label(),
            );
            assert!(
                tier.experience() >= Rarity::Common.experience(),
                "{} is worth less than the tier the poorest clear is built on",
                tier.label(),
            );
        }
    }

    /// **The two halves of the overlay's reading add up to the next level.**
    /// What a player reads off the `NEXT` row: how far into this level they
    /// are, over how far it goes.
    #[test]
    fn the_reading_toward_the_next_level_covers_the_gap_between_two_rows() {
        for (index, threshold) in THRESHOLDS.iter().enumerate() {
            let experience = *threshold;
            assert_eq!(into_level(experience), 0, "a level opens at its own row");
            match level_span(experience) {
                Some(span) => {
                    assert_eq!(
                        experience + span,
                        THRESHOLDS[index + 1],
                        "level {} does not span to the next row",
                        index + 1,
                    );
                    assert_eq!(into_level(experience + span - 1), span - 1);
                }
                None => assert_eq!(
                    index + 1,
                    THRESHOLDS.len(),
                    "a level below the top has nowhere to go",
                ),
            }
        }
        assert_eq!(level_span(EXPERIENCE_MAX), None, "the top level goes on");
    }
}
