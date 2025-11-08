#![deny(missing_docs)]
//! Geometry and maths

pub use self::rect::Rect;
pub use self::size::Size;
pub use self::vector::Vector;
/// A 4x4 matrix
pub type Mat = self::matrix::SquareMatrix<4>;
/// A vector with two components
pub type Vec2 = self::vector::Vector<2>;
/// A vector with two components representing a position
pub type Pos2d = self::vector::Vector<2>;
/// A vector representing a direction
pub type Dir = self::vector::Vector<2>;

mod matrix;
mod rect;
mod size;
mod vector;
