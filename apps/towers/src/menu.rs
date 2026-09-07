//! Towers' one menu: the pause panel.
//!
//! ```text
//!   paused ──▶ Paused
//!   running ─▶ none
//! ```
//!
//! # One row on it is this game's, and the rest belong to the loop
//!
//! Resume, fullscreen and the debug panel are the menu equivalents of the
//! engine's three reserved keys and live in [`crcbl::engine::MenuAction`].
//! `RESTART` is towers' own, so it takes an id from
//! [`FIRST_GAME_ID`] upward and
//! [`crate::app::Towers`] answers for it — which is the shape a game with a
//! menu action of its own has, and the thing `apps/breach`'s uninhabited
//! `MenuAction` could not demonstrate.
//!
//! **It is the same restart the `R` key sends**, and it crosses the wire the
//! same way: the menu sets a flag, the next tick seals it into a command and
//! the server throws the run away. A menu that reached into the stage would be
//! a client mutating server state, which rule 2 has no exemption for.

use crcbl::engine::{FIRST_GAME_ID, PAUSE_TITLE, pause_items};
use crcbl::ui::WidgetId;
use crcbl::ui::menu::{Menu, MenuItem, MenuSet};

/// The one widget id this game answers for.
pub const RESTART_ID: WidgetId = FIRST_GAME_ID;

/// Where `RESTART` sits among [`crcbl::engine::pause_items`]' three rows:
/// directly under `RESUME`.
const RESTART_ROW: usize = 1;

/// What only towers' menu does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    /// Throw the run away and start again.
    Restart,
}

/// Which menu a frame shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuKind {
    /// The field is being played: no menu at all.
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

/// Towers' menus, keyed by the state each belongs to.
pub type Menus = MenuSet<MenuKind>;

/// The one menu, with nothing shown while the field is being played.
#[must_use]
pub fn menus() -> Menus {
    // Inserted rather than appended: `RESTART` belongs directly under `RESUME`,
    // which is where a player who just lost reaches for it, and the arrow keys
    // walk the rows in this order.
    let mut items = pause_items();
    items.insert(RESTART_ROW, MenuItem::new(RESTART_ID, "RESTART", "R"));
    MenuSet::new(
        MenuKind::None,
        vec![(MenuKind::Paused, Menu::new(PAUSE_TITLE, items))],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crcbl::engine::{DEBUG_OVERLAY_ID, FULLSCREEN_ID, HostedGame as _, RESUME_ID};

    /// Pause is the only thing that puts a panel on screen here, and it always
    /// does — this sample has no other state a menu could belong to.
    #[test]
    fn the_pause_menu_is_shown_exactly_while_the_loop_is_paused() {
        assert_eq!(MenuKind::of(true), MenuKind::Paused);
        assert_eq!(MenuKind::of(false), MenuKind::None);

        let mut menus = menus();
        assert!(!menus.is_showing(), "a running frame draws no menu");
        menus.show(MenuKind::Paused);
        let menu = menus.current().expect("the paused kind has a menu");
        assert_eq!(menu.title, "PAUSED");
        assert_eq!(
            menu.items().iter().map(|item| item.id).collect::<Vec<_>>(),
            vec![RESUME_ID, RESTART_ID, FULLSCREEN_ID, DEBUG_OVERLAY_ID],
            "RESTART sits under RESUME, and the loop's three rows keep their order",
        );
    }

    /// **Every id on the menu resolves to exactly one action**, and the one
    /// this game claims is the only one it answers for.
    ///
    /// The second half is what an id collision would break: a game claiming a
    /// number the loop already owns would shadow `RESUME`, and a paused demo
    /// nobody can resume is a demo nobody can leave.
    #[test]
    fn the_game_answers_for_its_own_id_and_no_others() {
        // Constant on both sides, so the compiler is what refuses an id the
        // loop already owns.
        const { assert!(RESTART_ID >= FIRST_GAME_ID, "RESTART claims a reserved id") };
        assert_eq!(
            crate::app::Towers::menu_action(RESTART_ID),
            Some(MenuAction::Restart),
        );
        for id in [RESUME_ID, FULLSCREEN_ID, DEBUG_OVERLAY_ID] {
            assert_eq!(
                crate::app::Towers::menu_action(id),
                None,
                "the game claimed {id}, which the loop owns",
            );
            assert!(
                crcbl::engine::MenuAction::from_id(id, crate::app::Towers::menu_action).is_some(),
                "{id} resolves to nothing at all",
            );
        }
    }
}
