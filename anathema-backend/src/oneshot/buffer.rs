#![deny(missing_docs)]
use std::io::{Result, Write};
use std::ops::Index;

use anathema_geometry::Size;
use anathema_widgets::paint::{Glyph, GlyphMap};
use anathema_widgets::Style;
use crossterm::style::Print;
use crossterm::{cursor, QueueableCommand};

use super::screen::DrawCommand;
use super::style::write_style;
use super::LocalPos;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Cell {
    pub(crate) style: Style,
    pub(crate) state: CellState,
}

impl Cell {
    pub(crate) fn empty() -> Self {
        // It's important to reset the colours as there
        // might be residual colors from previous draw.
        Self {
            style: Style::reset(),
            state: CellState::Empty,
        }
    }

    pub(crate) fn reset() -> Self {
        Self {
            style: Style::reset(),
            state: CellState::Occupied(Glyph::space()),
        }
    }

    fn continuation(style: Style) -> Self {
        Self {
            style,
            state: CellState::Continuation,
        }
    }

    pub(crate) fn new(glyph: Glyph, style: Style) -> Self {
        Self {
            style,
            state: CellState::Occupied(glyph),
        }
    }
}

/// Represent the state of a cell inside a [`Buffer`].
#[derive(Copy, Clone, PartialEq)]
pub(crate) enum CellState {
    /// Empty
    Empty,
    /// Occupied by a certain character
    Occupied(Glyph),
    /// A continuation means this cell is part of another cell
    /// representing a value that spans more than two chars, e.g 💖
    Continuation,
}

impl std::fmt::Debug for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellState::Empty => write!(f, "<E>"),
            CellState::Occupied(Glyph::Single(glyph, _width)) => write!(f, "{glyph}"),
            CellState::Occupied(Glyph::Cluster(index, width)) => write!(f, "<C{index:?},{width}>"),
            CellState::Continuation => write!(f, "<C>"),
        }
    }
}

/// A buffer contains a list of cells representing characters that can be rendered.
/// This doesn't necessarily have to be `stdout`, it can be anything that implements
/// [`std::io::Write`]
///
/// The [`crate::Screen`] writes all the chars and their styles to the buffer, which works like a
/// grid.
#[derive(Debug, Clone)]
pub struct Buffer {
    size: Size,
    pub(crate) lines: Vec<Box<[Cell]>>,
}

