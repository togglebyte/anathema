use std::ops::{Sub, SubAssign};

use anathema_geometry::{Region, Size};

/// `Constraints` are used to ensure that a widget doesn't size it self outside of a set of given bounds.
/// A constraint can be tight, meaning then minimum and maximum width / height are the same.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Constraints {
    /// Minimum size.
    pub min: Size,
    /// Max size.
    pub max: Size,
}

impl Constraints {
    pub const MAX: Self = Self {
        min: Size::MAX,
        max: Size::MAX,
    };
    pub const ZERO: Self = Self {
        min: Size::ZERO,
        max: Size::ZERO,
    };

    /// Create a set of constraints with a given max width / height.
    /// If `None` is passed for either `max_width` and / or `max_height` then this is qualified as
    /// "unbounded" constraints.
    ///
    /// The `min_width` and `min_height` are zero by default.
    ///
    /// If the `min_width` and the `max_width` are the same the constraints are considered "tight".
    pub fn new(size: Size) -> Self {
        Self {
            min: Size::ZERO,
            max: size,
        }
    }

    /// Create unbounded constraints.
    pub const fn unbounded() -> Self {
        Self {
            min: Size::ZERO,
            max: Size::MAX,
        }
    }

    /// Returns true if the width and height is unbounded.
    pub fn is_unbounded(&self) -> bool {
        self.max == Size::MAX && self.min == Size::ZERO
    }

    /// Returns true if min and max are the same
    pub fn is_tight(&self) -> bool {
        self.max == self.min
    }

    /// Returns true if the width is unbounded.
    pub fn is_width_unbounded(&self) -> bool {
        self.max.width == u32::MAX && self.min.width == 0
    }

    /// Returns true if the height is unbounded.
    pub fn is_height_unbounded(&self) -> bool {
        self.max.height == u32::MAX && self.min.height == 0
    }

    /// Subtract `width` from the max width, as long
    /// as the width isn't unbounded.
    pub fn sub_max_width(&mut self, width: u32) {
        if !self.is_width_unbounded() {
            self.max.width = self.max.width.saturating_sub(width);
            self.min.width = self.min.width.min(self.max.width);
        }
    }

    /// Subtract `height` from the max height, as long
    /// as the height isn't unbounded.
    pub fn sub_max_height(&mut self, height: u32) {
        if !self.is_height_unbounded() {
            self.max.height = self.max.height.saturating_sub(height);
            self.min.height = self.min.height.min(self.max.height);
        }
    }

    /// The given width is clamped to the current max width,
    /// so the new width is not allowed to exceed the current max width
    ///
    /// This makes the width "tight"
    pub fn try_fit_width(&mut self, width: u32) {
        self.max.width = self.max.width.min(width).max(self.min.width);
    }

    /// The given height is clamped to the current max height,
    /// so the new height is not allowed to exceed the current max height
    ///
    /// This makes the height "tight"
    pub fn try_fit_height(&mut self, height: u32) {
        self.max.height = self.max.height.min(height).max(self.min.height);
    }
}

impl From<Size> for Constraints {
    fn from(size: Size) -> Self {
        Self::new(size)
    }
}

impl From<Region> for Constraints {
    fn from(value: Region) -> Self {
        let width = value.to.x - value.from.x;
        assert!(width > 0, "negative width is not allowed");
        let height = value.to.y - value.from.y;
        assert!(height > 0, "negative height is not allowed");

        Self::new(Size::new(width as u32, height as u32))
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self::ZERO
    }
}

impl SubAssign<Size> for Constraints {
    fn sub_assign(&mut self, rhs: Size) {
        if self.is_tight() {
            self.min -= rhs;
        }  

        if !self.is_unbounded() {
            self.max -= rhs;
        }
    }
}

impl Sub<Size> for Constraints {
    type Output = Self;

    fn sub(mut self, rhs: Size) -> Self::Output {
        if self.is_tight() {
            self.min -= rhs;
        }  

        if !self.is_unbounded() {
            self.max -= rhs;
        }

        self
    }
}
