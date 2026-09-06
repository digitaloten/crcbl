//! Footprints: the bitmask a placement is checked against, and the four turns.

use serde::de::Error as _;
use serde::ser::SerializeSeq as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::InventoryError;

/// The side of the square a [`Shape`] is cut from, in cells.
///
/// Eight, because `8 * 8` is the width of the `u64` a footprint is one of. The
/// plan's widest container is the `5×10` backpack, so this is not a cap an item
/// meets.
pub const MAX_SHAPE: u8 = 8;

/// One cell, in whatever coordinate space the call is about: a [`Shape`]'s own
/// top-left origin, or a [`Grid`](crate::Grid)'s.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Cell {
    /// The column, counting right from zero.
    pub x: u8,
    /// The row, counting down from zero.
    pub y: u8,
}

impl Cell {
    /// The cell at column `x`, row `y`.
    #[must_use]
    pub const fn new(x: u8, y: u8) -> Self {
        Self { x, y }
    }
}

/// How far a footprint is turned, clockwise, from the way its definition
/// spells it.
///
/// Four arms rather than two: see the crate docs — the plan's involution is the
/// rectangle case, and a bitmask footprint admits an L.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Rotation {
    /// The way the definition spells it.
    #[default]
    Deg0,
    /// A quarter turn clockwise.
    Deg90,
    /// A half turn.
    Deg180,
    /// A quarter turn anticlockwise.
    Deg270,
}

impl Rotation {
    /// Every rotation, in the order auto-placement tries them.
    ///
    /// [`Grid::find_slot`](crate::Grid::find_slot) walks this array at each
    /// cell, so this order — and not some other permutation of the same four —
    /// is what makes first-fit a repeatable answer.
    pub const ALL: [Self; 4] = [Self::Deg0, Self::Deg90, Self::Deg180, Self::Deg270];

    /// This rotation turned one more quarter clockwise.
    #[must_use]
    pub const fn turned(self) -> Self {
        match self {
            Self::Deg0 => Self::Deg90,
            Self::Deg90 => Self::Deg180,
            Self::Deg180 => Self::Deg270,
            Self::Deg270 => Self::Deg0,
        }
    }
}

/// An item's footprint: which of an `8×8` field of cells it covers.
///
/// The mask is row-major, bit `y * 8 + x`, so the whole footprint is one `u64`
/// and every operation on it — turning it, testing it against a grid — is
/// arithmetic rather than a loop over a `Vec<bool>`.
///
/// A shape's [`width`](Self::width) and [`height`](Self::height) are its
/// bounding box, not its cell count: an L is `2×3` and covers four cells.
///
/// # Serde
///
/// A shape reads and writes as its rows — `["#.", "#.", "##"]` — and not as the
/// mask. The number is the representation; the rows are what a person editing
/// a catalogue can see the item in. `'#'` is a covered cell and every other
/// character is a hole, so `.` and a space both read as empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Shape {
    mask: u64,
    w: u8,
    h: u8,
}

impl Shape {
    /// A solid `w × h` block.
    ///
    /// # Errors
    ///
    /// [`InventoryError::ShapeTooLarge`] if either side is past [`MAX_SHAPE`],
    /// and [`InventoryError::ShapeEmpty`] if either is zero. Neither is
    /// clamped: a `9×1` footprint refused here is the `x == 8` bit that would
    /// otherwise wrap into row one.
    pub const fn rect(w: u8, h: u8) -> Result<Self, InventoryError> {
        if w > MAX_SHAPE || h > MAX_SHAPE {
            return Err(InventoryError::ShapeTooLarge { w, h });
        }
        if w == 0 || h == 0 {
            return Err(InventoryError::ShapeEmpty);
        }
        let row = (1u64 << w) - 1;
        let mut mask = 0u64;
        let mut y = 0;
        while y < h {
            mask |= row << (y * MAX_SHAPE);
            y += 1;
        }
        Ok(Self { mask, w, h })
    }