impl Buffer {
    /// Crate a new `Buffer` with a given size.
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self { lines: vec![], size }
    }

    /// Create a new buffer with reset cells
    pub(crate) fn reset(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self { lines: vec![], size }
    }

    /// The size of the `Buffer`
    pub fn size(&self) -> Size {
        self.size
    }

    fn new_line(&mut self) {
        self.lines
            .push(vec![Cell::empty(); self.size.width as usize].into_boxed_slice());
    }

    fn ensure_storage(&mut self, pos: LocalPos) {
        // TODO: panic if the y value is too large

        while self.lines.len() <= pos.y as usize {
            self.new_line();
        }
    }

    /// Put a character with a style at a given position.
    pub fn put_glyph(&mut self, glyph: Glyph, pos: LocalPos) {
        assert!(
            pos.x < self.size.width && pos.y < self.size.height,
            "position out of bounds"
        );

        let style = match self.get(pos) {
            Some((_, style)) => *style,
            None => Style::new(),
        };

        let cell = Cell::new(glyph, style);
        self.put(cell, pos);
    }

    /// Update the attributes at a given cell.
    /// If there is no character at that cell, then write an empty space into it
    pub fn update_cell(&mut self, style: Style, pos: LocalPos) {
        if pos.x >= self.size.width {
            return;
        }

        self.ensure_storage(pos);

        let cell = &mut self.lines[pos.y as usize][pos.x as usize];

        if let fg @ Some(_) = style.fg {
            cell.style.fg = fg;
        }

        if let bg @ Some(_) = style.bg {
            cell.style.bg = bg;
        }

        cell.style.attributes |= style.attributes;
    }

    /// Get a reference to a `char` and [`Style`] at a given position inside the buffer.
    pub fn get(&mut self, pos: LocalPos) -> Option<(&Glyph, &Style)> {
        if pos.x >= self.size.width {
            return None;
        }

        self.ensure_storage(pos);

        let cell = &self.lines[pos.y as usize][pos.x as usize];
        match &cell.state {
            CellState::Occupied(c) => Some((c, &cell.style)),
            _ => None,
        }
    }

    /// Get a mutable reference to a `char` and [`Style`] at a given position inside the buffer.
    pub fn get_mut(&mut self, pos: LocalPos) -> Option<(&mut Glyph, &mut Style)> {
        if pos.x >= self.size.width || pos.y >= self.size.height {
            return None;
        }

        let index = pos.to_index(self.size.width);
        let cell = &mut self.lines[pos.y as usize][pos.x as usize];
        match &mut cell.state {
            CellState::Occupied(c) => Some((c, &mut cell.style)),
            _ => None,
        }
    }

    pub(super) fn reset_cell(&mut self, pos: LocalPos) {
        let cell = &mut self.lines[pos.y as usize][pos.x as usize];
        *cell = Cell::empty();
    }

    fn put(&mut self, mut cell: Cell, pos: LocalPos) {
        if let CellState::Occupied(c) = cell.state {
            // If this is a unicode char that is wider than one cell,
            // add a continuation cell if it fits, this way if we overwrite it
            // we can set the continuation cell to `Empty`.
            if pos.x < self.size.width && c.width() >= 2 {
                self.put(Cell::continuation(cell.style), LocalPos::new(pos.x + 1, pos.y));
            }
        }

        let current = &mut self.lines[pos.y as usize][pos.x as usize];
        cell.style.merge(current.style);

        match (&mut current.state, cell.state) {
            // Merge the styles
            (CellState::Occupied(current_char), CellState::Occupied(new_char)) => {
                *current_char = new_char;
                current.style.attributes |= cell.style.attributes;

                if let Some(col) = cell.style.fg {
                    current.style.fg = Some(col);
                }

                if let Some(col) = cell.style.bg {
                    current.style.bg = Some(col);
                }
            }
            _ => *current = cell,
        }
    }

    pub(super) fn cell_lines(&mut self) -> impl Iterator<Item = &mut Box<[Cell]>> {
        self.lines.iter_mut()
    }
}

