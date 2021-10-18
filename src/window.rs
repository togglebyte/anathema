use std::cell::RefCell;
use std::time::Duration;

use pancurses::Window as PanWindow;
use pancurses::{curs_set, endwin, initscr, napms, noraw, raw, start_color};

use super::{panerr, Attributes, Error, Input, Pair, Pos, Result, Size};

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
    pub fn size(&self) -> Size {
        let (y, x) = self.inner.get_max_yx();
        Size { width: x, height: y }
    }

    pub fn new_window(&self, pos: Pos, size: Size) -> Result<Window<Sub>> {
        let inst = Window {
            inner: self.inner.subwin(size.height, size.width, pos.y, pos.x).map_err(|_| Error::CreateWindow)?,
            _wintype: Sub,
        };
        Ok(inst)
    }

    pub fn set_attribute(&self, attribute: impl Into<u32>) -> Result<()> {
        let res = self.inner.attrset(attribute.into());
        panerr!(res, Error::AttributeSet);
    }

    pub fn set_color(&self, pair: Pair) -> Result<()> {
        self.set_attribute(pair)?;
        Ok(())
    }

    pub fn set_attributes(&self, attributes: Attributes) -> Result<()> {
        self.set_attribute(attributes)?;
        Ok(())
    }

    /// Uses `addstr`, **NOT** printw as it is rather unsafe.
    pub fn print(&self, s: impl AsRef<str>) -> Result<()> {
        let res = self.inner.addstr(s.as_ref());
        panerr!(res, Error::Print(s.as_ref().into()));
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
        panerr!(res, Error::Erase);
    }

    pub fn border(&self) {
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