    /// A footprint drawn as text, one string per row, `'#'` for a covered cell.
    ///
    /// The width is the longest row's; a short row is padded with holes.
    ///
    /// # Errors
    ///
    /// [`InventoryError::ShapeTooLarge`] if the drawing is wider or taller than
    /// [`MAX_SHAPE`] (a dimension past 255 is reported as 255, which is as much
    /// as the error can hold and more than enough to say what went wrong), and
    /// [`InventoryError::ShapeEmpty`] if it has no `'#'` in it at all.
    pub fn from_rows(rows: &[&str]) -> Result<Self, InventoryError> {
        let height = rows.len();
        let width = rows
            .iter()
            .map(|row| row.chars().count())
            .max()
            .unwrap_or(0);
        if width > MAX_SHAPE as usize || height > MAX_SHAPE as usize {
            return Err(InventoryError::ShapeTooLarge {
                w: u8::try_from(width).unwrap_or(u8::MAX),
                h: u8::try_from(height).unwrap_or(u8::MAX),
            });
        }
        let mut mask = 0u64;
        for (y, row) in rows.iter().enumerate() {
            for (x, glyph) in row.chars().enumerate() {
                if glyph == '#' {
                    mask |= 1u64 << (y * MAX_SHAPE as usize + x);
                }
            }
        }
        if mask == 0 {
            return Err(InventoryError::ShapeEmpty);
        }
        Ok(Self {
            mask,
            w: width as u8,
            h: height as u8,
        })
    }

    /// This footprint turned `rotation` clockwise.
    ///
    /// The bounding box turns with it: a `1×2` at [`Rotation::Deg90`] is `2×1`.
    #[must_use]
    pub const fn rotated(self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::Deg0 => self,
            Rotation::Deg90 => self.turned_once(),
            Rotation::Deg180 => self.turned_once().turned_once(),
            Rotation::Deg270 => self.turned_once().turned_once().turned_once(),
        }
    }

    /// One quarter turn clockwise: cell `(x, y)` of a `w×h` footprint lands at
    /// `(h - 1 - y, x)` of the `h×w` one.
    ///
    /// The reversal is the whole of it. A bare transpose — `(x, y)` to
    /// `(y, x)` — is a *reflection*, and it makes [`Rotation::Deg90`] and
    /// [`Rotation::Deg270`] the same footprint, which for every rectangle in
    /// the shipped table is invisible.
    const fn turned_once(self) -> Self {
        let mut mask = 0u64;
        let mut y = 0;
        while y < self.h {
            let mut x = 0;
            while x < self.w {
                if self.mask & (1u64 << (y * MAX_SHAPE + x)) != 0 {
                    let nx = self.h - 1 - y;
                    mask |= 1u64 << (x * MAX_SHAPE + nx);
                }
                x += 1;
            }
            y += 1;
        }
        Self {
            mask,
            w: self.h,
            h: self.w,
        }
    }

    /// The bounding box's width, in cells.
    #[must_use]
    pub const fn width(self) -> u8 {
        self.w
    }

    /// The bounding box's height, in cells.
    #[must_use]
    pub const fn height(self) -> u8 {
        self.h
    }

    /// How many cells the footprint actually covers, which for anything but a
    /// rectangle is fewer than `width * height`.
    #[must_use]
    pub const fn count(self) -> u32 {
        self.mask.count_ones()
    }

    /// Whether the footprint covers cell `(x, y)` of its own bounding box.
    #[must_use]
    pub const fn covers(self, x: u8, y: u8) -> bool {
        x < self.w && y < self.h && self.mask & (1u64 << (y * MAX_SHAPE + x)) != 0
    }

    /// Every covered cell, row-major from the footprint's own origin.
    ///
    /// Allocates nothing: this walks the set bits of the mask.
    pub fn cells(self) -> impl Iterator<Item = Cell> {
        let mut mask = self.mask;
        core::iter::from_fn(move || {
            if mask == 0 {
                return None;
            }
            let bit = mask.trailing_zeros() as u8;
            mask &= mask - 1;
            Some(Cell::new(bit % MAX_SHAPE, bit / MAX_SHAPE))
        })
    }
}

