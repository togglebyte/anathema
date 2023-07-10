use anathema_render::{Size, Style};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Layouts;
use anathema_widget_core::{
    fields, AnyWidget, LocalPos, TextPath, Value, ValuesAttributes, Widget, WidgetContainer,
    WidgetFactory,
};
use unicode_width::UnicodeWidthChar;

use crate::layout::border::BorderLayout;

// -----------------------------------------------------------------------------
//     - Indices -
//     Index into `DEFAULT_SLIM_EDGES` or `DEFAULT_THICK_EDGES`
// -----------------------------------------------------------------------------
pub const BORDER_EDGE_TOP_LEFT: usize = 0;
pub const BORDER_EDGE_TOP: usize = 1;
pub const BORDER_EDGE_TOP_RIGHT: usize = 2;
pub const BORDER_EDGE_RIGHT: usize = 3;
pub const BORDER_EDGE_BOTTOM_RIGHT: usize = 4;
pub const BORDER_EDGE_BOTTOM: usize = 5;
pub const BORDER_EDGE_BOTTOM_LEFT: usize = 6;
pub const BORDER_EDGE_LEFT: usize = 7;

// -----------------------------------------------------------------------------
//     - Sides -
// -----------------------------------------------------------------------------
bitflags::bitflags! {
    /// Border sides
    /// ```
    /// use anathema_widgets::Sides;
    /// let sides = Sides::TOP | Sides::LEFT;
    /// ```
    pub struct Sides: u8 {
        /// Empty
        const EMPTY = 0x0;
        /// Top border
        const TOP = 0b0001;
        /// Right border
        const RIGHT = 0b0010;
        /// Bottom border
        const BOTTOM = 0b0100;
        /// Left border
        const LEFT = 0b1000;
        /// All sides
        const ALL = Self::TOP.bits | Self::RIGHT.bits | Self::BOTTOM.bits | Self::LEFT.bits;
    }
}

impl From<&str> for Sides {
    fn from(s: &str) -> Sides {
        let mut sides = Sides::EMPTY;
        for side in s.split('|').map(str::trim) {
            match side {
                "top" => sides |= Sides::TOP,
                "right" => sides |= Sides::RIGHT,
                "bottom" => sides |= Sides::BOTTOM,
                "left" => sides |= Sides::LEFT,
                "all" => sides |= Sides::ALL,
                _ => {}
            }
        }
        sides
    }
}

// -----------------------------------------------------------------------------
//   - Border types -
// -----------------------------------------------------------------------------
pub const DEFAULT_SLIM_EDGES: [char; 8] = ['┌', '─', '┐', '│', '┘', '─', '└', '│'];
pub const DEFAULT_THICK_EDGES: [char; 8] = ['╔', '═', '╗', '║', '╝', '═', '╚', '║'];

/// The style of the border.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorderStyle {
    /// ```text
    /// ┌─────┐
    /// │hello│
    /// └─────┘
    /// ```
    Thin,
    /// ```text
    /// ╔═════╗
    /// ║hello║
    /// ╚═════╝
    /// ```
    Thick,
    /// ```text
    /// 0111112
    /// 7hello3
    /// 6555554
    /// ```
    Custom(String),
}

impl From<&str> for BorderStyle {
    fn from(s: &str) -> Self {
        match s {
            "thin" => Self::Thin,
            "thick" => Self::Thick,
            raw => Self::Custom(raw.to_string()),
        }
    }
}

impl BorderStyle {
    pub fn edges(&self) -> [char; 8] {
        match self {
            BorderStyle::Thin => DEFAULT_SLIM_EDGES,
            BorderStyle::Thick => DEFAULT_THICK_EDGES,
            BorderStyle::Custom(edge_string) => {
                let mut edges = [' '; 8];
                for (i, c) in edge_string.chars().take(8).enumerate() {
                    edges[i] = c;
                }
                edges
            }
        }
    }
}

/// Draw a border around an element.
///
/// The border will size it self around the child if it has one.
///
/// If a width and / or a height is provided then the border will produce tight constraints
/// for the child.
///
/// If a border has no size (width and height) and no child then nothing will be rendered.
///
/// To render a border with no child provide a width and a height.
#[derive(Debug, PartialEq)]
pub struct Border {
    /// Which sides of the border should be rendered.
    pub sides: Sides,
    /// All the characters for the border, starting from the top left moving clockwise.
    /// This means the top-left corner is `edges[0]`, the top if `edges[1]` and the top right is
    /// `edges[2]` etc.
    pub edges: [char; 8],
    /// The width of the border. This will make the constraints tight for the width.
    pub width: Option<usize>,
    /// The height of the border. This will make the constraints tight for the height.
    pub height: Option<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Option<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Option<usize>,
    /// The style of the border.
    pub style: Style,
}

