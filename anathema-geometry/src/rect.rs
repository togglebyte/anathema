use crate::{Pos, Size};

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub start: Pos,
    pub end: Pos,
}

impl Rect {
    pub const ZERO: Self = Self {
        start: Pos::ZERO,
        end: Pos::ZERO,
    };
}

impl From<(Pos, Size)> for Rect {
    fn from((start, size): (Pos, Size)) -> Self {
        let end = Pos {
            x: start.x + size.width as i32,
            y: start.y + size.height as i32,
        };

        Self { start, end }
    }
}
