use std::ops::{Add, Index, Mul, Sub, Neg};

use bytemuck::{Pod, Zeroable};

/// A vector...
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector<const N: usize>(pub(crate) [f32; N]);

unsafe impl<const N: usize> Pod for Vector<N> {
}
unsafe impl<const N: usize> Zeroable for Vector<N> {
}

impl<const N: usize> Default for Vector<N> {
    fn default() -> Self {
        Self::ZERO
    }
}

// -----------------------------------------------------------------------------
//   - Impl -
// -----------------------------------------------------------------------------
impl<const N: usize> Vector<N> {
    /// A vector with all components set to zero
    pub const ZERO: Self = Self([0.0; N]);

    /// Create a new instance of a Vector
    pub const fn new(values: [f32; N]) -> Self {
        Self(values)
    }

    /// Resize the vector to a new size
    pub fn resize<const M: usize>(self) -> Vector<M> {
        let i = M.min(N);
        let mut data = [0.0; M];
        data[..i].copy_from_slice(&self.0[..i]);
        Vector(data)
    }

    /// Calculate the dot product of the vector
    pub fn dot(self, rhs: Self) -> f32 {
        let mut result = 0.0;

        for i in 0..N {
            result += self.0[i] * rhs.0[i];
        }

        result
    }

    #[cfg(test)]
    pub(crate) fn eq_eps(&self, other: Self) -> bool {
        const EPS: f32 = 1.19209290e-06_f32;

        for i in 0..N {
            let val = (self.0[i] - other.0[i]).abs();
            if val > EPS {
                return false;
            }
        }

        true
    }

    /// Get the first value of the vector (generally used to represent the x axis)
    pub const fn x(&self) -> f32 {
        assert!(N > 0, "there should never be a zero sized vector");
        self.0[0]
    }

    /// Get the first value of the vector (generally used to represent the x axis)
    pub const fn y(&self) -> f32 {
        assert!(N > 1, "there should never be a zero sized vector");
        self.0[1]
    }

    /// Set the first value of the vector (generally x)
    pub fn set_x(&mut self, x: f32) {
        assert!(N > 0, "there should never be a zero sized vector");
        self.0[0] = x;
    }

    /// Set the second value of the vector
    pub fn set_y(&mut self, y: f32) {
        self.0[1] = y;
    }
}

impl Vector<2> {
    /// Create a vector from an x and y value
    pub const fn from_x_y(x: f32, y: f32) -> Self {
        Self([x, y])
    }

    /// Create a rotation vector from self with the given radians
    pub fn rotate(&self, rad: f32) -> Self {
        let x = self.0[0];
        let y = self.0[1];

        Self([x * rad.cos() - y * rad.sin(), x * rad.sin() + y * rad.cos()])
    }
}

// -----------------------------------------------------------------------------
//   - Maths -
// -----------------------------------------------------------------------------
impl<const N: usize> Mul for Vector<N> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = Self::ZERO;

        for i in 0..N {
            result.0[i] = self.0[i] * rhs.0[i];
        }

        result
    }
}

impl<const N: usize> Mul<f32> for Vector<N> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = Self::default();

        for i in 0..N {
            result.0[i] = self.0[i] * rhs;
        }

        result
    }
}

impl<const N: usize> Add for Vector<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Self::ZERO;

        for i in 0..N {
            result.0[i] = self.0[i] + rhs.0[i];
        }

        result
    }
}

impl<const N: usize> Add<f32> for Vector<N> {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        let mut result = Self::ZERO;

        for i in 0..N {
            result.0[i] = self.0[i] + rhs;
        }

        result
    }
}

impl<const N: usize> Sub for Vector<N> {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.0[i] -= rhs.0[i];
        }

        self
    }
}

// -----------------------------------------------------------------------------
//   - From -
// -----------------------------------------------------------------------------
impl<const N: usize> From<f32> for Vector<N> {
    fn from(value: f32) -> Self {
        Self([value; N])
    }
}

impl<const N: usize> From<[f32; N]> for Vector<N> {
    fn from(value: [f32; N]) -> Self {
        Self(value)
    }
}

// -----------------------------------------------------------------------------
//   - Index -
// -----------------------------------------------------------------------------
impl<const N: usize> Index<usize> for Vector<N> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rotate_vec() {
        let vec = Vector::new([3.0, 3.0]);
        let rotated = vec.rotate(std::f32::consts::PI);
        eprintln!("{rotated:#?}");
        assert!(rotated.eq_eps(Vector::new([-3.0, -3.0])));
    }
}
