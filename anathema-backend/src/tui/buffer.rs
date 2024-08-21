#![deny(missing_docs)]
use std::io::{Result, Write};

use anathema_geometry::Size;
use crossterm::style::Print;
use crossterm::{cursor, QueueableCommand};
use unicode_width::UnicodeWidthChar;

use super::{LocalPos, Style};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Cell {
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
            state: CellState::Occupied(' '),
        }
    }

    fn continuation(style: Style) -> Self {
        Self {
            style,
            state: CellState::Continuation,
        }
    }

    pub(crate) fn new(c: char, style: Style) -> Self {
        Self {
            style,
            state: CellState::Occupied(c),
        }
    }
}

/// Represent the state of a cell inside a [`Buffer`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum CellState {
    /// Empty
    Empty,
    /// Occupied by a certain character
    Occupied(char),
    /// A continuation means this cell is part of another cell
    /// representing a value that spans more than two chars, e.g 💖
    Continuation,
}

/// A buffer contains a list of cells representing characters that can be rendered.
/// This doesn't necessarily have to be `stdout`, it can be anything that implements
/// [`std::io::Write`]
///
/// The [`crate::Screen`] holds two `Buffer`s, used to only render the difference between frames.
///
/// The [`crate::Screen`] writes all the chars and their styles to the buffer, which works like a
/// grid.
#[derive(Debug, Clone)]
pub struct Buffer {
    size: Size,
    pub(crate) inner: Box<[Cell]>,
}

