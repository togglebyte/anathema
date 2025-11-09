use std::ops::{Add, AddAssign, Deref, DerefMut, Mul, Neg, Sub, SubAssign};

use bytemuck::{Pod, Zeroable};
use glam::Vec2;

use crate::Size;

// -----------------------------------------------------------------------------
//   - Generic position -
// -----------------------------------------------------------------------------

/// A position in global space.
/// Can contain negative coordinates
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Pos(pub(crate) Vec2);

impl Pos {
    /// Zero
    pub const ZERO: Self = Self(Vec2::ZERO);

    /// Create a new instance with the given x and y coordinates
    pub const fn new(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }
}

impl Deref for Pos {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Pos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for Pos {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}

impl From<(f32, f32)> for Pos {
    fn from((x, y): (f32, f32)) -> Self {
        Self::new(x, y)
    }
}

impl From<Vec2> for Pos {
    fn from(pos: Vec2) -> Self {
        Self(pos)
    }
}

impl From<LocalPos> for Pos {
    fn from(pos: LocalPos) -> Self {
        Self(*pos)
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

// -----------------------------------------------------------------------------
//   - Local position -
// -----------------------------------------------------------------------------

/// Positions in a local space.
/// These coordinates should not be negative.
///
/// `0.0, 0.0` refers to top left.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LocalPos(Vec2);

impl LocalPos {
    /// Create a new set of coordinates in local space
    pub const fn new(x: f32, y: f32) -> Self {
        assert!(x >= 0.0, "local position should never be negative");
        Self(Vec2::new(x, y))
    }

    pub const fn to_index(self, width: f32) -> usize {
        (self.0.y * width + self.0.x) as usize
    }
}

impl Deref for LocalPos {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalPos {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl TryFrom<Pos> for LocalPos {
    type Error = ();

    fn try_from(value: Pos) -> Result<Self, Self::Error> {
        if value.x < 0.0 || value.y < 0.0 {
            return Err(());
        }

        Ok(Self::new(value.x, value.y))
    }
}

impl Add for LocalPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(*self * *rhs)
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
