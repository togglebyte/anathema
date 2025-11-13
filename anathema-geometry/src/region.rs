use crate::{Pos, Size};

/// A normalized region in global space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Region {
    /// The starting position of the region
    pub from: Pos,
    /// The end position of the region
    pub to: Pos,
}

impl Region {
    /// Zero...
    pub const ZERO: Self = Self::new(Pos::ZERO, Pos::ZERO);

    /// Create a new instance of a region.
    pub const fn new(from: Pos, to: Pos) -> Self {
        assert!(from.0.x <= to.0.x && from.0.y <= to.0.y, "region is non-normalized");
        Self { from, to }
    }

    /// Check if another region is intersecting with this region
    pub const fn intersects(&self, other: &Region) -> bool {
        if other.to.0.x <= self.from.0.x || other.from.0.x >= self.to.0.x {
            return false;
        }

        if other.from.0.y >= self.to.0.y || other.to.0.y <= self.from.0.y {
            return false;
        }

        true
    }

    /// Get the size of the region
    pub const fn size(&self) -> Size {
        Size::new(self.to.0.x - self.from.0.x, self.to.0.y - self.from.0.y)
    }

    /// Get the position of the region.
    /// This is equal to the top left corner of the region
    pub const fn pos(&self) -> Pos {
        self.from
    }

    /// Move the region to a new position
    pub fn set_pos(&mut self, pos: Pos) {
        *self.from += *pos;
        *self.to += *pos;
    }

    /// Resize the region
    pub fn resize(&mut self, size: Size) {
        self.to.x = self.from.x + size.width;
        self.to.y = self.from.y + size.height;
    }

    /// Create a new region by intersecting two regions
    pub fn intersect_with(self, other: &Region) -> Self {
        // There is no intersection, making this a zero sized region
        // as there is no space in between
        if !self.intersects(other) {
            return Self::ZERO;
        }

        let from_x = self.from.x.max(other.from.x);
        let from_y = self.from.y.max(other.from.y);

        let to_x = self.to.x.min(other.to.x);
        let to_y = self.to.y.min(other.to.y);

        Region::new(Pos::new(from_x, from_y), Pos::new(to_x, to_y))
    }

    /// Check if a region contains a position.
    /// The check is exclusive, so a region from 0,0 to 10, 10 contains `Pos::ZERO`
    /// but not `Pos::new(10.0, 10.0)`
    pub const fn contains(&self, pos: Pos) -> bool {
        pos.0.x >= self.from.0.x && pos.0.x < self.to.0.x && pos.0.y >= self.from.0.y && pos.0.y < self.to.0.y
    }

    /// Check if a region contains a position.
    /// The check is inclusive, so a region from 0,0 to 10, 10 contains `Pos::ZERO`
    /// as well as `Pos::new(10.0, 10.0)`
    pub const fn icontains(&self, pos: Pos) -> bool {
        pos.0.x >= self.from.0.x && pos.0.x <= self.to.0.x && pos.0.y >= self.from.0.y && pos.0.y <= self.to.0.y
    }

    /// Constrain a region to fit within another region
    pub fn shrink_to_fit(&mut self, other: &Region) {
        self.from.x = self.from.x.max(other.from.x);
        self.from.y = self.from.y.max(other.from.y);
        self.to.x = self.to.x.min(other.to.x);
        self.to.y = self.to.y.min(other.to.y);
    }
}

impl From<(Pos, Size)> for Region {
    fn from((from, size): (Pos, Size)) -> Self {
        let to = Pos::new(from.x + size.width, from.y + size.height);
        Self::new(from, to)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn region_inersect() {
        let a = Region::new(Pos::ZERO, Pos::new(10.0, 10.0));
        let b = Region::new(Pos::new(5.0, 5.0), Pos::new(8.0, 8.0));
        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
    }

    #[test]
    fn region_contains() {
        let a = Region::new(Pos::ZERO, Pos::new(10.0, 10.0));
        assert!(a.contains(Pos::ZERO));
        assert!(a.contains(Pos::new(9.0, 9.0)));
        assert!(!a.contains(Pos::new(10.0, 10.0)));
    }

    #[test]
    fn constrain_region() {
        let inner = Region::from((Pos::ZERO, Size::new(10.0, 10.0)));
        let mut outer = Region::from((Pos::ZERO, Size::new(100.0, 100.0)));
        outer.shrink_to_fit(&inner);
        let expected = inner;
        assert_eq!(expected, outer);
    }
}
