use std::ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, Neg, Sub, SubAssign};

use crate::Size;

// -----------------------------------------------------------------------------
//   - Generic position -
// -----------------------------------------------------------------------------

/// A position in global space.
/// Can contain negative coordinates
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    /// Zero
    pub const ZERO: Self = Self { x: 0, y: 0 };

    /// Create a new instance with the given x and y coordinates
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn to_usize(&self) -> (usize, usize) {
        (self.x as usize, self.y as usize)
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<LocalPos> for Pos {
    fn from(pos: LocalPos) -> Self {
        Self {
            x: pos.x as i32,
            y: pos.y as i32,
        }
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for Pos {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl Mul<Size> for Pos {
    type Output = Self;

    fn mul(self, rhs: Size) -> Self::Output {
        Self {
            x: self.x * rhs.width as i32,
            y: self.y * rhs.height as i32,
        }
    }
}

impl Div<Size> for Pos {
    type Output = Self;

    fn div(self, rhs: Size) -> Self::Output {
        Self {
            x: self.x / rhs.width as i32,
            y: self.y / rhs.height as i32,
        }
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<Size> for Pos {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Self {
            x: self.x + rhs.width as i32,
            y: self.y + rhs.height as i32,
        }
    }
}

impl Add<LocalPos> for Pos {
    type Output = Self;

    fn add(self, rhs: LocalPos) -> Self::Output {
        Self {
            x: self.x + rhs.x as i32,
            y: self.y + rhs.y as i32,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Local position -
// -----------------------------------------------------------------------------

/// Positions in a local space.
/// These coordinates should not be negative.
///
/// `0.0, 0.0` refers to top left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LocalPos {
    x: u32,
    y: u32,
}

impl LocalPos {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    /// Create a new set of coordinates in local space
    pub const fn new(x: u32, y: u32) -> Self {
        Self {
            x,
            y,
        }
    }

    pub const fn to_index(self, width: u32) -> usize {
        (self.y * width + self.x) as usize
    }
}

impl TryFrom<Pos> for LocalPos {
    type Error = ();

    fn try_from(value: Pos) -> Result<Self, Self::Error> {
        if value.x < 0 || value.y < 0 {
            return Err(());
        }

        Ok(Self::new(value.x as u32, value.y as u32))
    }
}

impl Add for LocalPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for LocalPos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn index_from_coords() {
        let width = 20.0;

        let actual = LocalPos::ZERO.to_index(width);
        let expected = 0;
        assert_eq!(expected, actual);

        let actual = LocalPos::new(10, 0).to_index(width);
        let expected = 10;
        assert_eq!(expected, actual);

        let actual = LocalPos::new(4, 20).to_index(width);
        let expected = (width * width) as usize + 4;
        assert_eq!(expected, actual);
    }
}
