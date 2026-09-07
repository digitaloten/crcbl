//! The readout panel a 3D sample draws its numbers in.
//!
//! ```text
//!  ┌────────────────────┐
//!  │ POSITION  0.0 18.0 │   ← a label at the left margin, a reading
//!  │ HEALTH    100/100  │     right-aligned against the right one
//!  │ TORCHES   LIT      │
//!  └────────────────────┘
//!
//!         W/A/S/D walk   Q/E turn   SPACE strikes      ← the centred hint
//! ```
//!
//! # Geometry only, and the rows are the caller's
//!
//! A [`ReadoutPanel`] is an inset, a width, a row height, a padding and three
//! colours. It formats nothing and decides nothing: a sample builds its own
//! [`ReadoutRow`]s, in its own order, with its own colour per row, and this
//! draws them. That is what [`crate::hud::HudPanel`] could not do for these
//! pages — it sizes itself to its contents and its labels carry no colour —
//! and it is the whole of what five samples were each writing out.
//!
//! # Laid out against the surface
//!
//! [`ReadoutPanel::hint`] takes the extent the swapchain was actually acquired
//! at rather than a size the page assumed, so a hint is centred in a resized
//! window and in a headless offscreen ring at whatever size was asked for. The
//! panel itself hangs off its own corner and needs no extent at all.

use glam::Vec2;

use crate::draw_list::DrawList;
use crate::text::FontAtlas;
use crate::widget::NATURAL_FONT_SIZE;

/// The scale [`FontAtlas::text_width`] is measured at, which is a multiplier on
/// the baked glyph size rather than a size in pixels.
///
/// A panel draws at the font's natural size, so it measures at the natural
/// scale. Public because a page's own tests measure the strings it emitted the
/// same way the layout did — measuring at a different scale would assert about
/// a panel nothing drew.
pub const NATURAL_SCALE: f32 = 1.0;

/// One row: a label at the panel's left margin and a reading right-aligned
/// against its right one.
///
/// The value is already formatted — the panel does no formatting of its own,
/// on [`crate::debug::DebugRow`]'s terms — and the colour is the row's rather
/// than the panel's, because the one piece of state each of these pages puts a
/// colour on is a reading and not a label.
#[derive(Clone, Debug, PartialEq)]
pub struct ReadoutRow {
    /// What the number is.
    pub label: String,
    /// The number, already formatted.
    pub value: String,
    /// What the reading is drawn in.
    pub colour: [f32; 4],
}

impl ReadoutRow {
    /// A row.
    #[must_use]
    pub fn new(label: impl Into<String>, value: impl Into<String>, colour: [f32; 4]) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            colour,
        }
    }
}

/// Where a readout panel sits, how wide it is, and what it is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReadoutPanel {
    /// The panel's inset from the corner of the surface, in pixels — and the
    /// bottom margin [`ReadoutPanel::hint`] leaves.
    pub inset: f32,
    /// How wide the panel is, in pixels. Wide enough for the longest label and
    /// a right-aligned reading beside it, which is a fact about the rows a
    /// sample puts in it rather than one this crate can know.
    pub width: f32,
    /// The height of one row, in pixels.
    pub row_height: f32,
    /// The panel's padding inside its own border, in pixels.
    pub pad: f32,
    /// How thick the panel's border is, in pixels.
    pub border_width: f32,
    /// What the panel is filled with.
    pub background: [f32; 4],
    /// What its border is drawn in.
    pub border: [f32; 4],
    /// What a label — and a hint — is drawn in. A reading's colour is the
    /// row's; see [`ReadoutRow::colour`].
    pub label: [f32; 4],
}

impl ReadoutPanel {
    /// How tall a panel of `rows` rows stands, in pixels.
    ///
    /// What a page stacking a second panel under the first offsets it by.
    #[must_use]
    pub fn height(&self, rows: usize) -> f32 {
        2.0f32.mul_add(self.pad, rows as f32 * self.row_height)
    }

    /// Draws the panel in the top-left corner, [`ReadoutPanel::inset`] from
    /// both edges.
    pub fn draw(&self, list: &mut DrawList, atlas: &FontAtlas, rows: &[ReadoutRow]) {
        self.draw_at(list, atlas, Vec2::new(self.inset, self.inset), rows);
    }

    /// Draws it at `origin` instead, for a page that stacks two of them.
    ///
    /// `atlas` is only measured against — the glyphs themselves are the UI
    /// pass's business — and it is what right-aligns the readings against a
    /// proportional font rather than against a guess. Its measurements take a
    /// scale relative to the baked glyph size, not a pixel size, so everything
    /// here is drawn at [`NATURAL_FONT_SIZE`] and measured at
    /// [`NATURAL_SCALE`].
    pub fn draw_at(
        &self,
        list: &mut DrawList,
        atlas: &FontAtlas,
        origin: Vec2,
        rows: &[ReadoutRow],
    ) {
        let max = Vec2::new(origin.x + self.width, origin.y + self.height(rows.len()));
        list.rect(origin, max, self.background);
        list.rect_outline(origin, max, self.border_width, self.border);

        for (index, row) in rows.iter().enumerate() {
            let y = origin.y + self.pad + index as f32 * self.row_height;
            list.text(
                Vec2::new(origin.x + self.pad, y),
                row.label.clone(),
                self.label,
                NATURAL_FONT_SIZE,
            );
            let reading = atlas.text_width(&row.value, NATURAL_SCALE);
            list.text(
                Vec2::new(max.x - self.pad - reading, y),
                row.value.clone(),
                row.colour,
                NATURAL_FONT_SIZE,
            );
        }
    }

