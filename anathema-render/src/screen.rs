use std::io::{Result, Write};

use super::buffer::{diff, draw_changes, Buffer};
use super::{ScreenPos, Size, Style};
use crossterm::event::DisableMouseCapture;
use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, ExecutableCommand, QueueableCommand};

/// The `Screen` is used to draw to some `std::io::Write`able output (generally `stdout`);
pub struct Screen {
    // This is pub(crate) for testing purposes
    pub(crate) new_buffer: Buffer,
    old_buffer: Buffer,
}

impl Screen {
    /// Create a new instance of a screen.
    /// The `output` should be a mutable reference to whatever this screen renders to.
    /// The `output` is used initially to move the cursor and hide it.
    pub fn new(mut output: impl Write, size: impl Into<Size>) -> Result<Self> {
        let size: Size = size.into();
        output.queue(cursor::Hide)?;
        let inst = Self { old_buffer: Buffer::new(size), new_buffer: Buffer::new(size) };
        Ok(inst)
    }

    /// Access to the current buffer
    pub fn buffer(&self) -> &Buffer {
        &self.new_buffer
    }

    /// The size of the underlying buffer
    pub fn size(&self) -> Size {
        self.new_buffer.size()
    }

    /// Resize the buffer.
    /// This will empty the underlying buffers so everything will have
    /// to be redrawn.
    pub fn resize(&mut self, new_size: Size) {
        self.old_buffer = Buffer::new(new_size);
        self.new_buffer = Buffer::new(new_size);
    }

    /// Clear the entire screen.
    /// If anything was written to the screen (e.g through [`put`](Self::put)) it will be cleared
    /// as well.
    ///
    /// Will most likely cause flickering, and should only be used
    /// when initialising a blank screen.
    ///
    /// To clear the screen between draw calls, use [`Screen::erase`]
    pub fn clear_all(&mut self, mut output: impl Write) -> Result<()> {
        self.erase();
        output.flush()?;
        output.queue(cursor::MoveTo(0, 0))?;
        output.queue(SetForegroundColor(Color::Reset))?;
        output.queue(SetBackgroundColor(Color::Reset))?;
        output.queue(Clear(ClearType::All))?;
        output.flush()?;
        Ok(())
    }

    /// Erase the entire buffer by writing empty cells
    pub fn erase(&mut self) {
        self.erase_region(ScreenPos::ZERO, self.size());
    }

    /// Erase a specific region.
    /// Will reset the styles for all the cells as well.
    pub fn erase_region(&mut self, pos: ScreenPos, size: Size) {
        let to_x = (size.width as u16 + pos.x).min(self.size().width as u16);
        let to_y = (size.height as u16 + pos.y).min(self.size().height as u16);

        for x in pos.x.min(to_x)..to_x {
            for y in pos.y.min(to_y)..to_y {
                self.new_buffer.empty(ScreenPos::new(x, y));
            }
        }
    }

    /// Put a char at the given screen position, with a given style.
    /// If the screen position is outside the [`Buffer`]s size then this is
    /// out of bounds and will panic.
    pub fn put(&mut self, c: char, style: Style, pos: ScreenPos) {
        self.new_buffer.put_char(c, style, pos);
    }

    /// Get character and style at a given sceen position
    pub fn get(&self, pos: ScreenPos) -> Option<(char, Style)> {
        self.new_buffer.get(pos)
    }

    /// Draw the changes to the screen
    pub fn render(&mut self, mut output: impl Write) -> Result<()> {
        let changes = diff(&self.old_buffer, &self.new_buffer)?;

        if changes.is_empty() {
            return Ok(());
        }

        draw_changes(&mut output, changes)?;
        output.flush()?;

        self.old_buffer = self.new_buffer.clone();

        Ok(())
    }

    /// Enter an alternative screen.
    /// When using this with stdout it means the output will not persist once the program exits.
    pub fn enter_alt_screen(&self, mut output: impl Write) -> Result<()> {
        output.execute(EnterAlternateScreen)?;
        Ok(())
    }

    /// Enable raw mode: input will not be forwarded to the screen.
    pub fn enable_raw_mode(&self) -> Result<()> {
        enable_raw_mode()?;
        Ok(())
    }

    /// Disable raw mode: input will be forwarded to the screen.
    pub fn disable_raw_mode(&self) -> Result<()> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::buffer::Cell;
    use crate::Style;

    fn make_screen(size: Size, buffer: &mut Vec<u8>) -> Screen {
        let mut screen = Screen::new(buffer, size).unwrap();
        for y in 0..size.height {
            let c = y.to_string().chars().next().unwrap();
            for x in 0..size.width {
                screen.put(c, Style::reset(), ScreenPos::new(x as u16, y as u16));
            }
        }

        screen
    }

    #[test]
    fn render() {
        // Render a character
        let mut render_output = vec![];
        let mut screen = make_screen(Size::new(1, 1), &mut render_output);
        screen.put('x', Style::reset(), ScreenPos::ZERO);
        screen.render(&mut render_output).unwrap();

        let expected = Cell::new('x', Style::reset());
        let actual = screen.new_buffer.inner[0];
        assert_eq!(expected, actual);
    }

    #[test]
    fn erase_region() {
        // Erase a whole region, leaving all cells `empty`
        let mut render_output = vec![];
        let mut screen = make_screen(Size::new(2, 2), &mut render_output);
        screen.render(&mut render_output).unwrap();

        screen.erase_region(ScreenPos::new(1, 1), Size::new(1, 1));
        screen.render(&mut render_output).unwrap();

        let top_left = screen.new_buffer.inner[0];
        assert_eq!(Cell::new('0', Style::reset()), top_left);
        let bottom_right = screen.new_buffer.inner[3];
        assert_eq!(Cell::empty(), bottom_right);
    }

    #[test]
    fn clear_all() {
        // Clear the entire screen, as well as the buffers
        let mut render_output = vec![];
        let mut screen = make_screen(Size::new(1, 1), &mut render_output);
        screen.clear_all(&mut render_output).unwrap();
        let actual = screen.new_buffer.inner[0];
        assert_eq!(Cell::empty(), actual);
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 1 but the index is 4")]
    fn put_outside_of_screen() {
        // Put a character outside of the screen should panic
        let mut screen = make_screen(Size::new(1, 1), &mut vec![]);
        screen.put('x', Style::reset(), ScreenPos::new(2, 2));
        screen.render(&mut vec![]).unwrap();
    }
}
