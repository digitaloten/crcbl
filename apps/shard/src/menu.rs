//! Shard's one menu: the pause panel.
//!
//! ```text
//!   paused ──▶ Paused
//!   running ─▶ none
//! ```
//!
//! # Every button on it belongs to the loop
//!
//! Resume, fullscreen and the debug panel are the menu equivalents of the
//! engine's three reserved keys and live in [`crcbl::engine::MenuAction`], so
//! this sample declares no action of its own — [`crate::app::Shard`]'s
//! `menu_action` answers `None` for every id it is asked about, and its
//! `MenuAction` type is [`core::convert::Infallible`] because there is genuinely
//! no value it could ever be handed.
//!
//! There is nothing else to put on it yet. Walking, turning the camera,
//! striking, taking what is on the floor, opening the pack and putting the
//! torches out are all keys rather than rows, and the save is not a row either
//! — [`crate::save`] writes on a cadence the simulation owns, so there is no
//! button to press and nothing for a player to remember. The inventory is a
//! panel of its own ([`crate::panel`], on `I`) rather than a menu, because a
//! menu here stops the zone and looking at what you are carrying should not.
//! A character sheet is the row this menu will eventually want, and it belongs
//! to the *level* verb, which is the one part of
//! `docs/plan/sample/15-shard.md`'s milestone 1 that is not built.

use crcbl::engine::pause_only;
use crcbl::ui::menu::MenuSet;

/// Which menu a frame shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuKind {
    /// The zone is being walked: no menu at all.
    #[default]
    None,
    /// The loop has stopped ticking.
    Paused,
}

impl MenuKind {
    /// The menu this frame shows.
    #[must_use]
    pub const fn of(paused: bool) -> Self {
        if paused { Self::Paused } else { Self::None }
    }
}

/// Shard's menus, keyed by the state each belongs to.
pub type Menus = MenuSet<MenuKind>;

/// The one menu, with nothing shown while the demo runs.
#[must_use]
pub fn menus() -> Menus {
    pause_only(MenuKind::None, MenuKind::Paused)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pause is the only thing that puts a panel on screen here, and it always
    /// does — this sample has no other state a menu could belong to.
    ///
    /// **What is on the panel is not asserted here.** Every row of it is
    /// [`crcbl::engine::pause_menu`]'s, and the claims that used to be written
    /// out in each sample — the title, the three ids, and that none of them is
    /// one this game would have to answer for — are beside it in
    /// [`crcbl::engine::menu`].
    #[test]
    fn the_pause_menu_is_shown_exactly_while_the_loop_is_paused() {
        assert_eq!(MenuKind::of(true), MenuKind::Paused);
        assert_eq!(MenuKind::of(false), MenuKind::None);

        let mut menus = menus();
        assert!(!menus.is_showing(), "a running frame draws no menu");
        menus.show(MenuKind::Paused);
        assert!(menus.is_showing(), "the paused kind has a menu");
    }
}