    /// Draws `text` centred along the bottom of a surface of `extent`, one row
    /// above [`ReadoutPanel::inset`].
    ///
    /// The control hint, which on these pages is the whole of what a first-time
    /// visitor needs. Its text is the sample's; where it lands is the panel's,
    /// so a page and its readout keep one margin between them.
    pub fn hint(&self, list: &mut DrawList, atlas: &FontAtlas, extent: (u32, u32), text: &str) {
        let width = extent.0 as f32;
        let height = extent.1 as f32;
        let measured = atlas.text_width(text, NATURAL_SCALE);
        list.text(
            Vec2::new(
                (width - measured) * 0.5,
                height - self.inset - self.row_height,
            ),
            text,
            self.label,
            NATURAL_FONT_SIZE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw_list::DrawCommand;

    const PANEL: ReadoutPanel = ReadoutPanel {
        inset: 16.0,
        width: 180.0,
        row_height: 18.0,
        pad: 8.0,
        border_width: 1.0,
        background: [0.06, 0.07, 0.11, 0.80],
        border: [0.34, 0.38, 0.48, 1.0],
        label: [0.66, 0.70, 0.80, 1.0],
    };

    const READING: [f32; 4] = [0.95, 0.96, 1.0, 1.0];

    fn rows() -> Vec<ReadoutRow> {
        vec![
            ReadoutRow::new("SHORT", "1", READING),
            ReadoutRow::new("A LONGER LABEL", "-12.75 m", READING),
        ]
    }

    /// **A reading is right-aligned against the panel's own inner edge**, which
    /// is the arithmetic every one of these pages was writing out for itself:
    /// the value's measured width comes off the right margin, so a wider
    /// reading starts further left and both end on the same column.
    #[test]
    fn a_reading_ends_at_the_panels_right_margin_whatever_it_says() {
        let atlas = FontAtlas::built_in();
        let mut list = DrawList::new();
        let rows = rows();
        PANEL.draw(&mut list, &atlas, &rows);

        let drawn: Vec<(Vec2, String)> = list
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { pos, text, .. } => Some((*pos, text.clone())),
                _ => None,
            })
            .collect();
        assert_eq!(drawn.len(), rows.len() * 2, "a label and a reading a row");

        let right = PANEL.inset + PANEL.width - PANEL.pad;
        for row in &rows {
            let (pos, _) = drawn
                .iter()
                .find(|(_, text)| *text == row.value)
                .unwrap_or_else(|| panic!("{} was not drawn", row.value));
            let end = pos.x + atlas.text_width(&row.value, NATURAL_SCALE);
            assert!(
                (end - right).abs() < 1e-3,
                "{} ends at {end} rather than the panel's {right}",
                row.value,
            );
        }
    }

    /// **A label sits at the left margin and a row is one `row_height` below
    /// the last**, so a panel of any length keeps its own grid.
    #[test]
    fn every_row_stands_one_row_height_below_the_one_above_it() {
        let atlas = FontAtlas::built_in();
        let mut list = DrawList::new();
        let rows = rows();
        PANEL.draw(&mut list, &atlas, &rows);

        let labels: Vec<Vec2> = list
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { pos, text, .. }
                    if rows.iter().any(|row| row.label == *text) =>
                {
                    Some(*pos)
                }
                _ => None,
            })
            .collect();
        assert_eq!(labels.len(), rows.len());
        for (index, pos) in labels.iter().enumerate() {
            assert!((pos.x - (PANEL.inset + PANEL.pad)).abs() < 1e-3, "{pos:?}");
            let want = PANEL.inset + PANEL.pad + index as f32 * PANEL.row_height;
            assert!((pos.y - want).abs() < 1e-3, "row {index} at {pos:?}");
        }
    }

    /// **The box is as tall as its rows say**, which is what a page stacking a
    /// second panel under the first offsets it by.
    #[test]
    fn the_box_is_as_tall_as_the_rows_it_holds() {
        let atlas = FontAtlas::built_in();
        let mut list = DrawList::new();
        let rows = rows();
        PANEL.draw(&mut list, &atlas, &rows);

        let (min, max) = list
            .commands()
            .iter()
            .find_map(|command| match command {
                DrawCommand::Rect { min, max, .. } => Some((*min, *max)),
                _ => None,
            })
            .expect("the panel drew no box");
        assert_eq!(min, Vec2::new(PANEL.inset, PANEL.inset));
        assert_eq!(max.x, PANEL.inset + PANEL.width);
        assert!((max.y - (PANEL.inset + PANEL.height(rows.len()))).abs() < 1e-3);
    }

    /// **The hint is centred on the surface it was handed**, not on one the
    /// page assumed.
    #[test]
    fn the_hint_is_centred_on_whatever_surface_it_was_given() {
        let atlas = FontAtlas::built_in();
        for extent in [(960u32, 720u32), (1920, 1080)] {
            let mut list = DrawList::new();
            PANEL.hint(&mut list, &atlas, extent, "W/A/S/D walk");
            let pos = list
                .commands()
                .iter()
                .find_map(|command| match command {
                    DrawCommand::Text { pos, .. } => Some(*pos),
                    _ => None,
                })
                .expect("the hint drew nothing");
            let measured = atlas.text_width("W/A/S/D walk", NATURAL_SCALE);
            assert!(
                (pos.x + measured * 0.5 - extent.0 as f32 * 0.5).abs() < 1e-3,
                "the hint is off centre on a {extent:?} surface: {pos:?}"
            );
            assert!(
                (pos.y - (extent.1 as f32 - PANEL.inset - PANEL.row_height)).abs() < 1e-3,
                "the hint is not one row above the bottom margin: {pos:?}"
            );
        }
    }
}
