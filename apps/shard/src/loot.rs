//! What a felled foe leaves, and the grid the character carries it in.
//!
//! ```text
//!   data/items.ron ──▶ Catalog ──┬──▶ dropped(seed, foe) ──▶ Stack ──▶ the floor
//!                                │                                       │
//!                                └──▶ carried() ──▶ Grid ◀── Grid::insert ┘
//! ```
//!
//! # The kit is consumed, not extended
//!
//! Every rule about where an item fits, what a footprint is once it is turned
//! and what a stack holds is [`crcbl::inventory`]'s —
//! `docs/plan/34-inventory.md`'s part 2, which `apps/shard` is the first
//! consumer of. What is this module's is the *content*: the item table, how big
//! the character's own container is, which item a foe leaves, and how close a
//! player has to be to take it. That split is
//! `docs/plan/sample/15-shard.md`'s exit criterion — the kit used "without a
//! single engine change made on its behalf" — and it is why nothing here
//! reaches into the crate's internals.
//!
//! # The table is a data file
//!
//! `data/items.ron` is `include_str!`-ed rather than read through an
//! [`crcbl::store::StorageSource`], because shard has no asset source at all:
//! `crate::web`'s own docs say every byte this sample draws with is compiled
//! into the module. A file is still the right shape — it is the format
//! [`crcbl::inventory::Catalog`] reads and the one
//! `crcbl_render::stack::CameraStack` already writes — and routing it through
//! an asset source is what breach's adoption should force, not this slice.
//!
//! # The roll is a hash, not a stream
//!
//! Which item a foe leaves is a function of the seed and the foe's index and of
//! nothing else — `apps/sparks/src/show.rs`'s rule, and for the same reason: a
//! draw from a stream depends on how many draws came before it, so a zone
//! whose foes were felled in a different order would leave different loot and
//! two runs of one seed would stop agreeing. A hash has no such history, so
//! [`a_headless_run_is_deterministic`](crate::app) holds whatever the player
//! did.
//!
//! # Every instance is one a felled foe left
//!
//! A [`StackId`] here is the foe's own index plus one, so the roster bounds how
//! many instances can exist: three foes, at most three stacks, and the same
//! three ids however the session went. That is what makes conservation
//! checkable rather than asserted — `crate::game`'s floor plus the character's
//! grid is exactly the set of felled foes, and `crate::save`'s decoder refuses
//! a payload claiming a stack no foe could have left.

use crcbl::inventory::{Catalog, Cell, Grid, InventoryError, ItemId, Stack, StackId};

/// The item table, compiled in. See the module docs for why it is not an asset.
const ITEMS_RON: &str = include_str!("../data/items.ron");

/// How many cells across the character carries.
pub const GRID_W: u8 = 4;

/// How many cells down. Sixteen cells holds everything this zone can drop —
/// the plate is four of them, the key three and the brand two — with room to
/// drag things around, which is the whole point of a grid rather than a list.
pub const GRID_H: u8 = 4;

/// How close the character's feet have to be to a stack to take it, in metres.
///
/// **Past the cleave**, so a body the character could reach to fell is one they
/// can reach to loot from where they are standing. That is a measurement rather
/// than a preference: a living foe's collider stops the walk, and the character
/// is held about 2.14 m from its centre by the two capsules' radii — so a reach
/// at [`crate::foe::STRIKE_REACH_M`]'s 2.2 or below is one a player cannot use
/// without first stepping onto a corpse.
///
/// It is a distance and nothing else: there is no line of sight on the pickup,
/// so a stack on the far side of a doorpost within this radius can be taken
/// through the stone. `docs/backlog.md` carries that as the reach-and-line-of-
/// sight validation `docs/plan/34-inventory.md` asks a server for.
pub const LOOT_REACH_M: f64 = 2.5;

/// The published seed: the loot everybody who loads the page finds.
pub const DEFAULT_SEED: u32 = 0x1007_5EED;

/// An odd multiplier, so two foes on one seed do not share low bits.
const FOE_STRIDE: u32 = 0x9E37_79B9;

/// A second, unrelated key, so how many of an item a foe leaves is not a
/// function of which item it left.
const COUNT_SALT: u32 = 0x2545_F491;

/// The item table every session reads, parsed once.
///
/// # Panics
///
/// If `data/items.ron` is not a catalogue. That is this crate's own file being
/// wrong rather than a state a run can be in — the same call
/// `apps/sparks/src/show.rs` makes of its stock effects — and
/// `the_shipped_table_is_one_the_kit_accepts` is what catches it before a page
/// does.
#[must_use]
pub fn catalog() -> &'static Catalog {
    static CATALOG: std::sync::OnceLock<Catalog> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        Catalog::from_ron(ITEMS_RON).expect("data/items.ron is this crate's own catalogue")
    })
}