impl Buffer {
    /// Crate a new `Buffer` with a given size.
    pub fn new(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            inner: vec![Cell::empty(); size.width * size.height].into_boxed_slice(),
            size,
        }
    }

    /// Create a new buffer with reset cells
    pub(crate) fn reset(size: impl Into<Size>) -> Self {
        let size = size.into();
        Self {
            inner: vec![Cell::reset(); size.width * size.height].into_boxed_slice(),
            size,
        }
    }

    /// The size of the `Buffer`
    pub fn size(&self) -> Size {
        self.size
    }

    /// Resize the buffer, truncating what doesn't fit but keeps what does.
    pub fn resize(&mut self, size: Size) {
        let mut new_buf = Buffer::new(size);
        for (y, line) in self.cell_lines().enumerate() {
            if y >= size.height {
                break;
            }

            for (x, cell) in line.iter().enumerate() {
                if x >= size.width {
                    break;
                }

                let pos = LocalPos::new(x as u16, y as u16);
                new_buf.put(*cell, pos);
            }
        }

        self.size = size;
        self.inner = new_buf.inner;
    }

    /// Put a character with a style at a given position.
    pub fn put_char(&mut self, c: char, pos: LocalPos) {
        let style = match self.get(pos) {
            Some((_, style)) => *style,
            None => Style::new(),
        };
        let cell = Cell::new(c, style);
        self.put(cell, pos);
    }

    /// Update the attributes at a given cell.
    /// If there is no character at that cell, then write an empty space into it
    pub fn update_cell(&mut self, style: Style, pos: LocalPos) {
        if pos.x as usize >= self.size.width || pos.y as usize >= self.size.height {
            return;
        }

        let index = pos.to_index(self.size.width);
        let cell = &mut self.inner[index];

        if let fg @ Some(_) = style.fg {
            cell.style.fg = fg;
        }

        if let bg @ Some(_) = style.bg {
            cell.style.bg = bg;
        }

        cell.style.attributes |= style.attributes;

        if let CellState::Empty = cell.state {
            cell.state = CellState::Occupied(' ');
        }
    }

    /// Get a reference to a `char` and [`Style`] at a given position inside the buffer.
    pub fn get(&self, pos: LocalPos) -> Option<(&char, &Style)> {
        let index = self.index(pos);
        let cell = self.inner.get(index)?;
        match &cell.state {
            CellState::Occupied(c) => Some((c, &cell.style)),
            _ => None,
        }
    }

    /// Get a mutable reference to a `char` and [`Style`] at a given position inside the buffer.
    pub fn get_mut(&mut self, pos: LocalPos) -> Option<(&mut char, &mut Style)> {
        let index = self.index(pos);
        let cell = self.inner.get_mut(index)?;
        match &mut cell.state {
            CellState::Occupied(c) => Some((c, &mut cell.style)),
            _ => None,
        }
    }

    /// Empty a cell at a given position
    pub fn empty(&mut self, pos: LocalPos) {
        let index = self.index(pos);
        self.inner[index] = Cell::empty();
    }

    /// An iterator over all the rows in the buffer
    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = Option<(char, Style)>> + '_> {
        self.cell_lines().map(|chunk| {
            chunk.iter().map(|cell| match cell.state {
                CellState::Occupied(c) => Some((c, cell.style)),
                _ => None,
            })
        })
    }

    fn index(&self, pos: LocalPos) -> usize {
        pos.y as usize * self.size.width + pos.x as usize
    }

    fn put(&mut self, mut cell: Cell, pos: LocalPos) {
        let index = self.index(pos);

        if let CellState::Occupied(c) = cell.state {
            // If this is a unicode char that is wider than one cell,
            // add a continuation cell if it fits, this way if we overwrite it
            // we can set the continuation cell to `Empty`.
            if pos.x < self.size.width as u16 {
                if let Some(2..) = c.width() {
                    self.put(Cell::continuation(cell.style), LocalPos::new(pos.x + 1, pos.y));
                }
            }
        }

        let current = &mut self.inner[index];
        cell.style.merge(current.style);

        match (&mut current.state, cell.state) {
            // Merge the styles
            (CellState::Occupied(ref mut current_char), CellState::Occupied(new_char)) => {
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

    fn cell_lines(&self) -> impl Iterator<Item = &[Cell]> {
        self.inner.chunks(self.size.width)
    }
}

#[cfg(test)]
impl Buffer {
    fn cell_at(&self, x: usize, y: usize) -> Cell {
        let index = y * self.size.width + x;
        self.inner[index]
    }

    pub fn char_at(&self, x: usize, y: usize) -> char {
        let cell = self.cell_at(x, y);
        match cell.state {
            CellState::Occupied(c) => c,
            _ => panic!("no character at index {x}, {y}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Change {
    Remove,
    Insert(char),
}

impl Change {
    fn width(self) -> usize {
        match self {
            Change::Remove => 1,
            Change::Insert(c) => c.width().unwrap_or(1),
        }
    }
}

pub(crate) fn diff(old: &Buffer, new: &Buffer, changes: &mut Vec<(LocalPos, Option<Style>, Change)>) -> Result<()> {
    let mut previous_style = None;

    for (y, (old_line, new_line)) in old.cell_lines().zip(new.cell_lines()).enumerate() {
        for (x, (old_cell, new_cell)) in old_line.iter().zip(new_line).enumerate() {
            let x = x as u16;
            let y = y as u16;

            if old_cell == new_cell {
                continue;
            }

            let style = match previous_style {
                Some(previous) => (previous != new_cell.style).then_some(new_cell.style),
                None => Some(new_cell.style),
            };

            previous_style = Some(new_cell.style);

            let change = match new_cell.state {
                CellState::Empty => Change::Remove,
                CellState::Continuation => continue,
                CellState::Occupied(c) => Change::Insert(c),
            };

            changes.push((LocalPos::new(x, y), style, change));
        }
    }

    Ok(())
}

// -----------------------------------------------------------------------------
//     - Draw changes -
// -----------------------------------------------------------------------------
pub(crate) fn draw_changes(mut w: impl Write, changes: &Vec<(LocalPos, Option<Style>, Change)>) -> Result<()> {
    let mut last_y = None;
    let mut next_cell_x = None;

    for (screen_pos, style, change) in changes {
        // Cursor movement
        let should_move = match (last_y, next_cell_x) {
            (Some(last_y), Some(next_x)) => screen_pos.y > last_y || next_x != screen_pos.x,
            _ => true,
        };

        if should_move {
            w.queue(cursor::MoveTo(screen_pos.x, screen_pos.y))?;
        }

        last_y = Some(screen_pos.y);
        next_cell_x = Some(screen_pos.x + change.width() as u16);

        // Apply style
        if let Some(style) = style {
            style.write(&mut w)?;
        }

        // Draw changes
        match change {
            Change::Insert(c) => w.queue(Print(c))?,
            Change::Remove => w.queue(Print(' '))?,
        };
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn changes() {
        // Changes between the old buffer and the new buffer should be two inserts and one removal.
        // The isnerts are for C and N, and since V is no longer present
        // in the new buffer, it should be removed

        let mut changes = vec![];

        let mut old_buffer = Buffer::new((5u16, 3));
        old_buffer.inner[0] = Cell::new('O', Style::reset());
        old_buffer.inner[1] = Cell::new('V', Style::reset());

        let mut new_buffer = Buffer::new((5u16, 3));
        new_buffer.inner[0] = Cell::new('C', Style::reset());
        new_buffer.inner[2] = Cell::new('N', Style::reset());

        diff(&old_buffer, &new_buffer, &mut changes).unwrap();

        let (_, _, change_1) = changes[0]; // Insert 'C'
        let (_, _, change_2) = changes[1]; // Remove 'V'
        let (_, _, change_3) = changes[2]; // Insert 'N'

        assert_eq!(Change::Insert('C'), change_1);
        assert_eq!(Change::Remove, change_2);
        assert_eq!(Change::Insert('N'), change_3);
    }

    #[test]
    fn resize() {
        let mut buffer = Buffer::new((2u16, 2));
        buffer.inner[0] = Cell::new('1', Style::reset());
        buffer.inner[1] = Cell::new('2', Style::reset());
        buffer.inner[2] = Cell::new('3', Style::reset());
        buffer.inner[3] = Cell::new('4', Style::reset());

        buffer.resize(Size::new(1, 2));
        assert_eq!(buffer.inner[0], Cell::new('1', Style::reset()));
        assert_eq!(buffer.inner[1], Cell::new('3', Style::reset()));
    }
}
