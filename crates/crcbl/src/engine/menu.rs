//! The pause panel, which is the loop's menu rather than a game's.
//!
//! ```text
//!   ┌───────────────────────────┐
//!   │          PAUSED           │
//!   │  RESUME              ESC  │
//!   │  FULLSCREEN          F11  │
//!   │  DEBUG PANEL          F3  │
//!   └───────────────────────────┘
//! ```
//!
//! # Why this is not each sample's own
//!
//! Every row on it is the menu equivalent of one of [`PAUSE_KEY`](super::PAUSE_KEY),
//! [`FULLSCREEN_KEY`](super::FULLSCREEN_KEY) and
//! [`DEBUG_OVERLAY_KEY`](super::DEBUG_OVERLAY_KEY) — keys the loop acts on itself
//! and never hands a game. So the ids, the labels, the shortcuts and the order
//! are all the loop's, and a sample writing them out is writing down a fact
//! about its host. Seven of them did, character for character, and
//! `crates/crcbl-cli/templates/main.rs.tmpl` wrote it an eighth time into every
//! project `crcbl new` scaffolds.
//!
//! What stays with the sample is the state the panel belongs to — its
//! `MenuKind`, and the `paused` it maps from — because that is the only part
//! that differs between a demo with one menu and a demo with four.
//!
//! # Here rather than in `crcbl_ui::menu`
//!
//! [`Menu`] and [`MenuSet`] are the toolkit's: layout, selection and
//! activation, with no notion of what any button means. *Which* buttons a pause
//! panel has is [`MenuAction`](super::MenuAction)'s, and
//! [`RESUME_ID`] and its two neighbours live beside it — the
//! toolkit cannot see them, and giving it a "resume" would be the layer
//! boundary this crate's split exists to hold.

use crcbl_ui::menu::{Menu, MenuItem, MenuSet};

use super::{DEBUG_OVERLAY_ID, FULLSCREEN_ID, RESUME_ID};

/// The panel's heading.
///
/// Read back by every sample's browser row, which finds the panel by this
/// string.
pub const PAUSE_TITLE: &str = "PAUSED";

/// The three rows the loop owns, in the order they are drawn.
///
/// Separate from [`pause_menu`] for the sample that has a row of its own to put
/// among them: `apps/towers` inserts `RESTART` after `RESUME` rather than
/// appending it, so it needs the items before they become a [`Menu`].
#[must_use]
pub fn pause_items() -> Vec<MenuItem> {
    vec![
        MenuItem::new(RESUME_ID, "RESUME", "ESC"),
        MenuItem::new(FULLSCREEN_ID, "FULLSCREEN", "F11"),
        MenuItem::new(DEBUG_OVERLAY_ID, "DEBUG PANEL", "F3"),
    ]
}

/// The pause panel: [`PAUSE_TITLE`] over [`pause_items`].
#[must_use]
pub fn pause_menu() -> Menu {
    Menu::new(PAUSE_TITLE, pause_items())
}

/// A [`MenuSet`] holding [`pause_menu`] and nothing else.
///
/// `none` is the state that draws no menu at all and `paused` the one that
/// draws the panel — the two halves of a sample's `MenuKind`, or `false` and
/// `true` for a game keyed on the flag directly. `none` gets no entry, which is
/// how the set is told a running frame draws nothing.
#[must_use]
pub fn pause_only<K: Copy + Eq>(none: K, paused: K) -> MenuSet<K> {
    MenuSet::new(none, vec![(paused, pause_menu())])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{FIRST_GAME_ID, MenuAction};

    /// **Every button on the panel is one the loop owns**, in the order it
    /// draws them.
    ///
    /// The order is as load-bearing as the set: a sample's browser row drives
    /// this panel with the arrow keys and counts rows to reach one.
    #[test]
    fn the_panel_is_the_three_buttons_the_loop_owns() {
        let menu = pause_menu();
        assert_eq!(menu.title, PAUSE_TITLE);
        assert_eq!(
            menu.items()
                .iter()
                .map(|item| (item.id, item.label.as_str(), item.hint.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (RESUME_ID, "RESUME", "ESC"),
                (FULLSCREEN_ID, "FULLSCREEN", "F11"),
                (DEBUG_OVERLAY_ID, "DEBUG PANEL", "F3"),
            ],
        );
    }

    /// Nothing on it is numbered in a game's range, so
    /// [`MenuAction::from_id`] never has to ask a game about one — which is why
    /// a sample whose whole menu is this panel can declare its `MenuAction` as
    /// [`Infallible`](core::convert::Infallible).
    #[test]
    fn no_item_claims_an_id_a_game_would_have_to_answer_for() {
        for item in pause_menu().items() {
            assert!(
                item.id < FIRST_GAME_ID,
                "{} claims {}, which the game would have to name",
                item.label,
                item.id,
            );
            assert_eq!(
                MenuAction::from_id(item.id, |_| None::<core::convert::Infallible>),
                Some(match item.id {
                    RESUME_ID => MenuAction::Resume,
                    FULLSCREEN_ID => MenuAction::Fullscreen,
                    _ => MenuAction::DebugOverlay,
                }),
            );
        }
    }

    /// The set draws nothing until it is shown the paused state, which is what
    /// makes a running frame's menu absent rather than empty.
    #[test]
    fn a_pause_only_set_draws_nothing_until_the_paused_state_is_shown() {
        let mut menus = pause_only(false, true);
        assert!(!menus.is_showing(), "a running frame draws no menu");
        menus.show(true);
        assert_eq!(
            menus.current().expect("the paused state has a menu").title,
            PAUSE_TITLE,
        );
    }
}