/// The container the character carries: [`GRID_W`] by [`GRID_H`], accepting
/// anything.
///
/// Unfiltered on purpose. A filter is what the plan makes an equipment slot out
/// of, and this sample equips nothing — a grid that refused an item would be
/// refusing it for a rule shard does not have.
///
/// # Panics
///
/// Never: the dimensions are constants and neither is zero, which is the only
/// thing [`Grid::new`] refuses.
#[must_use]
pub fn carried() -> Grid {
    Grid::new(GRID_W, GRID_H, None).expect("a 4x4 grid has cells in it")
}

/// The 32-bit finaliser this module's rolls are made of: Chris Wellons'
/// `lowbias32`, as published, with its two multipliers and three shifts.
///
/// Named rather than hand-tuned because a mixer written from memory is one
/// nobody can check: this is a published function whose avalanche was measured
/// by its author, and `two_foes_on_one_seed_do_not_share_a_drop` is what says
/// this transcription of it spreads.
const fn mix(mut hash: u32) -> u32 {
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb_352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846c_a68b);
    hash ^ (hash >> 16)
}

/// Which item foe `index` leaves, on `seed`.
///
/// A pure function of the two, so the same seed drops the same thing whatever
/// order the zone was cleared in — see the module docs.
///
/// # Panics
///
/// If the catalogue is empty, which `the_shipped_table_is_one_the_kit_accepts`
/// rules out for the shipped file.
#[must_use]
pub fn dropped(seed: u32, index: usize) -> ItemId {
    let items = catalog().len();
    assert!(items > 0, "an empty catalogue has nothing to drop");
    let roll = mix(seed ^ (index as u32).wrapping_mul(FOE_STRIDE));
    #[allow(clippy::cast_possible_truncation)]
    ItemId((roll as usize % items) as u16)
}

/// The whole stack foe `index` leaves: the item, its identity, and how many of
/// it.
///
/// The count is a second roll rather than a slice of the first, so a table edit
/// that changes how many items there are does not silently change how many of
/// each a foe leaves.
///
/// # Panics
///
/// If [`dropped`] named an item the catalogue does not hold, which cannot
/// happen for an id it derived from that catalogue's own length.
#[must_use]
pub fn drop_of(seed: u32, index: usize) -> Stack {
    let item = dropped(seed, index);
    let max = catalog()
        .get(item)
        .expect("dropped names an item of this catalogue")
        .stack_max();
    let roll = mix(seed ^ COUNT_SALT ^ (index as u32).wrapping_mul(FOE_STRIDE));
    let count = 1 + (roll % u32::from(max)) as u16;
    Stack::new(item, stack_id(index), count)
}

/// The identity of foe `index`'s drop: **one-based**, so a `StackId(0)` is
/// never one this zone minted and a zeroed byte range cannot read as a stack.
#[must_use]
pub fn stack_id(index: usize) -> StackId {
    StackId(index as u32 + 1)
}

/// Which foe a stack came from, or `None` for an id no foe of `roster` mints.
///
/// The inverse of [`stack_id`], and it is what [`crate::save`]'s decoder checks
/// a payload's stacks against: an id outside the roster is one no session could
/// have produced.
#[must_use]
pub fn foe_of(stack: StackId, roster: usize) -> Option<usize> {
    let index = usize::try_from(stack.0).ok()?.checked_sub(1)?;
    (index < roster).then_some(index)
}

/// Where inside a footprint the pointer grabbed it, applied to where it was
/// dropped.
///
/// A drag names the cell under the pointer, not the placement's origin, so a
/// `2×2` plate grabbed by its bottom-right corner and dropped two cells over
/// has to land two cells over — not with its *origin* under the pointer, which
/// would jump it up and left by its own size. `None` is a destination that
/// would put the origin off the top or the left edge, which is a move
/// [`Grid::move_within`] would refuse anyway.
#[must_use]
pub fn dragged_origin(origin: Cell, grabbed: Cell, dropped_on: Cell) -> Option<Cell> {
    let dx = grabbed.x.checked_sub(origin.x)?;
    let dy = grabbed.y.checked_sub(origin.y)?;
    Some(Cell::new(
        dropped_on.x.checked_sub(dx)?,
        dropped_on.y.checked_sub(dy)?,
    ))
}

/// What the character is carrying, in grams — the kit's flat sum over one grid.
#[must_use]
pub fn weight_g(grid: &Grid) -> u64 {
    grid.weight_g(catalog())
}

