use display::Size;

use crate::attributes::Attributes;
use crate::Pos;

use super::position::NAME as POSITION_NAME;
use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};

/// The stack sizes itself to contain all the non-positioned children,
/// which are positioned according to alignment
/// (which defaults to:
/// the top-left corner in left-to-right environments
/// and the top-right corner in right-to-left environments).
#[derive(Debug)]
pub struct Stack {
    pub children: Vec<WidgetContainer>,
}

impl Stack {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
}

impl Widget for Stack {
    fn kind(&self) -> &'static str {
        "Stack"
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        let mut child_size = Size::zero();

        for child in self.children.iter_mut() {
            let size = child.layout(ctx.constraints, ctx.force_layout);
            child_size.width = child.size.width.max(size.width);
            child_size.height = child.size.height.max(size.height);
        }

        child_size
    }

    fn position(&mut self, ctx: PositionCtx) {
        for child in self.children.iter_mut() {
            child.position(ctx.pos);
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        // Draw children that are NOT position first
        self.children
            .iter_mut()
            .filter(|c| c.kind() != POSITION_NAME)
            .for_each(|c| c.paint(ctx.to_unsized()));

        // Draw all position children
        self.children
            .iter_mut()
            .filter(|c| c.kind() == POSITION_NAME)
            .for_each(|c| c.paint(ctx.to_unsized()));
    }

    fn children(&mut self) -> Vec<&mut WidgetContainer> {
        self.children.iter_mut().collect()
    }

    fn add_child(&mut self, widget: WidgetContainer) {
        self.children.push(widget);
    }

    fn remove_child(&mut self, child_id: &NodeId) -> Option<WidgetContainer> {
        if let Some(pos) = self.children.iter().position(|c| c.id.eq(child_id)) {
            return Some(self.children.remove(pos));
        }
        None
    }

    fn update(&mut self, _: Attributes) {}
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::test_widget;
    use crate::{Border, BorderStyle, HorzEdge, Position, Sides, Text, VertEdge};

    fn test_stack(h_edge: HorzEdge, v_edge: VertEdge, expected: &str) {
        let mut stack = Stack::new();
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        let mut position = Position::new(h_edge, v_edge);
        position.add_child(Text::with_text(fields::POSITION_X).into_container(NodeId::auto()));
        stack.add_child(border.into_container(NodeId::auto()));
        stack.add_child(position.into_container(NodeId::auto()));
        test_widget(stack, expected);
    }

    #[test]
    fn top_left() {
        test_stack(
            HorzEdge::Left(0),
            VertEdge::Top(0),
            r#"
            x───┐
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn top_right() {
        test_stack(
            HorzEdge::Right(0),
            VertEdge::Top(0),
            r#"
            ┌───x
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn bottom_right() {
        test_stack(
            HorzEdge::Right(0),
            VertEdge::Bottom(0),
            r#"
            ┌───┐
            │   │
            │   │
            └───x
            "#,
        );
    }

    #[test]
    fn bottom_left() {
        test_stack(
            HorzEdge::Left(0),
            VertEdge::Bottom(0),
            r#"
            ┌───┐
            │   │
            │   │
            x───┘
            "#,
        );
    }

    #[test]
    fn centre_ish() {
        test_stack(
            HorzEdge::Left(2),
            VertEdge::Top(1),
            r#"
            ┌───┐
            │ x │
            │   │
            └───┘
            "#,
        );
    }
}
