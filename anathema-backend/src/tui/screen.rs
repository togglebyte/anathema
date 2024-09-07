use std::io::{Result, Write};

use anathema_geometry::{Pos, Size};
use anathema_widgets::paint::CellAttributes;
use anathema_widgets::WidgetRenderer;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::style::Print;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, ExecutableCommand, QueueableCommand};
use unicode_width::UnicodeWidthChar;

use super::buffer::{Buffer, Change};
use super::{Cell, LocalPos, Style};

/// The `Screen` is used to draw to some `std::io::Write`able output (generally `stdout`);
#[derive(Debug)]
pub struct Screen {
    pub(crate) buffer: Buffer,
}

impl Screen {
    /// Hide the cursor
    pub(super) fn hide_cursor(mut output: impl Write) -> Result<()> {
        output.queue(cursor::Hide)?;
        Ok(())
    }

    /// Show the cursor
    pub(super) fn show_cursor(mut output: impl Write) -> Result<()> {
        output.queue(cursor::Show)?;
        Ok(())
    }

    /// Enable mouse support
    pub(super) fn enable_mouse(mut output: impl Write) -> Result<()> {
        output.queue(EnableMouseCapture)?;
        Ok(())
    }

    /// Create a new instance of a screen.
    /// The `output` should be a mutable reference to whatever this screen renders to.
    /// The `output` is used initially to move the cursor and hide it.
    pub fn new(size: impl Into<Size>) -> Self {
        let size: Size = size.into();

        Self {
            buffer: Buffer::new(size),
        }
    }

    /// Resize the buffer.
    /// This will empty the underlying buffers so everything will have
    /// to be redrawn.
    pub(super) fn resize(&mut self, new_size: Size) {
        self.buffer.resize(new_size);
    }

    /// Erase the entire buffer by writing empty cells
    pub(crate) fn erase(&mut self) {
        self.erase_region(LocalPos::ZERO, self.size());
    }

    /// Erase a specific region.
    /// Will reset the styles for all the cells as well.
    pub(crate) fn erase_region(&mut self, pos: LocalPos, size: Size) {
        let to_x = (size.width as u16 + pos.x).min(self.size().width as u16);
        let to_y = (size.height as u16 + pos.y).min(self.size().height as u16);

        for x in pos.x.min(to_x)..to_x {
            for y in pos.y.min(to_y)..to_y {
                self.buffer.remove(LocalPos::new(x, y));
            }
        }
    }

    /// Put a char at the given screen position, with a given style.
    pub(crate) fn paint_glyph(&mut self, c: char, pos: LocalPos) {
        self.buffer.put_char(c, pos);
    }

    pub(crate) fn update_cell(&mut self, style: Style, pos: LocalPos) {
        self.buffer.update_cell(style, pos);
    }

    /// Draw the changes to the screen
    pub(crate) fn render(&mut self, mut output: impl Write + std::fmt::Debug) -> Result<()> {
        let size = self.size();
        self.buffer.changes.sort();

        {
            for index in &self.buffer.changes {
                puffin::profile_scope!("apply change");
                let cell = std::mem::take(&mut self.buffer.cells[*index]);
                let c = match cell.state {
                    super::CellState::Empty => ' ',
                    super::CellState::Occupied(c) => c,
                    super::CellState::Continuation => continue,
                };

                let pos = LocalPos::from((*index, size));
                output.queue(cursor::MoveTo(pos.x, pos.y))?;
                {
                    cell.write(&mut output)?;
                }
            }
        }

        let mut src = self.buffer.changes.as_slice();
        let mut offset = 0;
        for index in self.buffer.prev_changes.drain(..) {
            match src[offset..].binary_search(&index) {
                Ok(idx) => offset = idx,
                Err(idx) => {
                    offset = idx;
                    let pos = LocalPos::from((index, size));
                    output.queue(cursor::MoveTo(pos.x, pos.y))?;
                    Cell::reset().write(&mut output)?;
                }
            }
        }

        std::mem::swap(&mut self.buffer.prev_changes, &mut self.buffer.changes);

        // self.buffer.changes.swap();

        // self.buffer.changes.sort();

        // let size = self.size();
        // let mut last_pos = LocalPos::ZERO;

        // for index in &self.buffer.changes {
        //     let cell = &mut self.buffer.cells[*index];
        //     cell.dirty = false;
        //     let c = match cell.state {
        //         super::CellState::Empty => ' ',
        //         super::CellState::Occupied(c) => c,
        //         super::CellState::Continuation => continue,
        //     };

        //     let pos = LocalPos::from((*index, size));
        //     last_pos.x += c.width().unwrap_or(0) as u16;

        //     if last_pos.x != pos.x || last_pos.y != pos.y {
        //         output.queue(cursor::MoveTo(pos.x, pos.y))?;
        //     }
        //     last_pos = pos;

        //     cell.write(&mut output);
        // }

        // if self.buffer.changes != self.buffer.prev_changes {
        //     let mut comp = self.buffer.changes.as_slice();
        //     for index in self.buffer.prev_changes.drain(..) {
        //         match comp.binary_search(&index) {
        //             Ok(idx) => comp = &comp[idx..],
        //             Err(idx) => {
        //                 // Remove this value
        //                 comp = &comp[idx..];

        //                 let cell = &mut self.buffer.cells[index];
        //                 *cell = Cell::reset();

        //                 let pos = LocalPos::from((index, size));
        //                 // last_pos.x += c.width().unwrap_or(0) as u16;

        //                 // if last_pos.x != pos.x || last_pos.y != pos.y {
        //                 output.queue(cursor::MoveTo(pos.x, pos.y))?;
        //                 // }
        //                 // last_pos = pos;

        //                 cell.write(&mut output)?;
        //             }
        //         }
        //     }
        // }

        // std::mem::swap(&mut self.buffer.changes, &mut self.buffer.prev_changes);

        output.flush()?;

        Ok(())
    }

