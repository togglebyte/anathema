use std::ops::{Add, AddAssign, Div, Mul, Sub};

use bytemuck::{Pod, Zeroable};
use glam::Vec2;

/// Size
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Default, Pod, Zeroable)]
pub struct Size {
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Size {
    /// Max size
    pub const MAX: Self = Self::new(f32::MAX, f32::MAX);
    /// Zero one
    pub const ONE: Self = Self::new(1.0, 1.0);
    /// Zero size
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Create a new Size
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// The area
    pub const fn area(self) -> f32 {
        self.width * self.height
    }

    /// Convert the size into a vector
    pub fn to_vec(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

impl From<(u16, u16)> for Size {
    fn from((width, height): (u16, u16)) -> Self {
        Size::new(width as f32, height as f32)
    }
}

impl From<(f32, f32)> for Size {
    fn from((width, height): (f32, f32)) -> Self {
        Size::new(width, height)
    }
}

impl From<[f32; 2]> for Size {
    fn from([width, height]: [f32; 2]) -> Self {
        Self::new(width, height)
    }
}

impl From<Size> for [f32; 2] {
    fn from(value: Size) -> Self {
        [value.width, value.height]
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

impl Mul for Size {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            width: self.width * other.width,
            height: self.height * other.height,
        }
    }
}

impl Mul<f32> for Size {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
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
