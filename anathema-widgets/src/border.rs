use std::fmt::Display;

use anathema_render::{Size, Style};
use anathema_values::{
    impl_dyn_value, Attributes, Context, DynValue, NodeId, Resolver, Value, ValueExpr, ValueRef,
    ValueResolver,
};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx, PositionCtx, WithSize};
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::{Layout, Layouts};
use anathema_widget_core::{
    AnyWidget, FactoryContext, LayoutNodes, LocalPos, Nodes, Widget, WidgetContainer,
    WidgetFactory, WidgetStyle,
};
use smallvec::SmallVec;
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
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
        const ALL = Self::TOP.bits() | Self::RIGHT.bits() | Self::BOTTOM.bits() | Self::LEFT.bits();
    }
}

impl Default for Sides {
    fn default() -> Self {
        Self::ALL
    }
}

impl DynValue for Sides {
    fn init_value(
        context: &Context<'_, '_>,
        node_id: Option<&NodeId>,
        expr: &ValueExpr,
    ) -> Value<Self> {
        let mut resolver = Resolver::new(context, node_id);
        let inner = resolver.resolve_list(expr);

        match resolver.is_deferred() {
            true => Value::Dyn {
                inner: Some(inner.into()),
                expr: expr.clone(),
            },
            false => match inner.is_empty() {
                false => Value::Static(inner.into()),
                true => Value::Empty,
            },
        }
    }

    fn resolve(value: &mut Value<Self>, context: &Context<'_, '_>, node_id: Option<&NodeId>) {
        match value {
            Value::Dyn { inner, expr } => {
                let sides = Resolver::new(context, node_id).resolve_list::<String>(expr);
                *inner = Some(sides.into())
            }
            _ => {}
        }
    }
}

impl From<SmallVec<[String; 4]>> for Sides {
    fn from(value: SmallVec<[String; 4]>) -> Self {
        let mut sides = Sides::EMPTY;
        for side in value {
            match side.as_str() {
                "all" => sides |= Sides::ALL,
                "top" => sides |= Sides::TOP,
                "left" => sides |= Sides::LEFT,
                "right" => sides |= Sides::RIGHT,
                "bottom" => sides |= Sides::BOTTOM,
                _ => {}
            }
        }

        sides
    }
}

impl Into<ValueExpr> for Sides {
    fn into(self) -> ValueExpr {
        let mut sides = vec![];

        for side in self {
            if side.contains(Sides::ALL) {
                sides.push("all".into());
            }
            if side.contains(Sides::TOP) {
                sides.push("top".into());
            }
            if side.contains(Sides::RIGHT) {
                sides.push("right".into());
            }
            if side.contains(Sides::BOTTOM) {
                sides.push("bottom".into());
            }
            if side.contains(Sides::LEFT) {
                sides.push("left".into());
            }
        }

        ValueExpr::List(sides.into())
    }
}

// -----------------------------------------------------------------------------
//   - Border types -
// -----------------------------------------------------------------------------
pub const DEFAULT_SLIM_EDGES: [char; 8] = ['┌', '─', '┐', '│', '┘', '─', '└', '│'];
pub const DEFAULT_THICK_EDGES: [char; 8] = ['╔', '═', '╗', '║', '╝', '═', '╚', '║'];

/// The style of the border.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum BorderStyle {
    /// ```text
    /// ┌─────┐
    /// │hello│
    /// └─────┘
    /// ```
    #[default]
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

impl_dyn_value!(BorderStyle);

impl TryFrom<ValueRef<'_>> for BorderStyle {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            ValueRef::Str("thin") => Self::Thin,
            ValueRef::Str("thick") => Self::Thick,
            ValueRef::Str(raw) => Self::Custom(raw.to_string()),
            _ => Self::default(),
        })
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

impl Display for BorderStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thin => write!(f, "thin"),
            Self::Thick => write!(f, "thick"),
            Self::Custom(s) => write!(f, "{s}"),
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
#[derive(Debug)]
pub struct Border {
    /// The border style decides the characters
    /// to be used for each side of the border.
    pub border_style: Value<BorderStyle>,
    /// Which sides of the border should be rendered.
    pub sides: Value<Sides>,
    /// All the characters for the border, starting from the top left moving clockwise.
    /// This means the top-left corner is `edges[0]`, the top if `edges[1]` and the top right is
    /// `edges[2]` etc.
    pub edges: [char; 8],
    /// The width of the border. This will make the constraints tight for the width.
    pub width: Value<usize>,
    /// The height of the border. This will make the constraints tight for the height.
    pub height: Value<usize>,
    /// The minimum width of the border. This will force the minimum constrained width to expand to
    /// this value.
    pub min_width: Value<usize>,
    /// The minimum height of the border. This will force the minimum constrained height to expand to
    /// this value.
    pub min_height: Value<usize>,
    /// The style of the border.
    pub style: WidgetStyle,
}

