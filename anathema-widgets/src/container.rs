use crate::attributes::{fields, Attributes};
use crate::{LocalPos, Pos};
use display::{Color, Size, Style};

use super::{LayoutCtx, NodeId, PaintCtx, Widget, WidgetContainer, WithSize};

/// If the widget has no child, no height, no width, no constraints,
/// and the parent provides unbounded constraints, then the Container
/// tries to size as small as possible.
///
/// If the widget has no child and no alignment, but a height, width, or constraints are provided,
/// then the Container tries to be as small as possible given the
/// combination of those constraints and the parent's constraints.
///
/// If the widget has no child, no height, no width, no constraints, and no alignment,
/// but the parent provides bounded constraints, then Container expands to fit the constraints provided by the parent.
#[derive(Debug)]
pub struct Container {
    pub background: Option<Color>,
    pub child: Option<WidgetContainer>,
    width: Option<usize>,
    height: Option<usize>,
}

impl Container {
    pub fn new(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self { child: None, width: width.into(), height: height.into(), background: None }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl Widget for Container {
    fn kind(&self) -> &'static str {
        "Container"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, mut ctx: LayoutCtx) -> Size {
        // -----------------------------------------------------------------------------
        //     - Layout child -
        //     In the following order:
        //     1. Alignment
        //     2. Attribute size (width, height)
        //     3. Child size
        //     4. Constraints
        // -----------------------------------------------------------------------------

        // Adjust the constraints to the width an height
        if let Some(ref mut width) = self.width {
            if *width < ctx.constraints.min_width {
                *width = ctx.constraints.min_width;
            }

            let max_width = ctx.constraints.max_width.unwrap_or(0);
            if *width < max_width {
                ctx.constraints.max_width = Some(*width);
            }
        }

        if let Some(ref mut height) = self.height {
            if *height < ctx.constraints.min_height {
                *height = ctx.constraints.min_height;
            }

            let max_height = ctx.constraints.max_height.unwrap_or(0);
            if *height < max_height {
                ctx.constraints.max_height = Some(*height);
            }
        }

        let size = match self.child.as_mut() {
            Some(child) => child.layout(ctx),
            None => Size::new(
                ctx.constraints.max_width.unwrap_or(ctx.constraints.min_width),
                ctx.constraints.max_height.unwrap_or(ctx.constraints.min_height),
            ),
        };

        super::adapt_size(size, self.width, self.height, ctx)
    }

    fn position(&mut self, pos: Pos, _: Size) {
        if let Some(child) = self.child.as_mut() {
            child.position(pos);
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        let width = ctx.local_size.width;
        let height = ctx.local_size.height;

        // Draw background
        if let Some(background) = self.background {
            let background_str = format!("{:width$}", "", width = width);
            let mut style = Style::new();
            style.set_bg(background);

            for y in 0..height {
                let pos = LocalPos::new(0, y);
                ctx.print(&background_str, style, pos);
            }
        }

        if let Some(child) = self.child.as_mut() {
            child.paint(ctx.to_unsized());
        }
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        match self.child.as_mut() {
            Some(c) => vec![c],
            None => vec![],
        }
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.child = Some(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(ref child) = self.child {
            if child.id.eq(child_id) {
                return self.child.take();
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::layout::Constraints;
    use crate::testing::test_widget;
    use crate::{Border, BorderStyle, Sides};

    fn test_container(expected: &str) {
        let mut container = Container::new(None, None);
        container.child = Some(Border::new(&BorderStyle::Thin, Sides::ALL, None, None).into_container(NodeId::anon()));
        test_widget(container, expected);
    }

    #[test]
    fn container() {
        test_container(
            r#"
            ┌──────┐
            │      │
            │      │
            │      │
            └──────┘
            "#,
        );
    }

    #[test]
    fn layout_with_parent_constraints_no_child_no_size() {
        // If the widget has no child and no alignment, but a height, width,
        // or constraints are provided, then the Container tries to be as
        // small as possible given the combination of those constraints and
        // the parent's constraints.
        let mut container = Container::new(None, None);
        let constraints = Constraints::new(10, 10);
        let actual = container.layout(LayoutCtx::new(constraints, false));
        let expected = Size::new(10, 10);
        assert_eq!(expected, actual);
    }

    #[test]
    fn with_width() {
        let mut container = Container::new(Some(10), None);
        let constraints = Constraints::new(100, 100);
        let actual = container.layout(LayoutCtx::new(constraints, false));
        let expected = Size::new(10, 100);
        assert_eq!(expected, actual);
    }

    #[test]
    fn with_height() {
        let mut container = Container::new(None, Some(10));
        let constraints = Constraints::new(100, 100);
        let actual = container.layout(LayoutCtx::new(constraints, false));
        let expected = Size::new(100, 10);
        assert_eq!(expected, actual);
    }

    #[test]
    fn with_width_and_child() {
        // To be able to layout without constraint the root
        // is ignored in the layout process, and layout is called
        // directly on the parent
        let mut parent = Container::new(10, 2);
        parent.child = Some(Container::new(2, 2).into_container(NodeId::anon()));
        let ctx = LayoutCtx::new(Constraints::unbounded(), false);
        let size = parent.layout(ctx);
        assert_eq!(size, Size::new(10, 2));
    }

    #[test]
    fn unsized_unconstrained() {
        // If the widget has no child, no height, no width, no constraints,
        // and the parent provides unbounded constraints,
        // then Container tries to size as small as possible.
        let mut container = Container::new(None, None);
        let constraints = Constraints::unbounded();
        let actual = container.layout(LayoutCtx::new(constraints, false));
        let expected = Size::zero();
        assert_eq!(expected, actual);
    }

    #[test]
    fn constrained_only() {
        // If the widget has no child, no height, no width, no constraints,
        // and no alignment, but the parent provides bounded constraints,
        // then Container expands to fit the constraints provided by the parent.
        let mut container = Container::new(None, None);
        let constraints = Constraints::new(10, 5);
        let actual = container.layout(LayoutCtx::new(constraints, false));
        let expected = Size::new(10, 5);
        assert_eq!(expected, actual);
    }

    #[test]
    fn sized_by_child() {
        // The widget has a child but no height, no width and no constraints.
        // The container sizes itself after the child.
        let mut parent = Container::new(None, None);
        parent.child = Some(Container::new(2, 2).into_container(NodeId::anon()));
        let ctx = LayoutCtx::new(Constraints::unbounded(), false);
        let size = parent.layout(ctx);
        assert_eq!(size, Size::new(2, 2));
    }

    #[test]
    fn no_size_no_children_no_constraint() {
        // If the container has nothing that affects the size it should
        // be as small as possible

        let mut container = Container::new(None, None);
        let constraint = Constraints::unbounded();
        let actual = container.layout(LayoutCtx::new(constraint, false));
        let expected = Size::zero();
        assert_eq!(expected, actual);
    }
}
