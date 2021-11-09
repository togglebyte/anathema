use std::cell::RefCell;
use std::time::Duration;

use pancurses::Window as PanWindow;
use pancurses::{curs_set, endwin, initscr, napms, noraw, raw, start_color, ToChtype, ACS_HLINE, ACS_VLINE};

use super::{panerr, Attribute, Error, Input, Pair, Pos, Result, Size};

thread_local! {
    static MAIN: RefCell<State> = RefCell::new(State::FirstTime);
}

#[derive(Debug, Copy, Clone)]
enum State {
    FirstTime,
    Init,
    Uninit,
}

#[derive(Debug, Copy, Clone)]
pub enum Cursor {
    Hide,
    Normal,
    HighlyVisible,
}

pub struct Main;
pub struct Sub;

pub struct Window<T> {
    inner: PanWindow,
    _wintype: T,
}

impl<T> Window<T> {
    /// Get the current size of the window
    pub fn size(&self) -> Size {
        let (y, x) = self.inner.get_max_yx();
        Size { width: x, height: y }
    }

    /// Create a new child window at a given position with a given size
    pub fn new_window(&self, pos: Pos, size: Size) -> Result<Window<Sub>> {
        let inst = Window {
            inner: self.inner.subwin(size.height, size.width, pos.y, pos.x).map_err(|_| Error::CreateWindow)?,
            _wintype: Sub,
        };
        Ok(inst)
    }

    /// Move the window to a new position.
    pub fn move_win(&mut self, pos: Pos) -> Result<()> {
        let res = self.inner.mvwin(pos.y, pos.x);
        panerr!(res, Error::MoveWindow);
    }

    /// Resize the window
    pub fn resize(&mut self, size: Size) -> Result<()> {
        let res = self.inner.resize(size.height, size.width);
        panerr!(res, Error::Resize);
    }

    /// Set an attribute
    pub fn set_attribute(&self, attribute: impl Into<u32>) -> Result<()> {
        let res = self.inner.attron(attribute.into());
        panerr!(res, Error::AttributeSet);
    }

    pub fn set_color(&self, pair: Pair) -> Result<()> {
        self.set_attribute(pair)?;
        Ok(())
    }

    pub fn enable_style(&self, attribute: Attribute) -> Result<()> {
        let res = self.inner.attron(attribute);
        panerr!(res, Error::AttributeSet);
    }

    pub fn disable_style(&self, attribute: Attribute) -> Result<()> {
        let res = self.inner.attroff(attribute);
        panerr!(res, Error::AttributeSet);
    }

    pub fn reset_style(&self) -> Result<()> {
        let (attrs, _col) = self.inner.attrget();
        let res = self.inner.attroff(attrs);
        panerr!(res, Error::AttributeSet);
    }

    /// Uses `addstr`, **NOT** printw as it is rather unsafe.
    pub fn print(&self, s: impl AsRef<str>) -> Result<()> {
        let res = self.inner.addstr(s.as_ref());
        panerr!(res, Error::Print(s.as_ref().into()));
    }

    /// Print a string at a position
    pub fn print_at(&self, pos: Pos, s: impl AsRef<str>) -> Result<()> {
        let res = self.inner.mvaddstr(pos.y, pos.x, s.as_ref());
        panerr!(res, Error::PrintAt(s.as_ref().into(), pos));
    }

    /// Draw what's in the virtual buffer to the screen
    pub fn refresh(&self) -> Result<()> {
        let res = self.inner.refresh();
        panerr!(res, Error::Refresh);
    }

    /// Clear the virtual buffer
    pub fn erase(&self) -> Result<()> {
        let res = self.inner.erase();
        panerr!(res, Error::Erase);
    }

    pub fn add_char(&self, c: char) -> Result<()> {
        let res = self.inner.addch(c);
        panerr!(res, Error::AddChar(c));
    }

    pub fn add_char_at(&self, pos: Pos, c: char) -> Result<()> {
        let res = self.inner.mvaddch(pos.y, pos.x, c);
        panerr!(res, Error::MoveAddChar(c, pos));
    }