impl Border {
    /// The name of the element
    pub const KIND: &'static str = "Border";

    fn border_size(&self) -> Size {
        // Get the size of the border (thickness).
        // This is NOT including the child.
        let mut border_width = 0;
        let sides = self.sides.value_or(Sides::ALL);

        if sides.contains(Sides::LEFT) {
            let mut width = self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0);

            if sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_LEFT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        if sides.contains(Sides::RIGHT) {
            let mut width = self.edges[BORDER_EDGE_RIGHT].width().unwrap_or(0);

            if sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BORDER_EDGE_BOTTOM_RIGHT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        // Set the height of the border it self (thickness)
        let mut border_height = 0;
        if sides.contains(Sides::TOP) {
            let mut height = 1;

            if sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        if sides.contains(Sides::BOTTOM) {
            let mut height = 1;

            if sides.contains(Sides::LEFT | Sides::RIGHT) {
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

impl Widget for Border {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn update(&mut self, context: &Context<'_, '_>, node_id: &NodeId) {
        self.style.resolve(context, None);
        self.border_style.resolve(context, None);
        self.sides.resolve(context, None);
        self.height.resolve(context, None);
        self.width.resolve(context, None);
        self.min_width.resolve(context, None);
        self.min_height.resolve(context, None);
    }

    fn layout<'e>(&mut self, nodes: &mut LayoutNodes<'_, '_, 'e>) -> Result<Size> {
        let mut layout = BorderLayout {
            min_height: self.min_height.value(),
            min_width: self.min_width.value(),
            height: self.height.value(),
            width: self.width.value(),
            border_size: self.border_size(),
        };
        layout.layout(nodes)
    }

    fn position(&mut self, children: &mut Nodes, mut ctx: PositionCtx) {
        let (child, children) = match children.first_mut() {
            Some(child) => child,
            None => return,
        };

        if self.sides.value_or_default().contains(Sides::TOP) {
            ctx.pos.y += 1;
        }

        if self.sides.value_or_default().contains(Sides::LEFT) {
            ctx.pos.x += self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0) as i32;
        }

        child.position(children, ctx.pos);
    }

    fn paint(&mut self, children: &mut Nodes, mut ctx: PaintCtx<'_, WithSize>) {
        // Draw the child
        if let Some((child, children)) = children.first_mut() {
            let clipping_region = ctx.create_region();

            // let child_ctx = ctx.sub_context(Some(&clipping_region));
            let child_ctx = ctx.sub_context(None);

            child.paint(children, child_ctx);
        }

        // Draw the border
        let width = ctx.local_size.width;
        let height = ctx.local_size.height;

        let sides = self.sides.value_or_default();
        let style = self.style.style();

        // Only draw corners if there are connecting sides:
        // e.g Sides::Left | Sides::Top
        //
        // Don't draw corners if there are no connecting sides:
        // e.g Sides::Top | Sides::Bottom

        // Top left
        let pos = LocalPos::ZERO;
        if sides.contains(Sides::LEFT | Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP_LEFT], style, pos);
        } else if sides.contains(Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP], style, pos);
        } else if sides.contains(Sides::LEFT) {
            ctx.put(self.edges[BORDER_EDGE_LEFT], style, pos);
        }

        // Top right
        let pos = LocalPos::new(width.saturating_sub(1), 0);
        if sides.contains(Sides::RIGHT | Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP_RIGHT], style, pos);
        } else if sides.contains(Sides::TOP) {
            ctx.put(self.edges[BORDER_EDGE_TOP], style, pos);
        } else if sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[BORDER_EDGE_RIGHT], style, pos);
        }

        // Bottom left
        let pos = LocalPos::new(0, height.saturating_sub(1));
        if sides.contains(Sides::LEFT | Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM_LEFT], style, pos);
        } else if sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM], style, pos);
        } else if sides.contains(Sides::LEFT) {
            ctx.put(self.edges[BORDER_EDGE_LEFT], style, pos);
        }

        // Bottom right
        let pos = LocalPos::new(width.saturating_sub(1), height.saturating_sub(1));
        if sides.contains(Sides::RIGHT | Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM_RIGHT], style, pos);
        } else if sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BORDER_EDGE_BOTTOM], style, pos);
        } else if sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[BORDER_EDGE_RIGHT], style, pos);
        }

        // Top
        if sides.contains(Sides::TOP) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, 0);
                ctx.put(self.edges[BORDER_EDGE_TOP], style, pos);
            }
        }

        // Bottom
        if sides.contains(Sides::BOTTOM) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, height.saturating_sub(1));
                ctx.put(self.edges[BORDER_EDGE_BOTTOM], style, pos);
            }
        }

        // Left
        if sides.contains(Sides::LEFT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(0, i);
                ctx.put(self.edges[BORDER_EDGE_LEFT], style, pos);
            }
        }

        // Right
        if sides.contains(Sides::RIGHT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(width.saturating_sub(1), i);
                ctx.put(self.edges[BORDER_EDGE_RIGHT], style, pos);
            }
        }
    }
}

