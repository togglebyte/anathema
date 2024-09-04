use anathema_geometry::Rect;

use crate::layout::Size;

/// `Constraints` are used to ensure that a widget doesn't size it self outside of a set of given bounds.
/// A constraint can be tight, meaning then minimum and maximum width / height are the same.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Constraints {
    /// Minimum width.
    pub min_width: usize,
    /// Minimum height.
    pub min_height: usize,
    /// Max width.
    max_width: usize,
    /// Max height.
    max_height: usize,
}

impl Constraints {
    pub const ZERO: Self = Self {
        min_width: 0,
        min_height: 0,
        max_width: 0,
        max_height: 0,
    };

    pub fn max_height(&self) -> usize {
        self.max_height
    }

    pub fn max_width(&self) -> usize {
        self.max_width
    }

    /// Subtract `width` from the max width, as long
    /// as the width isn't unbounded.
    pub fn sub_max_width(&mut self, width: usize) {
        if self.max_width < usize::MAX {
            self.max_width = self.max_width.saturating_sub(width);
            self.min_width = self.min_width.min(self.max_width);
        }
    }

    /// Subtract `height` from the max height, as long
    /// as the height isn't unbounded.
    pub fn sub_max_height(&mut self, height: usize) {
        if self.max_height < usize::MAX {
            self.max_height = self.max_height.saturating_sub(height);
            self.min_height = self.min_height.min(self.max_height);
        }
    }

    /// Create a set of constraints with a given max width / height.
    /// If `None` is passed for either `max_width` and / or `max_height` then this is qualified as
    /// "unbounded" constraints.
    ///
    /// The `min_width` and `min_height` are zero by default.
    ///
    /// If the `min_width` and the `max_width` are the same the constraints are considered "tight".
    pub fn new(max_width: impl Into<Option<usize>>, max_height: impl Into<Option<usize>>) -> Self {
        let max_width = max_width.into().unwrap_or(usize::MAX);
        let max_height = max_height.into().unwrap_or(usize::MAX);
        Self {
            min_width: 0,
            min_height: 0,
            max_width,
            max_height,
        }
    }

    /// Create unbounded constraints.
    pub fn unbounded() -> Self {
        Self {
            min_width: 0,
            min_height: 0,
            max_width: usize::MAX,
            max_height: usize::MAX,
        }
    }

    /// Create unbounded height
    pub fn unbound_height(&mut self) {
        self.max_height = usize::MAX;
    }

    /// Create unbounded width
    pub fn unbound_width(&mut self) {
        self.max_width = usize::MAX;
    }

    /// Returns true if the width and height is unbounded.
    pub fn is_unbounded(&self) -> bool {
        self.is_width_unbounded() && self.is_height_unbounded()
    }

    /// Returns true if the width is unbounded.
    pub fn is_width_unbounded(&self) -> bool {
        self.max_width == usize::MAX
    }

    /// Returns true if the height is unbounded.
    pub fn is_height_unbounded(&self) -> bool {
        self.max_height == usize::MAX
    }

    /// Returns true if the `min_width` and `max_width` are the same.
    pub fn is_width_tight(&self) -> bool {
        self.max_width == self.min_width
    }

    /// Returns true if the `min_height` and `max_height` are the same.
    pub fn is_height_tight(&self) -> bool {
        self.max_height == self.min_height
    }

    /// Make the width constraint tight.
    /// ```
    /// # use anathema_widgets::layout::Constraints;
    /// let mut constraints = Constraints::new(10, 10);
    /// constraints.make_width_tight(constraints.max_width());
    /// # assert_eq!(constraints.min_width, constraints.max_width());
    /// ```
    pub fn make_width_tight(&mut self, width: usize) {
        self.max_width = self.max_width.min(width);
        self.min_width = self.max_width;
    }

    /// Make the height constraint tight.
    /// ```
    /// # use anathema_widgets::layout::Constraints;
    /// let mut constraints = Constraints::new(10, 10);
    /// constraints.make_height_tight(constraints.max_height());
    /// # assert_eq!(constraints.min_height, constraints.max_height());
    /// ```
    pub fn make_height_tight(&mut self, height: usize) {
        self.max_height = self.max_height.min(height);
        self.min_height = self.max_height;
    }

    pub fn expand_horz(&mut self, mut size: Size) -> Size {
        size.width = self.max_width;
        size
    }

    pub fn expand_vert(&mut self, mut size: Size) -> Size {
        size.height = self.max_height;
        size
    }

    pub fn expand_all(&mut self, mut size: Size) -> Size {
        size = self.expand_horz(size);
        self.expand_vert(size)
    }

    /// This function does not verify anything, but simply
    /// sets the max width.
    /// There is no check to see if the max width is smaller than the min width here.
    pub fn set_max_width(&mut self, width: usize) {
        self.max_width = width;
    }

    /// This function does not verify anything, but simply
    /// sets the max height.
    /// There is no check to see if the max height is smaller than the min height here.
    pub fn set_max_height(&mut self, height: usize) {
        self.max_height = height;
    }

    pub fn div_assign_max_width(&mut self, width: usize) {
        self.max_width /= width
    }

    pub fn div_assign_max_height(&mut self, height: usize) {
        self.max_height /= height
    }

    /// If either the max width or max height are
    /// zero then nothing can be laid out within
    /// the given constraint.
    pub fn has_zero_dimension(&self) -> bool {
        self.max_width == 0 || self.max_height == 0
    }

    /// Get a size from the max width / height
    pub fn max_size(&self) -> Size {
        (self.max_width, self.max_height).into()
    }
}

impl From<Size> for Constraints {
    fn from(value: Size) -> Self {
        Self::new(value.width, value.height)
    }
}

impl From<Rect> for Constraints {
    fn from(value: Rect) -> Self {
        let width = value.end.x - value.start.x;
        let height = value.end.y - value.start.y;
        Self::new(width as usize, height as usize)
    }
}

impl Default for Constraints {
    fn default() -> Self {
        Self::ZERO
    }
}
