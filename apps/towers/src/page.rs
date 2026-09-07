//! The overlay: the two shared numbers, the wave, the build list, and the
//! banner a finished run wears.
//!
//! ```text
//!  ┌ towers ──────────┐
//!  │ GOLD         80  │
//!  │ LIVES     12/12  │
//!  │ WAVE        1/3  │
//!  │ NEXT      2.4 s  │
//!  │ CREEPS        3  │
//!  │ KILLS         5  │
//!  │ LEAKS         0  │
//!  └──────────────────┘
//!  │ ENTRY       BUILT│
//!  │ BEND         40g │
//!  │>EAST         40g │
//!  │ MIDDLE       40g │
//!  │ GATE         40g │
//!  └──────────────────┘
//!
//!     LEFT/RIGHT pick a plot   B builds   N sends the wave   R restarts
//! ```
//!
//! # The build list is the one thing on screen a player reads before acting
//!
//! It is the whole of the interface `PlaceTower` has: five rows, the highlighted
//! one is what `B` builds on, and a row already built says so rather than
//! showing a price the server would refuse. That last part is the client
//! agreeing with the server rather than replacing it — the refusal still
//! happens in `crate::game::Stage::place_tower`, and the debug panel's
//! `refused` row is where a disagreement between the two would show up.
//!
//! # Rule 11 is owed, and by more than this panel
//!
//! No `.crpix` art anywhere: the tower and creep icons, the wave banner and the
//! build menu that `docs/plan/sample/07-towers.md` asks for are all still
//! untextured rectangles and the engine's built-in font. This sample is not
//! claiming rule 11's exemption — a tower defense is exactly the kind of game
//! that should have pixel art — it simply has not got there yet, and that
//! document's status section says so.
//!
//! # One column, because the debug panel owns the other one
//!
//! Both panels stack down the left edge and nothing is drawn against the right.
//! The debug panel is on by default here (sample rule 1) and it is tall and
//! right-aligned, so a build list in the top right is a build list a player
//! reads through a wall of frame timings — which is what the first screenshot
//! of this sample showed. `the_page_keeps_to_its_own_column` is what holds the
//! two apart.
//!
//! # Laid out against the surface
//!
//! Every position is derived from the extent the swapchain was actually
//! acquired at, so the page is correct in a resized window and in the headless
//! offscreen ring at whatever `--size` asked for.

use crcbl::math::Vec2;
use crcbl::ui::draw_list::DrawList;
use crcbl::ui::readout::{NATURAL_SCALE, ReadoutPanel, ReadoutRow};
use crcbl::ui::text::FontAtlas;
use crcbl::ui::widget::NATURAL_FONT_SIZE;

use crate::game::RenderState;
use crate::map::PLOTS;
use crate::tower::COST;
use crate::wave::{Outcome, STARTING_LIVES, WAVES};

const PANEL_BG: [f32; 4] = [0.06, 0.09, 0.07, 0.80];
const BORDER: [f32; 4] = [0.34, 0.44, 0.36, 1.0];
const LABEL: [f32; 4] = [0.68, 0.76, 0.68, 1.0];
const VALUE: [f32; 4] = [0.95, 0.98, 0.94, 1.0];
/// What a reading in trouble is drawn in — the lives once they are down to a
/// third, and a plot nobody can afford.
const WARN: [f32; 4] = [0.95, 0.55, 0.36, 1.0];
/// What the highlighted build row and a won run are drawn in.
const PICKED: [f32; 4] = [0.55, 0.92, 0.62, 1.0];
/// What a lost run is drawn in.
const LOST: [f32; 4] = [0.95, 0.40, 0.36, 1.0];

/// The readout panel: this page's geometry and palette, over
/// [`crcbl::ui::readout`]'s layout.
const READOUT: ReadoutPanel = ReadoutPanel {
    inset: 16.0,
    width: 176.0,
    row_height: 18.0,
    pad: 8.0,
    border_width: 1.0,
    background: PANEL_BG,
    border: BORDER,
    label: LABEL,
};

/// The build panel, which stands under the readout and is narrower because its
/// rows are a plot's name and a price rather than a reading.
const BUILD: ReadoutPanel = ReadoutPanel {
    width: 168.0,
    ..READOUT
};

