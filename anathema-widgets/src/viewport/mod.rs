use anathema_render::Size;

use self::layout::ViewportLayout;
use self::position::Position;
use super::{PaintCtx, PositionCtx, WithSize};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::gen::generator::Generator;
use crate::layout::Layouts;
use crate::layout::horizontal::Horizontal;
use crate::layout::many::Many;
use crate::layout::vertical::Vertical;
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;
use crate::{AnyWidget, Direction, TextPath, Value, Widget, WidgetContainer, Axis};

mod layout;
mod position;

/// A viewport where the children can be rendered with an offset.
#[derive(Debug)]
pub struct Viewport {
    /// Line / cell offset
    pub offset: usize,
    /// Clamp the vertical space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp_vertical: bool,
    /// Clamp the horizontal space, meaning the edge of the content can not surpass the edge of the
    /// visible space.
    pub clamp_horizontal: bool,
    /// Layout direction.
    /// `Direction::Forward` is the default, and keeps the scroll position on the first child.
    /// `Direction::Backward` keeps the scroll position on the last child.
    pub direction: Direction,
    /// Vertical or horizontal
    pub axis: Axis,
}

impl Viewport {
    const KIND: &'static str = "Viewport";

    /// Create a new instance of a [`Viewport`]
    pub fn new(direction: Direction, axis: Axis, offset: Option<usize>) -> Self {
        Self {
            offset: offset.unwrap_or(0),
            clamp_horizontal: true,
            clamp_vertical: true,
            direction,
            axis,
        }
    }
}

impl Widget for Viewport {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'tpl, 'parent>(&mut self, mut ctx: LayoutCtx<'_, 'tpl, 'parent>) -> Result<Size> {
        Layouts::new(Many::new(self.direction, self.axis, self.offset), &mut ctx).layout()?.size()
    }

    fn position<'gen, 'ctx>(&mut self, ctx: PositionCtx, children: &mut [WidgetContainer<'gen>]) {
        let mut pos = ctx.padded_position();
        let offset = self.offset;
        for widget in children {
            widget.position(pos);
            match self.axis {
                Axis::Horizontal => pos.x += widget.size.width as i32,
                Axis::Vertical => pos.y += widget.size.height as i32,
            }
        }
    }

    fn paint<'gen, 'ctx>(
        &mut self,
        mut ctx: PaintCtx<'_, WithSize>,
        children: &mut [WidgetContainer<'gen>],
    ) {
        for child in children {
            let ctx = ctx.sub_context(None);
            // TODO: do we even need the clipping region here?
            // let ctx = ctx.sub_context(Some(&clip));
            child.paint(ctx);
        }
    }
}

pub(crate) struct ViewportFactory;

impl WidgetFactory for ViewportFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        text: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let data_source = values.get_attrib("source").map(|v| v.to_owned());
        let binding = values.get_attrib("binding").map(|v| v.to_string());
        let item = values.get_int("item").unwrap_or(0) as usize;
        let direction = values.direction().unwrap_or(Direction::Forward);
        let axis = values.axis().unwrap_or(Axis::Vertical);
        let offset = values.offset();
        let widget = Viewport::new(direction, axis, offset);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::testing::{test_widget, test_widget_container};
    // use crate::{Border, BorderStyle, Sides, Text};

    // fn viewport(child_range: std::ops::Range<usize>) -> Viewport {
    //     let offset = Offset::default();
    //     let mut viewport = Viewport::new(offset);

    //     for val in child_range {
    //         let child = Text::with_text(format!("{val}")).into_container(NodeId::anon());
    //         viewport.children.push(child);
    //     }

    //     viewport
    // }

    // fn test_viewport(viewport: Viewport, expected: &str) -> WidgetContainer {
    //     let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, None, None);
    //     border.child = Some(viewport.into_container(NodeId::Value("viewport".into())));
    //     test_widget(border, expected)
    // }

    // #[test]
    // fn change_offset_from_top() {
    //     let mut viewport = viewport(0..4);
    //     viewport.offset = Offset::Vertical(VertEdge::Top(1));

    //     test_viewport(
    //         viewport,
    //         r#"
    //         ┌───┐
    //         │1  │
    //         │2  │
    //         │3  │
    //         └───┘
    //         "#,
    //     );
    // }

    // #[test]
    // fn change_offset_from_bottom() {
    //     let mut viewport = viewport(0..5);
    //     viewport.offset = Offset::Vertical(VertEdge::Bottom(1));

    //     test_viewport(
    //         viewport,
    //         r#"
    //         ┌───┐
    //         │1  │
    //         │2  │
    //         │3  │
    //         └───┘
    //         "#,
    //     );
    // }

    // #[test]
    // fn edge_swap() {
    //     let mut viewport = viewport(0..10);
    //     viewport.offset = Offset::Vertical(VertEdge::Bottom(0));

    //     let mut root = test_viewport(
    //         viewport,
    //         r#"
    //         ┌───┐
    //         │7  │
    //         │8  │
    //         │9  │
    //         └───┘
    //         "#,
    //     );

    //     for _ in 0..3 {
    //         let viewport = root.by_id("viewport").unwrap();
    //         let size = viewport.size;
    //         viewport.to_mut::<Viewport>().swap_edges(size);

    //         root = test_widget_container(
    //             root,
    //             r#"
    //         ┌───┐
    //         │7  │
    //         │8  │
    //         │9  │
    //         └───┘
    //         "#,
    //         );
    //     }
    // }

    // #[test]
    // fn clamp_offset_negative() {
    //     let mut viewport = viewport(0..4);
    //     viewport.clamp_vertical = true;
    //     viewport.offset = Offset::Vertical(VertEdge::Top(-100));

    //     test_viewport(
    //         viewport,
    //         r#"
    //         ┌───┐
    //         │0  │
    //         │1  │
    //         │2  │
    //         └───┘
    //         "#,
    //     );
    // }

    // #[test]
    // fn clamp_offset_positive() {
    //     let mut viewport = viewport(0..4);
    //     viewport.clamp_vertical = true;
    //     viewport.offset = Offset::Vertical(VertEdge::Top(100));

    //     test_viewport(
    //         viewport,
    //         r#"
    //         ┌───┐
    //         │1  │
    //         │2  │
    //         │3  │
    //         └───┘
    //         "#,
    //     );
    // }
}