    /// Enter an alternative screen.
    /// When using this with stdout it means the output will not persist once the program exits.
    pub fn enter_alt_screen(mut output: impl Write) -> Result<()> {
        output.execute(EnterAlternateScreen)?;
        Ok(())
    }

    /// Enable raw mode: input will not be forwarded to the screen.
    pub fn enable_raw_mode() -> Result<()> {
        enable_raw_mode()?;
        Ok(())
    }

    /// Disable raw mode: input will be forwarded to the screen.
    pub fn disable_raw_mode() -> Result<()> {
        disable_raw_mode()?;
        Ok(())
    }

    /// Restore the terminal by setting the cursor to show, disable raw mode, disable mouse capture
    /// and leave any alternative screens
    pub fn restore(&mut self, mut output: impl Write) -> Result<()> {
        disable_raw_mode()?;
        output.execute(LeaveAlternateScreen)?;
        #[cfg(not(target_os = "windows"))]
        output.execute(DisableMouseCapture)?;
        output.execute(cursor::Show)?;
        Ok(())
    }
}

impl WidgetRenderer for Screen {
    fn draw_glyph(&mut self, c: char, pos: Pos) {
        let Ok(screen_pos) = pos.try_into() else { return };
        self.paint_glyph(c, screen_pos);
    }

    fn set_attributes(&mut self, attribs: &dyn CellAttributes, pos: Pos) {
        let Ok(screen_pos) = pos.try_into() else { return };
        let style = Style::from_cell_attribs(attribs);
        self.update_cell(style, screen_pos);
    }

    fn size(&self) -> Size {
        self.buffer.size()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tui::buffer::Cell;

    fn make_screen(size: Size) -> Screen {
        let mut screen = Screen::new(size);
        for y in 0..size.height {
            let c = y.to_string().chars().next().unwrap();
            for x in 0..size.width {
                screen.paint_glyph(c, LocalPos::new(x as u16, y as u16));
            }
        }

        screen
    }

    #[test]
    fn render() {
        // Render a character
        let mut render_output = vec![];
        let mut screen = make_screen(Size::new(1, 1));
        screen.paint_glyph('x', LocalPos::ZERO);
        screen.render(&mut render_output).unwrap();

        let expected = Cell::new('x', Style::reset());
        let actual = screen.new_buffer.cells[0];
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_region() {
        // Erase a whole region, leaving all cells `empty`
        let mut render_output = vec![];
        let mut screen = make_screen(Size::new(2, 2));
        screen.render(&mut render_output).unwrap();

        screen.erase_region(LocalPos::new(1, 1), Size::new(1, 1));
        screen.render(&mut render_output).unwrap();

        let top_left = screen.new_buffer.cells[0];
        assert_eq!(Cell::new('0', Style::reset()), top_left);
        let bottom_right = screen.new_buffer.cells[3];
        assert_eq!(Cell::empty(), bottom_right);
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 1 but the index is 4")]
    fn put_outside_of_screen() {
        // Put a character outside of the screen should panic
        let mut screen = make_screen(Size::new(1, 1));
        screen.paint_glyph('x', LocalPos::new(2, 2));
        screen.render(&mut vec![]).unwrap();
    }
}