impl Serialize for Shape {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.h as usize))?;
        let mut row = String::with_capacity(self.w as usize);
        for y in 0..self.h {
            row.clear();
            for x in 0..self.w {
                row.push(if self.covers(x, y) { '#' } else { '.' });
            }
            seq.serialize_element(&row)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Shape {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let rows = Vec::<String>::deserialize(deserializer)?;
        let borrowed: Vec<&str> = rows.iter().map(String::as_str).collect();
        Self::from_rows(&borrowed).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The L the crate docs argue four rotations for: `2×3`, covering the left
    /// column and the bottom row.
    fn ell() -> Shape {
        Shape::from_rows(&["#.", "#.", "##"]).expect("that is a 2x3 drawing")
    }

    /// **Four turns is the identity and one turn is not three.** The second
    /// half is the whole point: a `turned_once` written as a transpose —
    /// `(x, y)` to `(y, x)`, with the reversal forgotten — passes the first
    /// assertion, passes every rectangle in the shipped container table, and
    /// makes [`Rotation::Deg90`] and [`Rotation::Deg270`] the same footprint.
    /// Only a shape with no diagonal symmetry can tell the two apart, which is
    /// why this test is written on an L and not on the `1×2` pocket item.
    #[test]
    fn an_l_turned_four_times_is_the_shape_it_started_as() {
        let start = ell();
        let mut turned = start;
        for _ in 0..4 {
            turned = turned.turned_once();
        }
        assert_eq!(turned, start, "four quarter turns is no turn");
        assert_eq!(start.rotated(Rotation::Deg0), start);

        assert_ne!(
            start.rotated(Rotation::Deg90),
            start.rotated(Rotation::Deg270),
            "a quarter turn each way is the same shape only under a transpose"
        );
        assert_ne!(start.rotated(Rotation::Deg90), start, "an L is not square");
        assert_eq!(
            start.rotated(Rotation::Deg90).rotated(Rotation::Deg270),
            start,
            "a turn and its opposite cancel"
        );
    }

    /// **The turn is the one the docs draw**, cell for cell, so a sign flipped
    /// in `turned_once` fails here rather than somewhere a grid happened to
    /// have room either way.
    #[test]
    fn a_quarter_turn_puts_every_cell_where_the_drawing_says() {
        assert_eq!(
            ell().rotated(Rotation::Deg90),
            Shape::from_rows(&["###", "#.."]).expect("that is a 3x2 drawing"),
        );
        assert_eq!(ell().rotated(Rotation::Deg90).width(), 3);
        assert_eq!(ell().rotated(Rotation::Deg90).height(), 2);
    }

    /// **A footprint past the mask is refused, not truncated.** Nine cells in a
    /// row would set bit `y * 8 + 8`, which is bit `(y + 1) * 8` — the first
    /// cell of the *next* row. The refusal is what makes that unreachable, and
    /// the error carries the size asked for so a catalogue's author is told
    /// which item.
    #[test]
    fn a_shape_wider_than_the_mask_is_refused_rather_than_truncated() {
        assert_eq!(
            Shape::rect(MAX_SHAPE + 1, 1),
            Err(InventoryError::ShapeTooLarge { w: 9, h: 1 })
        );
        assert_eq!(
            Shape::rect(1, MAX_SHAPE + 1),
            Err(InventoryError::ShapeTooLarge { w: 1, h: 9 })
        );
        assert_eq!(
            Shape::from_rows(&["#########"]),
            Err(InventoryError::ShapeTooLarge { w: 9, h: 1 })
        );
        assert_eq!(Shape::rect(0, 2), Err(InventoryError::ShapeEmpty));
        assert_eq!(
            Shape::from_rows(&["..", ".."]),
            Err(InventoryError::ShapeEmpty)
        );

        let widest = Shape::rect(MAX_SHAPE, MAX_SHAPE).expect("8x8 is the mask exactly");
        assert_eq!(widest.count(), u32::from(MAX_SHAPE) * u32::from(MAX_SHAPE));
        assert_eq!(
            widest.cells().count(),
            widest.count() as usize,
            "every set bit is a cell the iterator yields"
        );
    }

    /// **Rows and mask say the same thing**, in both directions, so the serde
    /// form a catalogue is written in cannot drift from the bits a placement is
    /// tested against.
    #[test]
    fn a_drawing_and_its_cells_agree() {
        let shape = ell();
        assert_eq!(shape.width(), 2);
        assert_eq!(shape.height(), 3);
        assert_eq!(shape.count(), 4);
        let cells: Vec<Cell> = shape.cells().collect();
        assert_eq!(
            cells,
            vec![
                Cell::new(0, 0),
                Cell::new(0, 1),
                Cell::new(0, 2),
                Cell::new(1, 2)
            ],
            "row-major, from the footprint's own origin"
        );
        assert!(shape.covers(1, 2));
        assert!(!shape.covers(1, 0));
        assert!(
            !shape.covers(2, 0),
            "outside the bounding box is not covered"
        );
    }
}