/// Puts `stack` wherever it fits, or hands it back with the reason it did not.
///
/// One call rather than the kit's [`Grid::insert`] spelled at each site, so the
/// catalogue this sample places against is named once.
///
/// # Errors
///
/// Whatever [`Grid::insert`] refused it for — [`InventoryError::NoRoom`] for a
/// full grid, which is the one a player can cause.
pub fn stow(grid: &mut Grid, stack: Stack) -> Result<(), InventoryError> {
    grid.insert(catalog(), stack).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foe::FOES;

    /// **The shipped table is one the kit accepts, and it holds what the roll
    /// indexes.** [`catalog`] panics on a file that is not a catalogue, so
    /// without this the first failure would be a page that does not open.
    #[test]
    fn the_shipped_table_is_one_the_kit_accepts() {
        let catalog = catalog();
        assert!(!catalog.is_empty(), "an empty table drops nothing");
        for (id, item) in catalog.items() {
            assert!(!item.name().is_empty(), "item {} has no name", id.0);
            assert!(
                item.stack_max() > 0,
                "{} cannot be held at all",
                item.name()
            );
            assert!(
                item.shape().width() <= GRID_W && item.shape().height() <= GRID_H,
                "{} is {}x{}, which no rotation fits in a {GRID_W}x{GRID_H} grid",
                item.name(),
                item.shape().width(),
                item.shape().height(),
            );
        }
    }

    /// **Everything this zone can drop fits in the grid at once.** The control
    /// for every "the grid refused it" path being about a *full* grid rather
    /// than about a table nobody sized against the container.
    #[test]
    fn one_of_every_drop_this_zone_can_make_fits_the_carried_grid() {
        let mut grid = carried();
        for index in 0..FOES {
            let stack = drop_of(DEFAULT_SEED, index);
            stow(&mut grid, stack).expect("the zone's own drops fit the grid it carries");
        }
        assert_eq!(grid.len(), FOES, "a drop went missing on the way in");
    }

    /// **The same seed leaves the same loot, and a different one does not.**
    /// The pair `apps/sparks/src/show.rs` makes of its show, and the reason the
    /// roll is a hash: without the second half, a build that ignored the seed
    /// entirely would pass the first.
    #[test]
    fn the_same_seed_drops_the_same_items_and_another_seed_differs() {
        let drops = |seed: u32| (0..FOES).map(|at| drop_of(seed, at)).collect::<Vec<_>>();
        assert_eq!(drops(DEFAULT_SEED), drops(DEFAULT_SEED), "the same seed");

        assert_ne!(
            drops(DEFAULT_SEED),
            drops(DEFAULT_SEED ^ 1),
            "one bit of the seed changed nothing about the loot",
        );

        // …and it is a spread rather than one seed that happens to differ: a
        // build that hashed only the foe index would give every seed the
        // identical list and reach exactly one.
        let mut hauls: Vec<Vec<(u16, u16)>> = (0..64u32)
            .map(|seed| {
                (0..FOES)
                    .map(|at| {
                        let stack = drop_of(seed, at);
                        (stack.item().0, stack.count())
                    })
                    .collect()
            })
            .collect();
        hauls.sort_unstable();
        hauls.dedup();
        assert!(
            hauls.len() > 8,
            "64 seeds produced {} distinct hauls",
            hauls.len(),
        );
    }

    /// **Two foes on one seed are rolled apart.** The control for the stride:
    /// a mixer fed the bare index would leave neighbouring foes correlated, and
    /// a roll that ignored the index would give every foe the same item on
    /// every seed.
    #[test]
    fn two_foes_on_one_seed_do_not_share_a_drop() {
        let together = (0..256u32)
            .filter(|seed| (0..FOES).all(|at| dropped(*seed, at) == dropped(*seed, 0)))
            .count();
        assert!(
            together < 32,
            "{together} of 256 seeds gave every foe the same item",
        );
    }

    /// **A stack names the foe that left it, and nothing else does.** What
    /// `crate::save`'s decoder rests on: an id outside the roster is one no
    /// session minted, so a payload carrying one is refused rather than read as
    /// an item that came from nowhere.
    #[test]
    fn a_stack_id_names_its_foe_and_a_stranger_names_none() {
        for index in 0..FOES {
            assert_eq!(foe_of(stack_id(index), FOES), Some(index));
            assert_eq!(drop_of(DEFAULT_SEED, index).id(), stack_id(index));
        }
        assert_eq!(foe_of(StackId(0), FOES), None, "no foe is zero");
        assert_eq!(
            foe_of(stack_id(FOES), FOES),
            None,
            "a fourth stack in a three-foe zone",
        );
    }

    /// **A drag lands where the hand let go, whatever part of the item it took
    /// hold of.** A build that moved the *origin* under the pointer would jump
    /// a `2×2` up and left by its own size on every drag that grabbed it
    /// anywhere but its top-left cell.
    #[test]
    fn a_drag_keeps_the_grip_it_started_with() {
        let origin = Cell::new(1, 1);
        // Grabbed by the top-left cell: the item goes exactly where it is
        // dropped.
        assert_eq!(
            dragged_origin(origin, origin, Cell::new(2, 3)),
            Some(Cell::new(2, 3)),
        );
        // Grabbed by the bottom-right of a 2x2 and dropped one cell right: the
        // origin follows by one, rather than landing on the drop.
        assert_eq!(
            dragged_origin(origin, Cell::new(2, 2), Cell::new(3, 2)),
            Some(Cell::new(2, 1)),
        );
        // …and a drop that would put the origin off the grid is refused rather
        // than wrapped.
        assert_eq!(
            dragged_origin(origin, Cell::new(2, 2), Cell::new(0, 0)),
            None
        );
    }
}
