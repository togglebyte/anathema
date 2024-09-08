use std::ops::ControlFlow;

use anathema_geometry::Size;
use anathema_widgets::layout::{Constraints, LayoutCtx};
use anathema_widgets::LayoutChildren;

use super::{expand, spacers, Axis, Direction};

pub(crate) struct SizeMod {
    inner: Size,
    max_size: Size,
    axis: Axis,
}

impl SizeMod {
    const ZERO: Self = Self::new(Size::ZERO, Axis::Vertical);

    const fn new(max_size: Size, axis: Axis) -> Self {
        Self {
            inner: Size::ZERO,
            max_size,
            axis,
        }
    }

    fn apply(&mut self, size: Size) {
        match self.axis {
            Axis::Vertical => {
                self.inner.width = self.inner.width.max(size.width);
                self.inner.height = (self.inner.height + size.height).min(self.max_size.height);
            }
            Axis::Horizontal => {
                self.inner.height = self.inner.height.max(size.height);
                self.inner.width = (self.inner.width + size.width).min(self.max_size.width);
            }
        }
    }

    fn no_space_left(&self) -> bool {
        match self.axis {
            Axis::Horizontal => self.inner.width >= self.max_size.width,
            Axis::Vertical => self.inner.height >= self.max_size.height,
        }
    }

    fn to_constraints(&self) -> Constraints {
        match self.axis {
            Axis::Horizontal => Constraints::new(self.max_size.width - self.inner.width, self.max_size.height),
            Axis::Vertical => Constraints::new(self.max_size.width, self.max_size.height - self.inner.height),
        }
    }

    pub fn inner_size(&self) -> Size {
        self.inner
    }
}

pub struct Many {
    pub direction: Direction,
    pub axis: Axis,
    unconstrained: bool,
    pub(crate) used_size: SizeMod,
}

impl Many {
    pub fn new(direction: Direction, axis: Axis, unconstrained: bool) -> Self {
        Self {
            direction,
            axis,
            unconstrained,
            used_size: SizeMod::ZERO,
        }
    }
}

impl Many {
    pub(crate) fn layout<'bp>(
        &mut self,
        mut children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let max_constraints = constraints;

        self.used_size.axis = self.axis;
        self.used_size.max_size = Size::new(max_constraints.max_width(), max_constraints.max_height());

        let mut size = Size::ZERO;

        children.for_each(|node, children| {
            if ["spacer", "expand"].contains(&node.ident) {
                return ControlFlow::Continue(());
            }

            let widget_constraints = {
                let mut constraints = self.used_size.to_constraints();
                if self.unconstrained {
                    match self.axis {
                        Axis::Vertical => constraints.unbound_height(),
                        Axis::Horizontal => constraints.unbound_width(),
                    }
                }
                constraints
            };

            let widget_size = node.layout(children, widget_constraints, ctx);

            self.used_size.apply(widget_size);

            match self.used_size.no_space_left() {
                true => ControlFlow::Break(()),
                false => ControlFlow::Continue(()),
            }
        });

        // Apply spacer and expand if the layout is constrained and we have remaining space
        if !self.unconstrained && !self.used_size.no_space_left() {
            let constraints = self.used_size.to_constraints();
            let expanded_size = expand::layout_all_expansions(&mut children, constraints, self.axis, ctx);
            self.used_size.apply(expanded_size);

            let constraints = self.used_size.to_constraints();
            let spacer_size = spacers::layout_all_spacers(&mut children, constraints, self.axis, ctx);
            self.used_size.apply(spacer_size);
        }

        size.width = self.used_size.inner.width.max(max_constraints.min_width);
        size.height = (self.used_size.inner.height).max(max_constraints.min_height);

        match self.axis {
            Axis::Vertical => {
                size.width = self.used_size.inner.width.max(max_constraints.min_width);
                size.height = (self.used_size.inner.height).max(max_constraints.min_height);
            }
            Axis::Horizontal => {
                size.height = size.height.max(self.used_size.inner.height);
                size.height = size
                    .height
                    .max(self.used_size.inner.height)
                    .max(max_constraints.min_height);
                size.width = size
                    .width
                    .max(self.used_size.inner.width)
                    .max(max_constraints.min_width);
            }
        }

        if let Direction::Backward = self.direction {
            match self.axis {
                Axis::Horizontal => size.width = max_constraints.max_width(),
                Axis::Vertical => size.height = max_constraints.max_height(),
            }
        }

        size
    }
}
