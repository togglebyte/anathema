use std::fmt::Display;
use std::ops::{Add, Index, IndexMut, Mul};

use bytemuck::{Pod, Zeroable};

use super::{Size, Vector};

/// Column major matrix
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SquareMatrix<const N: usize>([[f32; N]; N]);

impl<const N: usize> Default for SquareMatrix<N> {
    fn default() -> Self {
        Self([[0.0; N]; N])
    }
}

// -----------------------------------------------------------------------------
//   - Index -
// -----------------------------------------------------------------------------
impl<const N: usize> Index<(usize, usize)> for SquareMatrix<N> {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl<const N: usize> IndexMut<(usize, usize)> for SquareMatrix<N> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

// -----------------------------------------------------------------------------
//   - From -
// -----------------------------------------------------------------------------
impl<const N: usize> From<[[f32; N]; N]> for SquareMatrix<N> {
    fn from(value: [[f32; N]; N]) -> Self {
        Self(value)
    }
}

// -----------------------------------------------------------------------------
//   - Display -
// -----------------------------------------------------------------------------
impl<const N: usize> Display for SquareMatrix<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        assert!(N > 0, "A matrix is not allowed to be zero sized");

        let mut numbers = vec![];
        for col in 0..N {
            for row in 0..N {
                let value = (self.0[col][row] * 1000.0).round() / 1000.0;
                numbers.push(value.to_string());
            }
        }

        let col_width = numbers
            .iter()
            .map(|n| n.len())
            .max()
            .expect("a matrix is never zero sized");

        writeln!(f, "┌{} ┐", " ".repeat(N * (1 + col_width)))?;
        for col in 0..N {
            write!(f, "│")?;
            for row in 0..N {
                let index = row * N + col;
                write!(f, " {:>col_width$}", numbers[index])?;
            }
            writeln!(f, " │")?;
        }
        writeln!(f, "└{} ┘", " ".repeat(N * (1 + col_width)))
    }
}

// -----------------------------------------------------------------------------
//   - Byte muck -
// -----------------------------------------------------------------------------
unsafe impl<const N: usize> Zeroable for SquareMatrix<N> {
}
unsafe impl<const N: usize> Pod for SquareMatrix<N> {
}

// -----------------------------------------------------------------------------
//   - Impl -
// -----------------------------------------------------------------------------
impl<const N: usize> SquareMatrix<N> {
    /// Create a matrix with all columns and rows set to zero
    pub const ZERO: Self = Self([[0.0; N]; N]);

    /// Create an identity matrix
    pub fn identity() -> Self {
        let mut this = Self::ZERO;
        for i in 0..N {
            this.0[i][i] = 1.0;
        }
        this
    }

    /// Create a new matrix from a translation vector
    pub fn from_translation<const M: usize>(pos: Vector<M>) -> Self {
        let mut mat = Self::identity();

        let max = M.min(N);

        for i in 0..max {
            mat[(N - 1, i)] = pos[i];
        }

        mat
    }

    /// Create a new matrix from a size representing the scale
    pub fn from_scale(scale: Size) -> Self {
        let mut mat = Self::identity();

        mat[(0, 0)] = scale.width;
        mat[(1, 1)] = scale.height;

        mat
    }

    /// Create a new rotation matrix
    pub fn from_rotation(rad: f32) -> Self {
        let mut mat = Self::identity();

        mat[(0, 0)] = rad.cos();
        mat[(0, 1)] = rad.sin();
        mat[(1, 0)] = -rad.sin();
        mat[(1, 1)] = rad.cos();

        mat
    }

    /// Create a new translation matrix based on a new position
    pub fn translate<const M: usize>(&self, pos: Vector<M>) -> Self {
        let translation = Self::from_translation(pos);
        *self * translation
    }

    /// Create a new scaling matrix based on the current matrix and a new scale
    pub fn scale(&self, scale: Size) -> Self {
        let scale = Self::from_scale(scale);
        *self * scale
    }

    /// Get the current translation of a given matrix
    pub const fn translation(&self) -> Vector<N> {
        Vector::new(self.0[N - 1])
    }
}

impl SquareMatrix<4> {
    /// Create an orthographic projection matrix
    pub fn ortho(left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) -> Self {
        let mut this = Self::ZERO;

        this.0[0][0] = 2.0 / (right - left);
        this.0[3][0] = -(right + left) / (right - left);

        this.0[1][1] = 2.0 / (top - bottom);
        this.0[3][1] = -(top + bottom) / (top - bottom);

        this.0[2][2] = -2.0 / (far - near);
        this.0[3][2] = -(far + near) / (far - near);

        this.0[3][3] = 1.0;

        this
    }
}

// -----------------------------------------------------------------------------
//   - Maths -
// -----------------------------------------------------------------------------
impl<const N: usize> Mul for SquareMatrix<N> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut new = Self::default();

        for i in 0..N {
            for j in 0..N {
                for k in 0..N {
                    new.0[j][i] += self.0[k][i] * other.0[j][k];
                }
            }
        }

        new
    }
}

impl<const N: usize> Add for SquareMatrix<N> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = Self::default();

        for col in 0..N {
            for row in 0..N {
                new.0[col][row] = self.0[col][row] + other.0[col][row];
            }
        }

        new
    }
}

impl<const N: usize, const M: usize> Add<Vector<M>> for SquareMatrix<N> {
    type Output = SquareMatrix<N>;

    fn add(mut self, rhs: Vector<M>) -> Self::Output {
        for i in 0..M.min(N) {
            self.0[N - 1][i] += rhs.0[i];
        }

        self
    }
}

impl<const N: usize, const M: usize> Mul<Vector<M>> for SquareMatrix<N> {
    type Output = Vector<M>;

    fn mul(self, other: Vector<M>) -> Self::Output {
        let mut new = Self::Output::default();

        for col in 0..N.min(M) {
            for row in 0..N.min(M) {
                new.0[col] += self.0[col][row] * other.0[row];
            }
        }

        new
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Pos2d;

    #[test]
    fn mul() {
        let lhs = SquareMatrix::from([[1.0, 4.0, 0.0], [2.0, 5.0, 0.0], [3.0, 6.0, 0.0]]);
        let rhs = SquareMatrix::from([[7.0, 9.0, 11.0], [8.0, 10.0, 12.0], [0.0; 3]]);

        let result = lhs * rhs;
        assert_eq!(result[(0, 0)], 58.0);
        assert_eq!(result[(0, 1)], 139.0);
        assert_eq!(result[(1, 0)], 64.0);
        assert_eq!(result[(1, 1)], 154.0);
    }

    #[test]
    fn translate() {
        let identity = SquareMatrix::<4>::identity();
        let pos = Pos2d::from([1.0, 2.0]);

        let translation = identity.translate(pos);

        let expected = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 2.0, 0.0, 1.0],
        ]
        .into();

        eprintln!("{expected}");
        assert_eq!(translation, expected);
    }

    #[test]
    fn scale() {
        let identity = SquareMatrix::<4>::identity();
        let size = Size::from((3.0, 2.0));

        let scale = identity.scale(size);

        let expected = [
            [3.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
        .into();

        eprintln!("{expected}");
        assert_eq!(scale, expected);
    }
}
