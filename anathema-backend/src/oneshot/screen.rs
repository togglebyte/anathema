use std::io::{Result, Write};

use anathema_geometry::{Pos, Size};
use anathema_value_resolver::Attributes;
use anathema_widgets::paint::Glyph;
use anathema_widgets::{GlyphMap, Style, WidgetRenderer};
use crossterm::event::EnableMouseCapture;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor, ExecutableCommand, QueueableCommand};

use super::buffer::{draw, Buffer, CellState};
use super::LocalPos;

pub(super) enum DrawCommand {
    Draw { x: u16, style: Style, glyph: Glyph },
    Newline,
}

/// The `Screen` is used to draw to some `std::io::Write`able output (generally `stdout`);
pub struct Screen {
    buffer: Buffer,
    draw_commands: Vec<DrawCommand>,
}

impl Screen {
    /// The `output` should be a mutable reference to whatever this screen renders to.
    /// The `output` is used initially to move the cursor and hide it.
    pub fn new(size: impl Into<Size>) -> Self {
        let size: Size = size.into();

        Self {
            buffer: Buffer::new(size),
            draw_commands: vec![],
        }
    }

    /// Erase a specific region.
    /// Will reset the styles for all the cells as well.
    pub(crate) fn erase_region(&mut self, pos: LocalPos, size: Size) {
        let to_x = (size.width + pos.x).min(self.size().width);
        let to_y = (size.height + pos.y).min(self.size().height);

        for x in pos.x.min(to_x)..to_x {
            for y in pos.y.min(to_y)..to_y {
                self.buffer.reset_cell(LocalPos::new(x, y));
            }
        }
    }

    /// Put a char at the given screen position, with a given style.
    /// If the screen position is outside the [`Buffer`]s size then this is
    /// out of bounds and will panic.
    pub(crate) fn paint_glyph(&mut self, glyph: Glyph, pos: LocalPos) {
        self.buffer.put_glyph(glyph, pos);
    }

    pub(crate) fn update_cell(&mut self, style: Style, pos: LocalPos) {
        self.buffer.update_cell(style, pos);
    }

    /// Draw the changes to the screen
    pub(crate) fn render(&mut self, mut output: impl Write, offset: LocalPos, glyph_map: &GlyphMap) -> Result<()> {
        // From buffer to draw commands
        for (y, line) in self.buffer.cell_lines().enumerate() {
            for (x, cell) in line.iter().enumerate() {
                let glyph = match cell.state {
                    CellState::Empty | CellState::Continuation => continue,
                    CellState::Occupied(c) => c,
                };

                self.draw_commands.push(DrawCommand::Draw { x: x as u16, style: cell.style, glyph });
            }

            self.draw_commands.push(DrawCommand::Newline);
        }

        draw(&mut output, glyph_map, &self.draw_commands, offset.y as usize, self.size())?;
        output.flush()?;
        Ok(())
    }
}

impl WidgetRenderer for Screen {
    fn draw_glyph(&mut self, c: Glyph, pos: Pos) {
        let Ok(screen_pos) = pos.try_into() else { return };
        self.paint_glyph(c, screen_pos);
    }

    fn set_attributes(&mut self, attribs: &Attributes<'_>, pos: Pos) {
        let style = Style::from_cell_attribs(attribs);
        self.set_style(style, pos)
    }

    fn set_style(&mut self, style: Style, local_pos: Pos) {
        let Ok(screen_pos) = local_pos.try_into() else { return };
        self.update_cell(style, screen_pos);
    }

    fn size(&self) -> Size {
        self.buffer.size()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tui::buffer::{Cell, CellState};

    fn state_at(buffer: &Buffer, x: usize, y: usize) -> CellState {
        let cell = buffer[(x, y)];
        cell.state
    }

    fn char_at(buffer: &Buffer, x: usize, y: usize) -> char {
        match state_at(buffer, x, y) {
            CellState::Occupied(Glyph::Single(c, _)) => c,
            _ => panic!(),
        }
    }

    #[test]
    fn render() {
        // Render a character
        let mut render_output = vec![];
        let glyph_map = GlyphMap::empty();
        let mut screen = Screen::new(Size::new(1, 1));
        screen.paint_glyph(Glyph::from_char('x', 1), LocalPos::ZERO);
        screen.render(&mut render_output, &glyph_map).unwrap();

        let expected = Cell::new(Glyph::from_char('x', 1), Style::reset());
        let actual = screen.new_buffer.inner[0];
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_region() {
        let mut render_output = vec![];
        let glyph_map = GlyphMap::empty();
        let mut screen = Screen::new(Size::new(2, 2));
        screen.render(&mut render_output, &glyph_map).unwrap();

        // Erase the bottom right corner of the 2x2 region
        screen.erase_region(LocalPos::new(1, 1), Size::new(2, 2));

        let top_left = screen.new_buffer.inner[0];
        assert_eq!(Cell::empty(), top_left);
        let bottom_right = screen.new_buffer.inner[3];
        assert_eq!(Cell::empty(), bottom_right);
    }

    #[test]
    #[should_panic(expected = "position out of bounds")]
    fn put_outside_of_screen() {
        // Put a character outside of the screen should panic
        let glyph_map = GlyphMap::empty();
        let mut screen = Screen::new(Size::new(1, 1));
        screen.paint_glyph(Glyph::from_char('x', 1), LocalPos::new(3, 0));
        screen.render(&mut vec![], &glyph_map).unwrap();
    }

    #[test]
    fn erasing_unicode_with_continuation_cell() {
        // Paint a bunny in the next cell

        let glyph_map = GlyphMap::empty();
        let mut screen = Screen::new(Size::new(4, 1));
        let bunny = '🐇';
        // Where B = Bunny, c = continuation, 1 = some character
        //
        // Buffer
        // B c 1 0
        //
        // Next buffer
        // - B c 1

        // First frame: Bc10
        screen.paint_glyph(Glyph::from_char('1', 1), LocalPos::new(2, 0));
        screen.paint_glyph(Glyph::from_char('0', 1), LocalPos::new(3, 0));
        screen.paint_glyph(Glyph::from_char(bunny, 2), LocalPos::new(0, 0));

        screen.render(&mut vec![], &glyph_map).unwrap();

        assert_eq!(char_at(&screen.new_buffer, 0, 0), bunny);
        assert_eq!(state_at(&screen.new_buffer, 1, 0), CellState::Continuation);
        assert_eq!(char_at(&screen.new_buffer, 2, 0), '1');
        assert_eq!(char_at(&screen.new_buffer, 3, 0), '0');

        // // Second frame: -Bc1
        screen.erase();

        screen.paint_glyph(Glyph::from_char('1', 1), LocalPos::new(3, 0));
        screen.paint_glyph(Glyph::from_char(bunny, 2), LocalPos::new(1, 0));
        screen.render(&mut vec![], &glyph_map).unwrap();

        assert_eq!(state_at(&screen.new_buffer, 0, 0), CellState::Empty);
        assert_eq!(char_at(&screen.new_buffer, 1, 0), bunny);
        assert_eq!(state_at(&screen.new_buffer, 2, 0), CellState::Continuation);
        assert_eq!(char_at(&screen.new_buffer, 3, 0), '1');
    }
}
