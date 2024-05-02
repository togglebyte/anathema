use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

// -----------------------------------------------------------------------------
//   - Generic position -
// -----------------------------------------------------------------------------

/// A position in global space.
/// Can contain negative coordinates
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
}

impl Pos {
    /// Zero
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new instance with the given x and y coordinates
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Pos {
    fn from(val: (i32, i32)) -> Self {
        Self::new(val.0, val.1)
    }
}

impl From<(u16, u16)> for Pos {
    fn from(val: (u16, u16)) -> Self {
        Self::new(val.0 as i32, val.1 as i32)
    }
}

impl From<(usize, usize)> for Pos {
    fn from(val: (usize, usize)) -> Self {
        Self::new(val.0 as i32, val.1 as i32)
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<LocalPos> for Pos {
    type Output = Self;

    fn add(self, rhs: LocalPos) -> Self::Output {
        Pos::new(self.x + rhs.x as i32, self.y + rhs.y as i32)
    }
}

impl Mul<f32> for Pos {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: (self.x as f32 * rhs).round() as i32,
            y: (self.y as f32 * rhs).round() as i32,
        }
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Pos) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Pos) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

// -----------------------------------------------------------------------------
//   - Local position -
// -----------------------------------------------------------------------------

/// Positions in a local space.
/// These coordiantes can not be negative.
/// `0, 0` refers to top left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    /// X coordinate
    pub x: u16,
    /// Y coordinate
    pub y: u16,
}

impl LocalPos {
    /// Zero...
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new set of coordinates in local space
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl Add for LocalPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        LocalPos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Pos> for LocalPos {
    type Output = Self;

    fn add(self, rhs: Pos) -> Self::Output {
        LocalPos {
            x: self.x + rhs.x as u16,
            y: self.y + rhs.y as u16,
        }
    }
}