/// What marks the highlighted row of the build list.
///
/// ASCII, and every other glyph on this page is too: the built-in atlas has no
/// arrows in it, so `\u{25b6}` and its friends draw as the missing-glyph box —
/// which on a five-row list is a marker a reader cannot tell from a bullet.
const MARKER: &str = ">";

/// The control hint, which is the whole of what a first-time visitor needs.
const HINT: &str = "LEFT/RIGHT pick a plot   B builds   N sends the wave   R restarts";

/// What the page drew, for the loop's own tests and its summary line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageStats {
    /// How many draw commands the page produced.
    pub commands: usize,
}

/// The readout panel's rows.
fn readout(state: &RenderState) -> Vec<ReadoutRow> {
    vec![
        ReadoutRow::new("GOLD", format!("{}", state.gold), VALUE),
        ReadoutRow::new(
            "LIVES",
            format!("{}/{STARTING_LIVES}", state.lives),
            if state.lives * 3 <= STARTING_LIVES {
                WARN
            } else {
                VALUE
            },
        ),
        ReadoutRow::new("WAVE", format!("{}/{}", state.wave, WAVES.len()), VALUE),
        ReadoutRow::new(
            "NEXT",
            match state.next_wave_in {
                Some(seconds) => format!("{seconds:.1} s"),
                None => "--".into(),
            },
            VALUE,
        ),
        ReadoutRow::new("CREEPS", format!("{}", state.creeps_alive), VALUE),
        ReadoutRow::new("KILLS", format!("{}", state.kills), VALUE),
        ReadoutRow::new(
            "LEAKS",
            format!("{}", state.leaks),
            if state.leaks > 0 { WARN } else { VALUE },
        ),
    ]
}

/// The build panel's rows: one per plot, the highlighted one marked.
fn build_list(state: &RenderState, selected: u8) -> Vec<ReadoutRow> {
    PLOTS
        .iter()
        .enumerate()
        .map(|(plot, at)| {
            let picked = plot == usize::from(selected);
            let label = format!(
                "{}{}",
                if picked { MARKER } else { " " },
                at.label.to_uppercase(),
            );
            let (reading, colour) = if state.towers[plot].is_some() {
                ("BUILT".to_string(), PICKED)
            } else if state.gold < COST {
                (format!("{COST}g"), WARN)
            } else {
                (format!("{COST}g"), VALUE)
            };
            ReadoutRow::new(label, reading, colour)
        })
        .collect()
}

