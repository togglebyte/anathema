use crate::display::Size;

/// `Constraints` are used to ensure that a widget doesn't size it self outside of a set of given bounds.
/// A constraint can be tight, meaning then minimum and maximum width / height are the same.
#[derive(Debug, Copy, Clone, PartialEq)]
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
        Self { min_width: 0, min_height: 0, max_width, max_height }
    }

    /// Create unbounded constraints.
    pub fn unbounded() -> Self {
        Self { min_width: 0, min_height: 0, max_width: usize::MAX, max_height: usize::MAX }
    }

    /// Returns true if both width and height are unbounded.
    pub fn is_unbounded(&self) -> bool {
        self.max_width == usize::MAX && self.max_height == usize::MAX && self.min_width == 0 && self.min_height == 0
    }

    /// Create new tight constraints.
    pub fn tight(width: usize, height: usize) -> Self {
        Self { min_width: width, min_height: height, max_width: width, max_height: height }
    }

    /// Returns true if the `min_width` and `max_width` are the same.
    pub fn is_width_tight(&self) -> bool {
        self.max_width == self.min_width
    }

    /// Returns true if the `min_height` and `max_height` are the same.
    pub fn is_height_tight(&self) -> bool {
        self.max_height == self.min_height
    }

    /// Constrain the size to fit inside the constraints.
    pub fn constrain_size(&self, size: &mut Size) {
        size.width = size.width.max(self.min_width);
        size.height = size.height.max(self.min_height);
        size.width = size.width.min(self.max_width);
        size.height = size.height.min(self.max_height);
    }

    /// Change the constraints to fit within the `other` constraints.
    pub fn fit_constraints(&mut self, other: &Constraints) {
        self.min_width = self.min_width.max(other.min_width);
        self.min_height = self.min_height.max(other.min_height);

        self.max_width = self.max_width.min(other.max_width);
        self.max_height = self.max_height.min(other.max_height);

        if self.min_width > self.max_width {
            self.min_width = self.max_width;
        }

        if self.min_height > self.max_height {
            self.min_height = self.max_height;
        }
    }

    /// Make the width constraint tight.
    /// ```
    /// # use anathema::widgets::Constraints;
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
    /// # use anathema::widgets::Constraints;
    /// let mut constraints = Constraints::new(10, 10);
    /// constraints.make_height_tight(constraints.max_height);
    /// # assert_eq!(constraints.min_height, constraints.max_height);
    /// ```
    pub fn make_height_tight(&mut self, height: usize) {
        self.max_height = self.max_height.min(height);
        self.min_height = self.max_height;
    }

    /// Fit the width inside the constraint.
    pub fn constrain_width(&self, width: &mut usize) {
        if *width > self.max_width {
            *width = self.max_width;
        }

        if self.min_width > *width {
            *width = self.min_width;
        }
    }

    /// Fit the height inside the constraint.
    pub fn constrain_height(&self, height: &mut usize) {
        if *height > self.max_height {
            *height = self.max_height;
        }

        if self.min_height > *height {
            *height = self.min_height;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::display::Size;

    #[test]
    fn shrink_constrained_size() {
        let mut size = Size::new(10, 10);
        let expected = Size::new(9, 2);
        let constraints = Constraints::new(9, 2);

        constraints.constrain_size(&mut size);
        assert_eq!(expected, size);
    }

    #[test]
    fn grow_constrained_size() {
        let mut size = Size::new(1, 1);
        let expected = Size::new(10, 10);
        let constraints = Constraints::tight(10, 10);

        constraints.constrain_size(&mut size);
        assert_eq!(expected, size);
    }

    #[test]
    fn merge_constraints() {
        let mut constraint_a = Constraints::new(10, 10);
        let constraint_b = Constraints::new(15, 9);
        constraint_a.fit_constraints(&constraint_b);
        let expected = Constraints { min_width: 0, min_height: 0, max_width: 10, max_height: 9 };
        assert_eq!(expected, constraint_a);
    }

    #[test]
    fn merge_constraints_with_min_values() {
        let mut constraint_a = Constraints::new(10, 10);
        constraint_a.min_width = 2;
        let mut constraint_b = Constraints::new(15, 9);
        constraint_b.min_height = 5;
        constraint_a.fit_constraints(&constraint_b);
        let expected = Constraints { min_width: 2, min_height: 5, max_width: 10, max_height: 9 };
        assert_eq!(expected, constraint_a);
    }

    #[test]
    fn merge_constraints_with_lots_of_issues() {
        let mut constraint_a = Constraints::tight(10, 10);
        let mut constraint_b = Constraints::new(15, 9);
        constraint_b.min_width = 12;

        constraint_a.fit_constraints(&constraint_b);
        let expected = Constraints { min_width: 10, min_height: 9, max_width: 10, max_height: 9 };
        assert_eq!(expected, constraint_a);
    }

    #[test]
    fn constraint_fit_size() {
        // Tight constraints
        let mut size = Size::new(1, 1);
        let constraints = Constraints::tight(10, 11);
        constraints.constrain_width(&mut size.width);
        constraints.constrain_height(&mut size.height);
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 11);

        // Min / max fit
        let mut constraints = Constraints::new(10, 10);
        constraints.min_width = 5;
        constraints.min_height = 5;
        let mut size = Size::new(12, 12);
        constraints.constrain_width(&mut size.width);
        constraints.constrain_height(&mut size.height);
        assert_eq!(size.width, 10);
        assert_eq!(size.height, 10);
        let mut size = Size::ZERO;
        constraints.constrain_width(&mut size.width);
        constraints.constrain_height(&mut size.height);
        assert_eq!(size.width, 5);
        assert_eq!(size.height, 5);

        // Unbounded constraints
        let constraints = Constraints::unbounded();
        let mut size = Size::new(123, 456);
        constraints.constrain_width(&mut size.width);
        constraints.constrain_height(&mut size.height);
        assert_eq!(size.width, 123);
        assert_eq!(size.height, 456);
    }
}
