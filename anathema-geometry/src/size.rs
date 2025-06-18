use std::ops::{Add, AddAssign, Sub};

/// Size
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Size {
    /// Width
    pub width: u16,
    /// Height
    pub height: u16,
}

impl Size {
    /// Max size
    pub const MAX: Self = Self::new(u16::MAX, u16::MAX);
    /// Zero size
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new Size
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub const fn area(self) -> usize {
        (self.width * self.height) as usize
    }
}

impl From<(u16, u16)> for Size {
    fn from(parts: (u16, u16)) -> Self {
        Size::new(parts.0, parts.1)
    }
}

impl From<(usize, usize)> for Size {
    fn from(parts: (usize, usize)) -> Self {
        Size::new(parts.0 as u16, parts.1 as u16)
    }
}

impl From<(i32, i32)> for Size {
    fn from(parts: (i32, i32)) -> Self {
        Size::new(parts.0 as u16, parts.1 as u16)
    }
}

impl From<Size> for (i32, i32) {
    fn from(size: Size) -> Self {
        (size.width as i32, size.height as i32)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl Add for Size {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            width: self.width + other.width,
            height: self.height + other.height,
        }
    }
}

impl Sub for Size {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            width: self.width - other.width,
            height: self.height - other.height,
        }
    }
}
