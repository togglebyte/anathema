use std::io::{Result, Write};

use anathema_geometry::{Pos, Size};
use anathema_value_resolver::Attributes;
use anathema_widgets::paint::Glyph;
use anathema_widgets::{GlyphMap, Style, WidgetRenderer};
use crossterm::event::EnableMouseCapture;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use crossterm::{ExecutableCommand, QueueableCommand, cursor};

use super::LocalPos;
use super::buffer::{Buffer, Change, diff, draw_changes};

/// The `Screen` is used to draw to some `std::io::Write`able output (generally `stdout`);
pub struct Screen {
    // This is pub(crate) for testing purposes
    pub(crate) new_buffer: Buffer,
    old_buffer: Buffer,
    changes: Vec<(LocalPos, Option<Style>, Change)>,
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
            old_buffer: Buffer::new(size),
            new_buffer: Buffer::new(size),
            changes: vec![],
        }
    }

    /// Resize the buffer.
    /// This will empty the underlying buffers so everything will have
    /// to be redrawn.
    pub(super) fn resize(&mut self, new_size: Size) {
        self.old_buffer = Buffer::new(new_size);
        self.new_buffer = Buffer::reset(new_size);
    }

    /// Erase the entire buffer by writing empty cells
    pub(crate) fn erase(&mut self) {
        self.erase_region(LocalPos::ZERO, self.size());
    }

    /// Erase a specific region.
    /// Will reset the styles for all the cells as well.
    pub(crate) fn erase_region(&mut self, pos: LocalPos, size: Size) {
        let to_x = (size.width + pos.x).min(self.size().width);
        let to_y = (size.height + pos.y).min(self.size().height);

        for x in pos.x.min(to_x)..to_x {
            for y in pos.y.min(to_y)..to_y {
                self.new_buffer.reset_cell(LocalPos::new(x, y));
            }
        }
    }

    /// Put a char at the given screen position, with a given style.
    /// If the screen position is outside the [`Buffer`]s size then this is
    /// out of bounds and will panic.
    pub(crate) fn paint_glyph(&mut self, glyph: Glyph, pos: LocalPos) {
        self.new_buffer.put_glyph(glyph, pos);
    }

    pub(crate) fn update_cell(&mut self, style: Style, pos: LocalPos) {
        self.new_buffer.update_cell(style, pos);
    }

    /// Draw the changes to the screen
    pub(crate) fn render(&mut self, mut output: impl Write, glyph_map: &GlyphMap) -> Result<()> {
        diff(&mut self.old_buffer, &mut self.new_buffer, &mut self.changes)?;

        if self.changes.is_empty() {
            return Ok(());
        }

        draw_changes(&mut output, glyph_map, &self.changes)?;

        self.changes.clear();

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
    pub fn restore(&mut self, mut output: impl Write, leave_alt_screen: bool) -> Result<()> {
        disable_raw_mode()?;
        if leave_alt_screen {
            output.execute(LeaveAlternateScreen)?;
        }
        #[cfg(not(target_os = "windows"))]
        output.execute(crossterm::event::DisableMouseCapture)?;
        output.execute(cursor::Show)?;
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
        self.new_buffer.size()
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
                screen.paint_glyph(Glyph::from_char(c, 1), LocalPos::new(x, y));
            }
        }

        screen
    }

    #[test]
    fn render() {
        // Render a character
        let mut render_output = vec![];
        let glyph_map = GlyphMap::empty();
        let mut screen = make_screen(Size::new(1, 1));
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
        let mut screen = make_screen(Size::new(2, 2));
        screen.render(&mut render_output, &glyph_map).unwrap();

        // Erase the bottom right corner of the 2x2 region
        screen.erase_region(LocalPos::new(1, 1), Size::new(1, 1));

        let top_left = screen.new_buffer.inner[0];
        assert_eq!(Cell::new(Glyph::from_char('0', 1), Style::reset()), top_left);
        let bottom_right = screen.new_buffer.inner[3];
        assert_eq!(Cell::empty(), bottom_right);
    }

    #[test]
    #[should_panic(expected = "position out of bounds")]
    fn put_outside_of_screen() {
        // Put a character outside of the screen should panic
        let glyph_map = GlyphMap::empty();
        let mut screen = make_screen(Size::new(1, 1));
        screen.paint_glyph(Glyph::from_char('x', 1), LocalPos::new(3, 0));
        screen.render(&mut vec![], &glyph_map).unwrap();
    }
}
