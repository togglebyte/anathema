use std::ops::{Add, AddAssign, SubAssign, Div, Mul, Sub};

/// Size
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct Size {
    /// Width
    pub width: u32,
    /// Height
    pub height: u32,
}

impl Size {
    /// Max size
    pub const MAX: Self = Self::new(u32::MAX, u32::MAX);
    /// Zero one
    pub const ONE: Self = Self::new(1, 1);
    /// Zero size
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new Size
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// The area
    pub const fn area(self) -> u32 {
        self.width * self.height
    }
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Size::new(width as u32, height as u32)
    }
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size::new(width, height)
    }
}

impl From<u32> for Size {
    fn from(width_height: u32) -> Self {
        Size::new(width_height, width_height)
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
        self.width += rhs.width;
        self.height += rhs.height;
    }
}

impl SubAssign<Size> for Size {
    fn sub_assign(&mut self, rhs: Size) {
        self.width -= rhs.width;
        self.height -= rhs.height;
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

impl Mul for Size {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            width: self.width * other.width,
            height: self.height * other.height,
        }
    }
}

impl Mul<u32> for Size {
    type Output = Self;

    fn mul(self, other: u32) -> Self {
        Self {
            width: self.width * other,
            height: self.height * other,
        }
    }
}

impl Div for Size {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            width: self.width / other.width,
            height: self.height / other.height,
        }
    }
}