pub(crate) struct BorderFactory;

impl WidgetFactory for BorderFactory {
    fn make(&self, ctx: FactoryContext<'_>) -> Result<Box<dyn AnyWidget>> {
        let border_style = ctx.get::<BorderStyle>("border-style");
        let edges = border_style
            .value_ref()
            .map(|s| s.edges())
            .unwrap_or(DEFAULT_SLIM_EDGES);

        let widget = Border {
            edges,
            border_style,
            sides: ctx.get("sides"),
            width: ctx.get("width"),
            height: ctx.get("height"),
            min_width: ctx.get("min_width"),
            min_height: ctx.get("min_height"),
            style: ctx.style(),
        };

        Ok(Box::new(widget))
    }
}

#[cfg(test)]
mod test {
    use anathema_widget_core::generator::Expression;
    use anathema_widget_core::testing::{expression, FakeTerm};

    use super::*;
    use crate::testing::test_widget;

    fn border(
        border_style: BorderStyle,
        sides: Sides,
        width: Option<usize>,
        height: Option<usize>,
        text: Option<&'static str>,
    ) -> Expression {
        let mut attribs = vec![("border-style".into(), border_style.to_string().into())];

        if let Some(width) = width {
            attribs.push(("width".to_string(), width.into()))
        }

        if let Some(height) = height {
            attribs.push(("height".into(), height.into()))
        }

        attribs.push(("sides".into(), sides.into()));

        let children = match text {
            Some(t) => vec![expression("text", Some(t), [], [])],
            None => vec![],
        };
        expression("border", None, attribs, children)
    }

    #[test]
    fn thin_border() {
        test_widget(
            border(BorderStyle::Thin, Sides::ALL, Some(5), Some(4), None),
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
            border(BorderStyle::Thick, Sides::ALL, Some(5), Some(4), None),
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
            border(
                BorderStyle::Custom("01234567".to_string()),
                Sides::ALL,
                Some(5),
                Some(4),
                None,
            ),
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
            border(BorderStyle::Thin, Sides::TOP, Some(5), Some(2), None),
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
            border(
                BorderStyle::Thin,
                Sides::TOP | Sides::BOTTOM,
                Some(5),
                Some(4),
                None,
            ),
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
            border(BorderStyle::Thin, Sides::LEFT, Some(1), Some(2), None),
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
            border(BorderStyle::Thin, Sides::RIGHT, Some(3), Some(2), None),
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
            border(
                BorderStyle::Thin,
                Sides::TOP | Sides::LEFT,
                Some(4),
                Some(3),
                None,
            ),
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
            border(
                BorderStyle::Thin,
                Sides::BOTTOM | Sides::RIGHT,
                Some(4),
                Some(3),
                None,
            ),
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
            border(
                BorderStyle::Thin,
                Sides::BOTTOM | Sides::RIGHT,
                None,
                None,
                None,
            ),
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
        test_widget(
            border(
                BorderStyle::Thin,
                Sides::ALL,
                None,
                None,
                Some("hello world"),
            ),
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
        test_widget(
            border(
                BorderStyle::Thin,
                Sides::ALL,
                Some(7),
                Some(4),
                Some("hello world"),
            ),
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