    pub fn enable_scroll(&self) -> Result<()> {
        let res = self.inner.scrollok(true);
        panerr!(res, Error::EnableScrolling);
    }

    pub fn disable_scroll(&self) -> Result<()> {
        let res = self.inner.scrollok(false);
        panerr!(res, Error::DisableScrolling);
    }

    pub fn set_scroll_region(&self, top_y: i32, bottom_y: i32) -> Result<()> {
        let res = self.inner.setscrreg(top_y, bottom_y);
        panerr!(res, Error::SetScrollRegion);
    }

    pub fn get_cursor(&self) -> Pos {
        let (y, x) = self.inner.get_cur_yx();
        Pos::new(x, y)
    }

    pub fn move_cursor(&self, pos: impl Into<Pos>) -> Result<()> {
        let pos = pos.into();
        let res = self.inner.mv(pos.y, pos.x);
        panerr!(res, Error::MoveCursor(pos));
    }

    pub fn draw_box(&self) {
        self.inner.draw_box(ACS_VLINE(), ACS_HLINE());
    }

    pub fn border_thin(&self) {
        self.inner.border(
            '│', // left_side: T,
            '│', // right_side: T,
            '─', // top_side: T,
            '─', // bottom_side: T,
            '┌', // top_left_corner: T,
            '┐', //top_right_corner: T,
            '└', //bottom_left_corner: T,
            '┘', //bottom_right_corner: T
        );
    }

    pub fn horizontal_line<C: ToChtype>(&self, c: C, len: i32) -> Result<()> {
        let res = self.inner.hline(c, len);
        panerr!(res, Error::HorizontalLine);
    }

    pub fn horizontal_line_at<C: ToChtype>(&self, pos: Pos, c: C, len: i32) -> Result<()> {
        self.move_cursor(pos)?;
        self.horizontal_line(c, len)?;
        Ok(())
    }

    pub fn contains(&self, pos: Pos) -> bool {
        let size = self.size();
        pos.x >= 0 
        && pos.x < size.width
        && pos.y >= 0
        && pos.y < size.height
    }
}

// -----------------------------------------------------------------------------
//     - Main window -
//     There should only be one of these at a time
// -----------------------------------------------------------------------------
impl Window<Main> {
    pub fn main(no_echo: bool) -> Result<Self> {
        if let State::Init = MAIN.with(|win_init| *win_init.borrow()) {
            return Err(Error::InitMain);
        }

        let inst = Self { inner: initscr(), _wintype: Main };

        if let State::FirstTime = MAIN.with(|win_init| *win_init.borrow()) {
            if let pancurses::ERR = start_color() {
                return Err(Error::StartColor);
            }
            if no_echo {
                if let pancurses::ERR = pancurses::noecho() {
                    return Err(Error::NoEcho);
                }
            }
        }

        MAIN.with(|win_init| win_init.replace(State::Init));

        Ok(inst)
    }

    /// Don't block on input `get_input`
    pub fn no_delay(&self, no_delay: bool) -> Result<()> {
        let res = self.inner.nodelay(no_delay);
        panerr!(res, Error::NoDelay);
    }

    pub fn set_cursor_visibility(&self, cursor: Cursor) -> Result<()> {
        let res = match cursor {
            Cursor::Hide => curs_set(0),
            Cursor::Normal => curs_set(1),
            Cursor::HighlyVisible => curs_set(2),
        };
        panerr!(res, Error::SetCursor(cursor));
    }

    pub fn nap(&self, dur: Duration) -> Result<()> {
        let res = napms(dur.as_millis() as i32);
        panerr!(res, Error::Nap);
    }

    pub fn get_input(&self) -> Option<Input> {
        self.inner.getch()
    }

    pub fn enable_raw(&self) {
        raw();
    }

    pub fn disable_raw(&self) {
        noraw();
    }
}

impl Drop for Main {
    fn drop(&mut self) {
        MAIN.with(|win_init| win_init.replace(State::Uninit));
        endwin();
    }
}
