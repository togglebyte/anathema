use super::{Pos2d, Size, Vector};

/// A rectangle composed of two points
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect {
    /// The top left corner of the rect
    pub start: Vector<2>,
    /// The bottom right corner of the rect
    pub end: Vector<2>,
}

impl Rect {
    /// Create new rectangle from the given points
    pub const fn new(start: Pos2d, end: Pos2d) -> Self {
        Self { start, end }
    }

    fn abs(&self) -> Self {
        let min_x = self.start.x().min(self.end.x());
        let min_y = self.start.y().min(self.end.y());

        let max_x = self.start.x().max(self.end.x());
        let max_y = self.start.y().max(self.end.y());

        Self {
            start: Vector::new([min_x, min_y]),
            end: Vector::new([max_x, max_y]),
        }
    }

    /// Create a rect from a top_left position and a size
    pub const fn from_pos(top_left: Pos2d, size: Size) -> Self {
        let bottom_right = Vector::new([size.width + top_left.x(), size.height + top_left.y()]);
        Self::new(top_left, bottom_right)
    }

    /// The centre of the rect as a vector
    pub fn centre(&self) -> Vector<2> {
        let rect = self.abs();
        let width = rect.end.x() - rect.start.x();
        let height = rect.end.y() - rect.start.y();
        Vector::new([width * 0.5 + rect.start.x(), height * 0.5 + rect.start.y()])
    }

    /// Returns true if the position is contained with the rect
    pub fn contains(&self, pos: Vector<2>) -> bool {
        let rect = self.abs();
        pos.x() >= rect.start.x() && pos.y() >= rect.start.y() && pos.x() < rect.end.x() && pos.y() < rect.end.y()
    }

    /// Check if a rect is intersecting another rect
    pub fn intersects(&self, other: Self) -> bool {
        if self.end.x() < other.start.x() || other.end.x() < self.start.x() {
            return false;
        }

        if self.end.y() < other.start.y() || other.end.y() < self.start.y() {
            return false;
        }

        true
    }

    /// Get four positions representing the four corners of the rectangle
    /// ```
    /// # fn rect(rect: Rect) {
    /// let [top_left, top_right, bottom_right, bottom_left] = rect.corners();
    /// # }
    /// ```
    pub fn corners(&self) -> [Pos2d; 4] {
        let top_left = self.start;
        let top_right = Vector::new([self.end.x(), self.start.y()]);
        let bottom_right = Vector::new([self.end.x(), self.end.y()]);
        let bottom_left = Vector::new([self.start.x(), self.end.y()]);

        [top_left, top_right, bottom_right, bottom_left]
    }

    /// Make an axis aligned bounding box from this rectangle with a given rotation
    pub fn aabb(&self, rad: f32) -> Self {
        let mut rect = *self;
        let corners = rect.corners();

        let centre = (rect.start + rect.end) * 0.5;

        for corner in corners {
            let corner = corner - centre;
            let final_corner = corner.rotate(rad) + centre;

            rect.start.set_x(final_corner.x().min(rect.start.x()));
            rect.start.set_y(final_corner.y().min(rect.start.y()));

            rect.end.set_x(final_corner.x().max(rect.end.x()));
            rect.end.set_y(final_corner.y().max(rect.end.y()));
        }

        rect
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contains() {
        let rect = Rect::new([0.0, 0.0].into(), [320.0, 200.0].into());
        let valid = [
            Pos2d::new([0.0, 0.0].into()),
            Pos2d::new([100.0, 100.0].into()),
            Pos2d::new([319.0, 199.0].into()),
        ];

        let invalid = [
            Pos2d::new([320.0, 100.0].into()),
            Pos2d::new([320.0, 200.0].into()),
            Pos2d::new([100.0, 200.0].into()),
        ];

        for v in valid {
            assert!(rect.contains(v));
        }

        for i in invalid {
            assert!(!rect.contains(i));
        }
    }

    #[test]
    fn intersection() {
        let rect = Rect::new([2.0, 2.0].into(), [320.0, 200.0].into());

        let intersecting = [
            // On the left edge of the square
            Rect::new([0.0, 0.0].into(), [2.0, 2.0].into()),
            // On the right edge
            Rect::new([320.0, 200.0].into(), [2.0, 2.0].into()),
            Rect::new([0.0, 0.0].into(), [322.0, 202.0].into()),
            Rect::new([0.0, 0.0].into(), [319.0, 200.0].into()),
            Rect::new([0.0, 0.0].into(), [320.0, 199.0].into()),
            Rect::new([3.0, 0.0].into(), [322.0, 202.0].into()),
            Rect::new([0.0, 3.0].into(), [322.0, 202.0].into()),
        ];

        let not_intersecting = [
            Rect::new([0.0, 0.0].into(), [1.0, 1.0].into()),
            Rect::new([321.0, 190.0].into(), [20.0, 20.0].into()),
            Rect::new([100.0, 210.0].into(), [320.0, 200.0].into()),
        ];

        for v in intersecting {
            assert!(rect.intersects(v));
        }

        for v in not_intersecting {
            assert!(!rect.intersects(v));
        }
    }

    #[test]
    fn make_aabb() {
        let rect = Rect::new([-8.0, -8.0].into(), [8.0, 8.0].into());

        let rot = std::f32::consts::PI / 4.0;
        let res = rect.aabb(rot);
        let expected = Rect {
            start: Vector([-11.313708, -11.313708]),
            end: Vector([11.313708, 11.313708]),
        };
        assert_eq!(expected, res);
    }

    #[test]
    fn centre() {
        let rect = Rect::new([-8.0, -8.0].into(), [8.0, 8.0].into());
        let expected = Pos2d::new([0.0, 0.0]);
        let actual = rect.centre();
        assert_eq!(expected, actual);
    }
}
