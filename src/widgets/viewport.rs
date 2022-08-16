use std::any::Any;

use super::{HorzEdge, LayoutCtx, NodeId, PaintCtx, PositionCtx, UpdateCtx, VertEdge, WithSize};
use crate::display::Size;
use crate::widgets::layout::{horizontal, vertical};
use crate::widgets::{Offset, Widget, WidgetContainer};

/// A viewport where the children can be rendered with an offset.
#[derive(Debug)]
pub struct Viewport {
    /// Offset, either vertical or horizontal
    pub offset: Offset,
    /// Children
    pub children: Vec<WidgetContainer>,
    /// Clamp the vertical space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp_vertical: bool,
    /// Clamp the horizontal space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp_horizontal: bool,
}

impl Viewport {
    const KIND: &'static str = "Viewport";

    /// Create a new instance of a [`Viewport`]
    pub fn new(offset: Offset) -> Self {
        Self { offset, children: vec![], clamp_horizontal: true, clamp_vertical: true }
    }

    /// Swap the current edge to the opposite edge, updating the offset so the content isn't
    /// repositioned.
    ///
    /// This is useful to "freeze" scrolling behaviour.
    pub fn swap_edges(&mut self, size: Size) {
        match self.offset {
            Offset::Vertical(edge @ VertEdge::Top(offset) | edge @ VertEdge::Bottom(offset)) => {
                let total_child_height = self.children.iter().map(|c| c.size.height).sum::<usize>() as i32;
                let start = total_child_height - size.height as i32;
                let new_offset = start - offset;
                match edge {
                    VertEdge::Top(_) => self.offset = Offset::Vertical(VertEdge::Bottom(new_offset)),
                    VertEdge::Bottom(_) => self.offset = Offset::Vertical(VertEdge::Top(new_offset)),
                }
            }
            Offset::Horizontal(edge @ HorzEdge::Left(offset) | edge @ HorzEdge::Right(offset)) => {
                let total_child_width = self.children.iter().map(|c| c.size.width).sum::<usize>() as i32;
                let start = total_child_width - size.width as i32;
                let new_offset = start - offset;
                match edge {
                    HorzEdge::Left(_) => self.offset = Offset::Horizontal(HorzEdge::Right(new_offset)),
                    HorzEdge::Right(_) => self.offset = Offset::Horizontal(HorzEdge::Left(new_offset)),
                }
            }
        }
    }
}