impl Border {
    /// The name of the element
    pub const KIND: &'static str = "Border";

    /// Create a new instance of a border
    ///
    ///```
    /// use anathema_widgets::{Border, BorderStyle, Sides};
    /// let border = Border::new(BorderStyle::Thin, Sides::ALL, None, None);
    /// ```
    pub fn new(
        style: BorderStyle,
        sides: Sides,
        width: impl Into<Option<usize>>,
        height: impl Into<Option<usize>>,
    ) -> Self {
        let width = width.into();
        let height = height.into();

        let edges = style.edges();

        Self {
            sides,
            edges,
            width,
            height,
            min_width: None,
            min_height: None,
            style: Style::new(),
        }
    }

    /// Create a "thin" border with an optional width and height
    pub fn thin(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self::new(BorderStyle::Thin, Sides::ALL, width, height)
    }

    /// Create a "thick" border with an optional width and height
    pub fn thick(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self::new(BorderStyle::Thick, Sides::ALL, width, height)
    }

    fn border_size(&self) -> Size {
        // Get the size of the border (thickness).
        // This is NOT including the child.
        let mut border_width = 0;
        if self.sides.contains(Sides::LEFT) {
            let mut width = self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0);

            if self.sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_LEFT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        if self.sides.contains(Sides::RIGHT) {
            let mut width = self.edges[BORDER_EDGE_RIGHT].width().unwrap_or(0);

            if self.sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_RIGHT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        // Set the height of the border it self (thickness)
        let mut border_height = 0;
        if self.sides.contains(Sides::TOP) {
            let mut height = 1;

            if self.sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        if self.sides.contains(Sides::BOTTOM) {
            let mut height = 1;

            if self.sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[BORDER_EDGE_BOTTOM_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        Size::new(border_width, border_height)
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::new(BorderStyle::Thin, Sides::ALL, None, None)
    }
}

impl Widget for Border {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn layout<'widget, 'parent>(
        &mut self,
        mut ctx: LayoutCtx<'widget, 'parent>,
        children: &mut Vec<WidgetContainer>,
    ) -> Result<Size> {
        let border_layout = BorderLayout {
            min_height: self.min_height,
            min_width: self.min_width,
            height: self.height,
            width: self.width,
            border_size: self.border_size(),
        };
        let mut layout = Layouts::new(border_layout, &mut ctx);
        layout.layout(children)?.size()
    }

    fn position<'ctx>(&mut self, mut ctx: PositionCtx, children: &mut [WidgetContainer]) {
        let child = match children.first_mut() {
            Some(child) => child,
            None => return,
        };

        if self.sides.contains(Sides::TOP) {
            ctx.pos.y += 1;
        }

        if self.sides.contains(Sides::LEFT) {
            ctx.pos.x += self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0) as i32;
        }

        child.position(ctx.pos);
    }

    fn paint<'ctx>(&mut self, mut ctx: PaintCtx<'_, WithSize>, children: &mut [WidgetContainer]) {
        // Draw the child
        if let Some(child) = children.first_mut() {
            let clipping_region = ctx.create_region();

            let child_ctx = ctx.sub_context(Some(&clipping_region));

            child.paint(child_ctx);
        }

        // Draw the border
        let width = ctx.local_size.width;
        let height = ctx.local_size.height;

        // Only draw corners if there are connecting sides:
        // e.g Sides::Left | Sides::Top
        //
        // Don't draw corners if there are no connecting sides:
        // e.g Sides::Top | Sides::Bottom

        // Top left
        let pos = LocalPos::ZERO;
        if self.sides.contains(Sides::LEFT | Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP_LEFT], self.style, pos);
        } else if self.sides.contains(Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP], self.style, pos);
        } else if self.sides.contains(Sides::LEFT) {
            ctx.put(self.edges[BORDER_EDGE_LEFT], self.style, pos);
        }

        // Top right
        let pos = LocalPos::new(width.saturating_sub(1), 0);
        if self.sides.contains(Sides::RIGHT | Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP_RIGHT], self.style, pos);
        } else if self.sides.contains(Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP], self.style, pos);
        } else if self.sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[BORDER_EDGE_RIGHT], self.style, pos);
        }

        // Bottom left
        let pos = LocalPos::new(0, height.saturating_sub(1));
        if self.sides.contains(Sides::LEFT | Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM_LEFT], self.style, pos);
        } else if self.sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM], self.style, pos);
        } else if self.sides.contains(Sides::LEFT) {
            ctx.put(self.edges[BORDER_EDGE_LEFT], self.style, pos);
        }

        // Bottom right
        let pos = LocalPos::new(width.saturating_sub(1), height.saturating_sub(1));
        if self.sides.contains(Sides::RIGHT | Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM_RIGHT], self.style, pos);
        } else if self.sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM], self.style, pos);
        } else if self.sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[BORDER_EDGE_RIGHT], self.style, pos);
        }

        // Top
        if self.sides.contains(Sides::TOP) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, 0);
                ctx.put(self.edges[BORDER_EDGE_TOP], self.style, pos);
            }
        }

        // Bottom
        if self.sides.contains(Sides::BOTTOM) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, height.saturating_sub(1));
                ctx.put(self.edges[BORDER_EDGE_BOTTOM], self.style, pos);
            }
        }

        // Left
        if self.sides.contains(Sides::LEFT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(0, i);
                ctx.put(self.edges[BORDER_EDGE_LEFT], self.style, pos);
            }
        }

        // Right
        if self.sides.contains(Sides::RIGHT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(width.saturating_sub(1), i);
                ctx.put(self.edges[BORDER_EDGE_RIGHT], self.style, pos);
            }
        }
    }
}

