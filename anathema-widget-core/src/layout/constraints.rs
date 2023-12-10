use anathema_render::Size;

use crate::Padding;

/// `Constraints` are used to ensure that a widget doesn't size it self outside of a set of given bounds.
/// A constraint can be tight, meaning then minimum and maximum width / height are the same.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Constraints {
    /// Minimum width.
    pub min_width: usize,
    /// Minimum height.
    pub min_height: usize,
    /// Max width.
    pub max_width: usize,
    /// Max height.
    pub max_height: usize,
}

impl Constraints {
    pub const ZERO: Self = Self {
        min_width: 0,
        min_height: 0,
        max_width: 0,
        max_height: 0,
    };

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
    /// # use anathema_widget_core::layout::Constraints;
    /// let mut constraints = Constraints::new(10, 10);
    /// constraints.make_width_tight(constraints.max_width);
    /// # assert_eq!(constraints.min_width, constraints.max_width);
    /// ```
    pub fn make_width_tight(&mut self, width: usize) {
        self.max_width = self.max_width.min(width);
        self.min_width = self.max_width;
    }

    /// Make the height constraint tight.
    /// ```
    /// # use anathema_widget_core::layout::Constraints;
    /// let mut constraints = Constraints::new(10, 10);
    /// constraints.make_height_tight(constraints.max_height);
    /// # assert_eq!(constraints.min_height, constraints.max_height);
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

    pub fn apply_padding(&mut self, padding: Padding) {
        if padding == Padding::ZERO {
            return;
        }

        if !self.is_width_unbounded() {
            self.max_width = self.max_width.saturating_sub(padding.left + padding.right);
            self.min_width = self.min_width.min(self.max_width);
        }

        if !self.is_height_unbounded() {
            self.max_height = self.max_height.saturating_sub(padding.top + padding.bottom);
            self.min_height = self.min_height.min(self.max_height);
        }
    }
}

#[cfg(test)]
mod test {}