impl Widget for Viewport {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }

    fn layout(&mut self, ctx: LayoutCtx) -> Size {
        let mut child_ctx = ctx;
        match self.offset {
            Offset::Vertical(_) => child_ctx.constraints.make_height_unbounded(),
            Offset::Horizontal(_) => child_ctx.constraints.make_width_unbounded(),
        }

        let size = match self.offset {
            Offset::Vertical(_) => vertical::layout(&mut self.children, child_ctx, true),
            Offset::Horizontal(_) => horizontal::layout(&mut self.children, child_ctx, true),
        };

        Size { width: size.width.min(ctx.constraints.max_width), height: size.height.min(ctx.constraints.max_height) }
    }

    fn position(&mut self, mut ctx: PositionCtx) {
        match self.offset {
            Offset::Vertical(VertEdge::Top(mut offset)) => {
                if self.clamp_vertical {
                    let total_child_height = self.children.iter().map(|c| c.size.height).sum::<usize>() as i32;
                    let start = total_child_height - ctx.size.height as i32;
                    offset = offset.clamp(0, start);
                }

                ctx.pos.y -= offset;
            }
            Offset::Vertical(VertEdge::Bottom(mut offset)) => {
                let total_child_height = self.children.iter().map(|c| c.size.height).sum::<usize>() as i32;
                let start = total_child_height - ctx.size.height as i32;

                if self.clamp_vertical {
                    offset = offset.clamp(0, start);
                }

                ctx.pos.y -= start - offset;
            }
            Offset::Horizontal(HorzEdge::Left(mut offset)) => {
                if self.clamp_horizontal {
                    let total_child_width = self.children.iter().map(|c| c.size.width).sum::<usize>() as i32;
                    let start = total_child_width - ctx.size.width as i32;
                    offset = offset.clamp(0, start);
                }

                ctx.pos.x -= offset;
            }
            Offset::Horizontal(HorzEdge::Right(mut offset)) => {
                let total_child_width = self.children.iter().map(|c| c.size.width).sum::<usize>() as i32;
                let start = total_child_width - ctx.size.width as i32;

                if self.clamp_horizontal {
                    offset = offset.clamp(0, start);
                }

                ctx.pos.x -= start - offset;
            }
        }

        match self.offset {
            Offset::Vertical(_) => vertical::position(&mut self.children, ctx),
            Offset::Horizontal(_) => horizontal::position(&mut self.children, ctx),
        }
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        let clip = ctx.create_region();
        for child in self.children.iter_mut() {
            let ctx = ctx.sub_context(Some(&clip));
            child.paint(ctx);
        }
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

    fn update(&mut self, ctx: UpdateCtx) {
        if let Some(offset) = ctx.attributes.offset() {
            self.offset = offset;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::testing::{test_widget, test_widget_container};
    use crate::widgets::{Border, BorderStyle, Sides, Text};

    fn viewport(child_range: std::ops::Range<usize>) -> Viewport {
        let offset = Offset::default();
        let mut viewport = Viewport::new(offset);

        for val in child_range {
            let child = Text::with_text(format!("{val}")).into_container(NodeId::auto());
            viewport.children.push(child);
        }

        viewport
    }

    fn test_viewport(viewport: Viewport, expected: &str) -> WidgetContainer {
        let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
        border.child = Some(viewport.into_container(NodeId::Value("viewport".into())));
        test_widget(border, expected)
    }

    #[test]
    fn change_offset_from_top() {
        let mut viewport = viewport(0..4);
        viewport.offset = Offset::Vertical(VertEdge::Top(1));

        test_viewport(
            viewport,
            r#"
            ┌───┐
            │1  │
            │2  │
            │3  │
            └───┘
            "#,
        );
    }

    #[test]
    fn change_offset_from_bottom() {
        let mut viewport = viewport(0..5);
        viewport.offset = Offset::Vertical(VertEdge::Bottom(1));

        test_viewport(
            viewport,
            r#"
            ┌───┐
            │1  │
            │2  │
            │3  │
            └───┘
            "#,
        );
    }

    #[test]
    fn edge_swap() {
        let mut viewport = viewport(0..10);
        viewport.offset = Offset::Vertical(VertEdge::Bottom(0));

        let mut root = test_viewport(
            viewport,
            r#"
            ┌───┐
            │7  │
            │8  │
            │9  │
            └───┘
            "#,
        );

        for _ in 0..3 {
            let viewport = root.by_id("viewport").unwrap();
            let size = viewport.size;
            viewport.to::<Viewport>().swap_edges(size);

            root = test_widget_container(
                root,
                r#"
            ┌───┐
            │7  │
            │8  │
            │9  │
            └───┘
            "#,
            );
        }
    }

    #[test]
    fn clamp_offset_negative() {
        let mut viewport = viewport(0..4);
        viewport.clamp_vertical = true;
        viewport.offset = Offset::Vertical(VertEdge::Top(-100));

        test_viewport(
            viewport,
            r#"
            ┌───┐
            │0  │
            │1  │
            │2  │
            └───┘
            "#,
        );
    }

    #[test]
    fn clamp_offset_positive() {
        let mut viewport = viewport(0..4);
        viewport.clamp_vertical = true;
        viewport.offset = Offset::Vertical(VertEdge::Top(100));

        test_viewport(
            viewport,
            r#"
            ┌───┐
            │1  │
            │2  │
            │3  │
            └───┘
            "#,
        );
    }
}
