use anathema_render::Size;

use super::{HorzEdge, Pos, PositionCtx, VertEdge, Widget, WidgetContainer};
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::layout::single::Single;
use crate::layout::Layouts;
use crate::lookup::WidgetFactory;
use crate::values::ValuesAttributes;
use crate::{AnyWidget, TextPath};

/// If the horizontal edge is set to `Right` the widget will expand to fill all available space
/// on the horizontal axis.
/// Same is true if the `VertEdge::Bottom` is set.
///
/// Position on the horizontal axis:
/// Left 0 would mean the left edge of the widget is positioned at the `left` value.
/// Right 0 would mean the right edge of the widget is positioned at the `right` value, where zero
/// is closest to the right.
/// ```text
/// ----- total width -----
/// ┌─┐                 ┌─┐
/// │1│                 │2│
/// └─┘                 └─┘
///
/// Position on the vertical axis:
/// Top 0 would mean the top edge of the child is positioned at `top` value.
/// Bottom 0 would mean the bottom edge of the child is positioned at the `bottom` value, where
/// zero is closest to the bottom.
/// ```text
/// | ┌────────┐
/// | │1       │
/// | └────────┘
/// |
/// |
/// |
/// |
/// |
/// | ┌────────┐
/// | │2       │
/// | └────────┘
/// ```
/// ```
#[derive(Debug)]
pub struct Position {
    /// Horizontal edge
    pub horz_edge: HorzEdge,
    /// Vertical edge
    pub vert_edge: VertEdge,
}

impl Position {
    /// Widget name
    pub const KIND: &'static str = "Position";

    /// Create a new instance of a `Position`
    pub fn new(horz_edge: HorzEdge, vert_edge: VertEdge) -> Self {
        Self {
            horz_edge,
            vert_edge,
        }
    }

    /// Position to the left
    pub fn left(&mut self, offset: i32) {
        self.horz_edge = HorzEdge::Left(offset);
    }

    /// Position to the right
    pub fn right(&mut self, offset: i32) {
        self.horz_edge = HorzEdge::Right(offset);
    }

    /// Position at the top
    pub fn top(&mut self, offset: i32) {
        self.vert_edge = VertEdge::Top(offset);
    }

    /// Position at the bottom
    pub fn bottom(&mut self, offset: i32) {
        self.vert_edge = VertEdge::Bottom(offset);
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(HorzEdge::Left(0), VertEdge::Top(0))
    }
}

impl Widget for Position {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout(&mut self, mut ctx: LayoutCtx<'_, '_, '_>) -> Result<Size> {
        let mut layout = Layouts::new(Single, &mut ctx);
        layout.layout()?;
        if let HorzEdge::Right(_) = self.horz_edge {
            layout.expand_horz();
        }
        if let VertEdge::Bottom(_) = self.vert_edge {
            layout.expand_vert();
        }
        layout.size()
    }

    fn position<'gen, 'ctx>(
        &mut self,
        mut ctx: PositionCtx,
        children: &mut [WidgetContainer<'gen>],
    ) {
        let child = match children.first_mut() {
            Some(c) => c,
            None => return,
        };

        let x = match self.horz_edge {
            HorzEdge::Left(x) => x,
            HorzEdge::Right(x) => ctx.inner_size.width as i32 - x - child.outer_size().width as i32,
        };

        let y = match self.vert_edge {
            VertEdge::Top(y) => y,
            VertEdge::Bottom(y) => {
                ctx.inner_size.height as i32 - y - child.outer_size().height as i32
            }
        };

        ctx.pos += Pos::new(x, y);
        child.position(ctx.pos);
    }

    //     // fn update(&mut self, ctx: UpdateCtx) {
    //     //     for (k, _) in &ctx.attributes {
    //     //         match k.as_str() {
    //     //             fields::LEFT => self.horz_edge = HorzEdge::Left(ctx.attributes.left().unwrap_or(0)),
    //     //             fields::RIGHT => {
    //     //                 self.horz_edge = HorzEdge::Right(ctx.attributes.right().unwrap_or(0))
    //     //             }
    //     //             fields::TOP => self.vert_edge = VertEdge::Top(ctx.attributes.top().unwrap_or(0)),
    //     //             fields::BOTTOM => {
    //     //                 self.vert_edge = VertEdge::Bottom(ctx.attributes.bottom().unwrap_or(0))
    //     //             }
    //     //             _ => {}
    //     //         }
    //     //     }
    //     // }
}

pub(crate) struct PositionFactory;

impl WidgetFactory for PositionFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        _: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let horz_edge = match values.left() {
            Some(left) => HorzEdge::Left(left),
            None => match values.right() {
                Some(right) => HorzEdge::Right(right),
                None => HorzEdge::Left(0),
            },
        };

        let vert_edge = match values.top() {
            Some(top) => VertEdge::Top(top),
            None => match values.bottom() {
                Some(bottom) => VertEdge::Bottom(bottom),
                None => VertEdge::Top(0),
            },
        };
        let widget = Position::new(horz_edge, vert_edge);
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::template::template_text;
    use crate::testing::{test_widget, FakeTerm};

    #[test]
    fn top_left() {
        let body = [template_text("top left")];

        test_widget(
            Position::new(HorzEdge::Left(0), VertEdge::Top(0)),
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║top left       ║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn top_right() {
        let body = [template_text("top right")];
        test_widget(
            Position::new(HorzEdge::Right(0), VertEdge::Top(0)),
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║      top right║
            ║               ║
            ║               ║
            ║               ║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn bottom_right() {
        let body = [template_text("bottom right")];
        test_widget(
            Position::new(HorzEdge::Right(0), VertEdge::Bottom(0)),
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║               ║
            ║               ║
            ║               ║
            ║   bottom right║
            ╚═══════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn bottom_left() {
        let body = [template_text("bottom left")];
        test_widget(
            Position::new(HorzEdge::Left(0), VertEdge::Bottom(0)),
            &body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═╗
            ║               ║
            ║               ║
            ║               ║
            ║bottom left    ║
            ╚═══════════════╝
            "#,
            ),
        );
    }
}
