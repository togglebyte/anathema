use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use bytemuck::Zeroable;

use crate::vector::Vector;
use crate::Size;

// -----------------------------------------------------------------------------
//   - Generic position -
// -----------------------------------------------------------------------------

/// A position in global space.
/// Can contain negative coordinates
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Zeroable)]
pub struct Pos {
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl Pos {
    /// Zero
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Create a new instance with the given x and y coordinates
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert the position into a vector
    pub fn to_vec(self) -> Vector<2> {
        Vector::new([self.x, self.y])
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(f32, f32)> for Pos {
    fn from(val: (f32, f32)) -> Self {
        Self::new(val.0, val.1)
    }
}

impl From<Pos> for [f32; 2] {
    fn from(value: Pos) -> Self {
        [value.x, value.y]
    }
}

impl From<LocalPos> for Pos {
    fn from(LocalPos { x, y }: LocalPos) -> Self {
        Self::new(x, y)
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<Size> for Pos {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        Pos::new(self.x + rhs.width, self.y + rhs.height)
    }
}

impl Add<LocalPos> for Pos {
    type Output = Self;

    fn add(self, rhs: LocalPos) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<LocalPos> for Pos {
    type Output = Self;

    fn sub(self, rhs: LocalPos) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f32> for Pos {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Mul<Vector<2>> for Pos {
    type Output = Self;

    fn mul(self, rhs: Vector<2>) -> Self::Output {
        Self {
            x: self.x * rhs.x(),
            y: self.y * rhs.y(),
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

impl Neg for Pos {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
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
    /// X coordinate
    pub x: f32,
    /// Y coordinate
    pub y: f32,
}

impl LocalPos {
    /// Zero...
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Create a new set of coordinates in local space
    pub const fn new(x: f32, y: f32) -> Self {
        assert!(x >= 0.0, "local position should never be negative");
        Self { x, y }
    }

    pub const fn to_index(self, width: f32) -> usize {
        (self.y * width + self.x) as usize
    }
}

impl From<(f32, f32)> for LocalPos {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl TryFrom<Pos> for LocalPos {
    type Error = ();

    fn try_from(value: Pos) -> Result<Self, Self::Error> {
        if value.x < 0.0 || value.y < 0.0 {
            return Err(());
        }

        Ok(Self { x: value.x, y: value.y })
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
        let width = 20;

        let actual = LocalPos::new(0, 0).to_index(width);
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
