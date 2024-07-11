use crate::{Pos, Size};

/// A region in global space
#[derive(Debug, Clone, Copy)]
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
        Self { from, to }
    }

    /// Check if another region is intersecting with this region
    pub const fn intersects(&self, other: &Region) -> bool {
        if other.to.x < self.from.x || other.from.x >= self.to.x {
            return false;
        }

        if other.to.y < self.from.y || other.from.y >= self.to.y {
            return false;
        }

        true
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
    /// Regions are inclusive, so a region from 0,0 to 10, 10 contains both Pos::ZERO and
    /// Pos::New(10, 10)
    pub const fn contains(&self, pos: Pos) -> bool {
        pos.x >= self.from.x && pos.x < self.to.x && pos.y >= self.from.y && pos.y < self.to.y
    }

    /// Constrain a region to fit within another region
    pub fn constrain(&mut self, other: &Region) {
        self.from.x = self.from.x.max(other.from.x);
        self.from.y = self.from.y.max(other.from.y);
        self.to.x = self.to.x.min(other.to.x);
        self.to.y = self.to.y.min(other.to.y);
    }
}

impl From<(Pos, Size)> for Region {
    fn from((from, size): (Pos, Size)) -> Self {
        let to = Pos::new(from.x + size.width as i32, from.y + size.height as i32);
        Self::new(from, to)
    }
}
