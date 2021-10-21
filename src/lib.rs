mod colors;
mod errors;
mod window;
mod widgets;
mod strings;

pub use colors::{Color, Colors, Pair};
pub use errors::{Error, Result};
pub use pancurses::{Attributes, Input};
pub use window::{Cursor, Window, Sub, Main};
pub use widgets::ScrollBuffer;
pub use widgets::lines::{Line, Lines, Instruction};
pub use strings::split;

#[macro_export]
macro_rules! panerr {
    ($res:expr, $err:expr) => {
        match $res {
            pancurses::ERR => return Err($err),
            _ => return Ok(()),
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn abs(self) -> Self {
        Pos::new(self.x.abs(), self.y.abs())
    }
}

impl From<(i32, i32)> for Pos {
    fn from((x, y): (i32, i32)) -> Self {
        Self { x, y }
    }
}

impl std::ops::Sub for Pos {
    type Output = Pos;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}
