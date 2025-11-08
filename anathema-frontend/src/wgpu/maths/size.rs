use std::ops::{Div, Mul};

use bytemuck::{Pod, Zeroable};

use super::Vector;

/// A size composed of a width and height
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Pod, Zeroable)]
pub struct Size {
    /// The width
    pub width: f32,
    /// The height
    pub height: f32,
}

// -----------------------------------------------------------------------------
//   - Impl -
// -----------------------------------------------------------------------------
impl Size {
    /// A size with the width and height set to zero
    pub const ZERO: Self = Self::new(0.0, 0.0);

    /// Create a instance of a size with a given width and height
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

// -----------------------------------------------------------------------------
//   - Trait impl -
// -----------------------------------------------------------------------------
impl From<[f32; 2]> for Size {
    fn from(value: [f32; 2]) -> Self {
        Self {
            width: value[0],
            height: value[1],
        }
    }
}

impl From<(f32, f32)> for Size {
    fn from(value: (f32, f32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Maths -
// -----------------------------------------------------------------------------
impl Div<Size> for Size {
    type Output = Size;

    fn div(self, rhs: Size) -> Self::Output {
        Size {
            width: self.width / rhs.width,
            height: self.height / rhs.height,
        }
    }
}

impl Mul<Size> for Size {
    type Output = Size;

    fn mul(self, rhs: Size) -> Self::Output {
        Size {
            width: self.width * rhs.width,
            height: self.height * rhs.height,
        }
    }
}

impl Mul<Size> for Vector<2> {
    type Output = Vector<2>;

    fn mul(self, rhs: Size) -> Self::Output {
        Vector([self.0[0] * rhs.width, self.0[1] * rhs.height])
    }
}