pub(crate) struct BorderFactory;

impl WidgetFactory for BorderFactory {
    fn make(
        &self,
        values: ValuesAttributes<'_, '_>,
        _: Option<&TextPath>,
    ) -> Result<Box<dyn AnyWidget>> {
        let border_style = values
            .get_attrib(fields::BORDER_STYLE)
            .and_then(Value::to_str)
            .map(From::from)
            .unwrap_or(BorderStyle::Thin);

        let sides = values
            .get_attrib(fields::BORDER_STYLE)
            .and_then(Value::to_str)
            .map(From::from)
            .unwrap_or(Sides::ALL);

        let width = values.width();
        let height = values.height();

        let mut widget = Border::new(border_style, sides, width, height);
        widget.min_width = values.min_width();
        widget.min_height = values.min_height();
        widget.style = values.style();
        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::template::template_text;
    use anathema_widget_core::testing::FakeTerm;

    use super::*;
    use crate::testing::test_widget;

    #[test]
    fn thin_border() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::ALL, 5, 4),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║┌───┐               ║
            ║│   │               ║
            ║│   │               ║
            ║└───┘               ║
            ║                    ║
            ║                    ║
            ╚════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn thick_border() {
        test_widget(
            Border::new(BorderStyle::Thick, Sides::ALL, 5, 4),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║╔═══╗               ║
            ║║   ║               ║
            ║║   ║               ║
            ║╚═══╝               ║
            ║                    ║
            ║                    ║
            ╚════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn custom_border() {
        test_widget(
            Border::new(
                BorderStyle::Custom("01234567".to_string()),
                Sides::ALL,
                5,
                4,
            ),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══════╗
            ║01112               ║
            ║7   3               ║
            ║7   3               ║
            ║65554               ║
            ║                    ║
            ║                    ║
            ╚════════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn border_top() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::TOP, 5, 2),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║─────           ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn border_top_bottom() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::TOP | Sides::BOTTOM, 5, 4),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║─────           ║
            ║                ║
            ║                ║
            ║─────           ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn border_left() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::LEFT, 1, 2),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║│               ║
            ║│               ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn border_right() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::RIGHT, 3, 2),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║  │             ║
            ║  │             ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn border_top_left() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::TOP | Sides::LEFT, 4, 3),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║┌───            ║
            ║│               ║
            ║│               ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn border_bottom_right() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::BOTTOM | Sides::RIGHT, 4, 3),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║   │            ║
            ║   │            ║
            ║───┘            ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn unsized_empty_border() {
        test_widget(
            Border::new(BorderStyle::Thin, Sides::BOTTOM | Sides::RIGHT, None, None),
            [],
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [══╗
            ║                ║
            ║                ║
            ║                ║
            ║                ║
            ╚════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn sized_by_child() {
        let body = [template_text("hello world")];
        test_widget(
            Border::new(BorderStyle::Thin, Sides::ALL, None, None),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [════╗
            ║┌───────────┐     ║
            ║│hello world│     ║
            ║└───────────┘     ║
            ║                  ║
            ╚══════════════════╝
            "#,
            ),
        );
    }

    #[test]
    fn fixed_size() {
        let body = [template_text("hello world")];
        test_widget(
            Border::new(BorderStyle::Thin, Sides::ALL, 7, 4),
            body,
            FakeTerm::from_str(
                r#"
            ╔═] Fake term [═══╗
            ║┌─────┐          ║
            ║│hello│          ║
            ║│world│          ║
            ║└─────┘          ║
            ║                 ║
            ╚═════════════════╝
            "#,
            ),
        );
    }
}