/// Draws the overlay into `list`, laid out against a surface of `extent`.
///
/// `atlas` is only measured against — the glyphs themselves are the UI pass's
/// business — and it is what right-aligns the readings against a proportional
/// font rather than against a guess; see [`ReadoutPanel::draw_at`].
pub fn draw(
    list: &mut DrawList,
    atlas: &FontAtlas,
    extent: (u32, u32),
    state: &RenderState,
    selected: u8,
) -> PageStats {
    let width = extent.0 as f32;
    let height = extent.1 as f32;

    let readout = readout(state);
    let build = build_list(state, selected);
    READOUT.draw(list, atlas, &readout);
    BUILD.draw_at(
        list,
        atlas,
        Vec2::new(
            READOUT.inset,
            2.0f32.mul_add(READOUT.inset, READOUT.height(readout.len())),
        ),
        &build,
    );

    // The banner, which is the one thing on screen a player who has stopped
    // watching the field still reads.
    if let Some((text, colour)) = match state.outcome {
        Outcome::Playing => None,
        Outcome::Won => Some(("EVERY WAVE HELD", PICKED)),
        Outcome::Lost => Some(("OVERRUN", LOST)),
    } {
        let banner = atlas.text_width(text, NATURAL_SCALE);
        list.text(
            Vec2::new((width - banner) * 0.5, height * 0.5 - READOUT.row_height),
            text.to_string(),
            colour,
            NATURAL_FONT_SIZE,
        );
    }

    READOUT.hint(list, atlas, extent, HINT);

    PageStats {
        commands: list.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crcbl::ui::draw_list::DrawCommand;

    /// A window the page is laid out against.
    const EXTENT: (u32, u32) = (960, 720);

    /// Every `Text` command the page produced.
    fn text(list: &DrawList) -> Vec<String> {
        list.commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    /// A run part way through the second wave with two towers up.
    fn playing() -> RenderState {
        RenderState {
            gold: 64,
            lives: 9,
            wave: 2,
            kills: 5,
            leaks: 3,
            creeps_alive: 3,
            next_wave_in: None,
            towers: [Some(false), Some(true), None, None, None],
            ..RenderState::default()
        }
    }

    /// **Every number the game turns on is on the page**, and every plot has a
    /// row. A readout that dropped one is a player who cannot see why a build
    /// was refused.
    #[test]
    fn the_readout_carries_every_number_a_player_plays_on() {
        let mut list = DrawList::new();
        let state = playing();
        let stats = draw(&mut list, &FontAtlas::built_in(), EXTENT, &state, 2);
        assert_eq!(stats.commands, list.len());
        assert!(stats.commands > 0, "the page drew nothing");

        let text = text(&list);
        for expected in [
            "GOLD".to_string(),
            "64".to_string(),
            format!("{}/{STARTING_LIVES}", state.lives),
            format!("{}/{}", state.wave, WAVES.len()),
            "5".to_string(),
            "3".to_string(),
        ] {
            assert!(text.contains(&expected), "{expected} is not on the page");
        }
        for plot in PLOTS {
            assert!(
                text.iter()
                    .any(|drawn| drawn.contains(&plot.label.to_uppercase())),
                "{} has no row on the build list",
                plot.label,
            );
        }
        assert!(
            text.iter().any(|drawn| drawn == "BUILT"),
            "a built plot is priced"
        );
        assert!(
            text.contains(&HINT.to_string()),
            "the control hint is missing"
        );
    }

    /// **The highlighted plot is the one marked**, and only that one — the
    /// marker is the whole of what says which plot `B` will build on.
    #[test]
    fn exactly_the_selected_plot_is_marked() {
        for selected in 0..PLOTS.len() as u8 {
            let mut list = DrawList::new();
            draw(
                &mut list,
                &FontAtlas::built_in(),
                EXTENT,
                &RenderState::default(),
                selected,
            );
            let marked: Vec<String> = text(&list)
                .into_iter()
                .filter(|drawn| drawn.starts_with(MARKER))
                .collect();
            assert_eq!(marked.len(), 1, "{marked:?} rows are marked");
            assert!(
                marked[0].contains(&PLOTS[usize::from(selected)].label.to_uppercase()),
                "the marker is on {marked:?} rather than on {}",
                PLOTS[usize::from(selected)].label,
            );
        }
    }

    /// **The player's panels keep to the left column**, which is the half of
    /// the surface the debug panel does not take. A build list drawn in the top
    /// right is legible in a test and unreadable on screen, and only a position
    /// assertion can tell the two apart.
    #[test]
    fn the_page_keeps_to_its_own_column() {
        let mut list = DrawList::new();
        draw(&mut list, &FontAtlas::built_in(), EXTENT, &playing(), 2);
        let middle = EXTENT.0 as f32 * 0.5;
        let mut panels = 0;
        for command in list.commands() {
            let DrawCommand::Rect { min, max, .. } = command else {
                continue;
            };
            panels += 1;
            assert!(
                max.x < middle,
                "a panel runs from {} to {}, across the middle at {middle}",
                min.x,
                max.x,
            );
        }
        assert_eq!(panels, 2, "the page drew {panels} panels rather than two");
    }

    /// **A finished run wears its banner, and a running one does not.** The
    /// second half is the control: a page that always drew one would say
    /// `OVERRUN` over a field that is doing fine.
    #[test]
    fn a_finished_run_says_so_and_a_running_one_does_not() {
        let banner_of = |outcome: Outcome| {
            let mut list = DrawList::new();
            draw(
                &mut list,
                &FontAtlas::built_in(),
                EXTENT,
                &RenderState {
                    outcome,
                    ..playing()
                },
                0,
            );
            text(&list)
        };
        assert!(
            banner_of(Outcome::Won)
                .iter()
                .any(|drawn| drawn == "EVERY WAVE HELD")
        );
        assert!(
            banner_of(Outcome::Lost)
                .iter()
                .any(|drawn| drawn == "OVERRUN")
        );
        for drawn in banner_of(Outcome::Playing) {
            assert_ne!(drawn, "EVERY WAVE HELD");
            assert_ne!(drawn, "OVERRUN");
        }
    }
}