// -----------------------------------------------------------------------------
//     - Draw changes -
// -----------------------------------------------------------------------------
pub(crate) fn draw(mut w: impl Write, glyph_map: &GlyphMap, cmds: &Vec<DrawCommand>, y: usize, size: Size) -> Result<()> {
    // let mut next_cell_x = None;
    w.queue(Print(format!("width: {}", size.width)))?;

    // // for (screen_pos, style, change) in cmds {
    // for cmd in cmds {
    //     match cmd {
    //         DrawCommand::Newline => _ = w.queue(Print('\n'))?,
    //         DrawCommand::Draw { x, style, glyph } => {
    //             // Cursor movement
    //             let should_move = match next_cell_x {
    //                 Some(next_x) => next_x != *x,
    //                 _ => true,
    //             };

    //             if should_move {
    //                 w.queue(cursor::MoveTo(*x as u16, y as u16))?;
    //             }

    //             next_cell_x = Some(x + glyph.width() as u16);

    //             // Apply style
    //             write_style(style, &mut w)?;

    //             // Draw
    //             match glyph {
    //                 Glyph::Single(c, _) => {
    //                     w.queue(Print(c))?;
    //                 }
    //                 Glyph::Cluster(index, _) => {
    //                     if let Some(glyph) = glyph_map.get(*index) {
    //                         w.queue(Print(glyph))?;
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    // write_style(&Style::reset(), &mut w)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use anathema_state::Color;

    use super::*;
    use crate::tui;

    #[test]
    fn changes() {
        // Changes between the old buffer and the new buffer should be two inserts and one removal.
        // The isnerts are for C and N, and since V is no longer present
        // in the new buffer, it should be removed

        let mut changes = vec![];

        let mut old_buffer = Buffer::new((5u16, 3));
        old_buffer.inner[0] = Cell::new(Glyph::from_char('O', 1), Style::reset());
        old_buffer.inner[1] = Cell::new(Glyph::from_char('V', 1), Style::reset());

        let mut new_buffer = Buffer::new((5u16, 3));
        new_buffer.inner[0] = Cell::new(Glyph::from_char('C', 1), Style::reset());
        new_buffer.inner[2] = Cell::new(Glyph::from_char('N', 1), Style::reset());

        diff(&mut old_buffer, &mut new_buffer, &mut changes).unwrap();

        let (_, _, change_1) = changes[0]; // Insert 'C'
        let (_, _, change_2) = changes[1]; // Remove 'V'
        let (_, _, change_3) = changes[2]; // Insert 'N'

        assert_eq!(Change::Insert(Glyph::from_char('C', 1)), change_1);
        assert_eq!(Change::Remove, change_2);
        assert_eq!(Change::Insert(Glyph::from_char('N', 1)), change_3);
    }

    #[test]
    fn resize() {
        let mut buffer = Buffer::new((2u16, 2));
        buffer.inner[0] = Cell::new(Glyph::from_char('1', 1), Style::reset());
        buffer.inner[1] = Cell::new(Glyph::from_char('2', 1), Style::reset());
        buffer.inner[2] = Cell::new(Glyph::from_char('3', 1), Style::reset());
        buffer.inner[3] = Cell::new(Glyph::from_char('4', 1), Style::reset());

        buffer.resize(Size::new(1, 2));
        assert_eq!(buffer.inner[0], Cell::new(Glyph::from_char('1', 1), Style::reset()));
        assert_eq!(buffer.inner[1], Cell::new(Glyph::from_char('3', 1), Style::reset()));
    }

    #[test]
    fn update_cell_checks_range() {
        let mut under_test = Buffer::new((1, 2));

        let valid_pos = LocalPos::new(0, 1);
        under_test.put_glyph(Glyph::from_char('1', 1), valid_pos);
        under_test.put_glyph(Glyph::from_char('2', 1), valid_pos);

        let new_style = Style {
            fg: Some(Color::Red),
            bg: None,
            attributes: tui::Attributes::empty(),
        };
        under_test.update_cell(new_style, LocalPos::new(1, 0));

        assert_eq!(under_test.get(valid_pos).unwrap().1.clone(), Style::reset());
    }

    #[test]
    #[should_panic(expected = "position out of bounds")]
    fn put_glyph_checks_range() {
        let mut under_test = Buffer::new((1, 2));

        under_test.put_glyph(Glyph::from_char('x', 1), LocalPos::new(0, 0));
        under_test.put_glyph(Glyph::from_char('x', 1), LocalPos::new(0, 1));
        under_test.put_glyph(Glyph::from_char('o', 1), LocalPos::new(1, 0));
    }

    #[test]
    fn get_checks_range() {
        let mut under_test = Buffer::new((1, 2));

        under_test.put_glyph(Glyph::from_char('1', 1), LocalPos::new(0, 0));
        under_test.put_glyph(Glyph::from_char('2', 1), LocalPos::new(0, 1));

        assert_eq!(under_test.get(LocalPos::new(1, 0)), None);
    }

    #[test]
    fn get_mut_checks_range() {
        let mut under_test = Buffer::new((1, 2));

        under_test.put_glyph(Glyph::from_char('1', 1), LocalPos::new(0, 0));
        under_test.put_glyph(Glyph::from_char('2', 1), LocalPos::new(0, 1));

        assert_eq!(under_test.get_mut(LocalPos::new(1, 0)), None);
    }
}
