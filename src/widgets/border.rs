use unicode_width::UnicodeWidthChar;

use crate::display::{Size, Style};

use super::LocalPos;
use super::{LayoutCtx, NodeId, PaintCtx, PositionCtx, Widget, WidgetContainer, WithSize};
use crate::widgets::{fields, Attributes};

const DEFAULT_SLIM_EDGES: [char; 8] = ['┌', '─', '┐', '│', '┘', '─', '└', '│'];
const DEFAULT_THICK_EDGES: [char; 8] = ['╔', '═', '╗', '║', '╝', '═', '╚', '║'];

// -----------------------------------------------------------------------------
//     - Indices -
//     Index into `DEFAULT_SLIM_EDGES` or `DEFAULT_THICK_EDGES`
// -----------------------------------------------------------------------------
const TOP_LEFT: usize = 0;
const TOP: usize = 1;
const TOP_RIGHT: usize = 2;
const RIGHT: usize = 3;
const BOTTOM_RIGHT: usize = 4;
const BOTTOM: usize = 5;
const BOTTOM_LEFT: usize = 6;
const LEFT: usize = 7;

// -----------------------------------------------------------------------------
//     - Sides -
// -----------------------------------------------------------------------------
bitflags::bitflags! {
    /// Border sides
    /// ```
    /// use anathema::widgets::Sides;
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

/// The style of the border.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BorderStyle {
    ///```text
    ///┌─────┐
    ///│hello│
    ///└─────┘
    ///```
    Thin,
    ///```text
    ///╔═════╗
    ///║hello║
    ///╚═════╝
    ///```
    Thick,
    ///```text
    ///0111112
    ///7hello3
    ///6555554
    ///```
    Custom(String),
}

impl BorderStyle {
    fn edges(&self) -> [char; 8] {
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
#[derive(Debug)]
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
    /// If the border has a child widget, it will size it self around the child.
    pub child: Option<WidgetContainer>,
    /// The style of the border.
    pub style: Style,
}

impl Border {
    /// The name of the element
    pub const KIND: &'static str = "Border";

    /// Create a new instance of a border
    ///
    /// ```
    /// use anathema::widgets::{Border, BorderStyle, Sides};
    /// let border_style = BorderStyle::Thin;
    /// let border = Border::new(&border_style, Sides::ALL, None, None);
    /// ```
    pub fn new(
        style: &BorderStyle,
        sides: Sides,
        width: impl Into<Option<usize>>,
        height: impl Into<Option<usize>>,
    ) -> Self {
        let width = width.into();
        let height = height.into();

        let edges = style.edges();
        Self { sides, edges, width, height, min_width: None, min_height: None, child: None, style: Style::new() }
    }

    /// Create a "thin" border with an optional width and height
    pub fn thin(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self::new(&BorderStyle::Thin, Sides::ALL, width, height)
    }

    /// Create a "thick" border with an optional width and height
    pub fn thick(width: impl Into<Option<usize>>, height: impl Into<Option<usize>>) -> Self {
        Self::new(&BorderStyle::Thick, Sides::ALL, width, height)
    }

    fn border_size(&self) -> Size {
        // Get the size of the border (thickness).
        // This is NOT including the child.
        let mut border_width = 0;
        if self.sides.contains(Sides::LEFT) {
            let mut width = self.edges[LEFT].width().unwrap_or(0);

            if self.sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[TOP_LEFT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BOTTOM_LEFT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        if self.sides.contains(Sides::RIGHT) {
            let mut width = self.edges[RIGHT].width().unwrap_or(0);

            if self.sides.contains(Sides::TOP | Sides::BOTTOM) {
                let corner = self.edges[TOP_RIGHT].width().unwrap_or(0);
                width = width.max(corner);

                let corner = self.edges[BOTTOM_RIGHT].width().unwrap_or(0);
                width = width.max(corner);
            }
            border_width += width;
        }

        // Set the height of the border it self (thickness)
        let mut border_height = 0;
        if self.sides.contains(Sides::TOP) {
            let mut height = 1;

            if self.sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[TOP_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[TOP_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        if self.sides.contains(Sides::BOTTOM) {
            let mut height = 1;

            if self.sides.contains(Sides::LEFT | Sides::RIGHT) {
                let corner = self.edges[BOTTOM_LEFT].width().unwrap_or(0);
                height = height.max(corner);

                let corner = self.edges[BOTTOM_RIGHT].width().unwrap_or(0);
                height = height.max(corner);
            }
            border_height += height;
        }

        Size::new(border_width, border_height)
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::new(&BorderStyle::Thin, Sides::ALL, None, None)
    }
}

impl Widget for Border {
    fn kind(&self) -> &'static str {
        Self::KIND
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn layout(&mut self, mut ctx: LayoutCtx) -> Size {
        // If there is a min width / height, make sure the minimum constraints
        // are matching these
        if let Some(min_width) = self.min_width {
            ctx.constraints.min_width = ctx.constraints.min_width.max(min_width);
        }

        if let Some(min_height) = self.min_height {
            ctx.constraints.min_height = ctx.constraints.min_height.max(min_height);
        }

        // If there is a width / height then make the constraints tight
        // around the size. This will modify the size to fit within the
        // constraints first.
        if let Some(width) = self.width {
            ctx.constraints.make_width_tight(width);
        }

        if let Some(height) = self.height {
            ctx.constraints.make_height_tight(height);
        }

        let border_size = self.border_size();

        match self.child.as_mut() {
            Some(child) => {
                let mut constraints = ctx.padded_constraints();

                // Shrink the constraint for the child to fit inside the border
                constraints.max_width = constraints.max_width.saturating_sub(border_size.width);
                if constraints.min_width > constraints.max_width {
                    constraints.min_width = constraints.max_width;
                }

                constraints.max_height = constraints.max_height.saturating_sub(border_size.height);
                if constraints.min_height > constraints.max_height {
                    constraints.min_height = constraints.max_height;
                }

                let mut size = child.layout(constraints, ctx.force_layout) + border_size + ctx.padding_size();

                if let Some(min_width) = self.min_width {
                    size.width = size.width.max(min_width);
                }

                if let Some(min_height) = self.min_height {
                    size.height = size.height.max(min_height);
                }

                if ctx.constraints.is_width_tight() {
                    size.width = ctx.constraints.max_width;
                }
                if ctx.constraints.is_height_tight() {
                    size.height = ctx.constraints.max_height;
                }
                size
            }
            None => {
                let mut size = Size::new(ctx.constraints.min_width, ctx.constraints.min_height);
                if ctx.constraints.is_width_tight() {
                    size.width = ctx.constraints.max_width;
                }
                if ctx.constraints.is_height_tight() {
                    size.height = ctx.constraints.max_height;
                }
                size
            }
        }
    }

    fn position(&mut self, mut ctx: PositionCtx) {
        let child = match self.child.as_mut() {
            Some(child) => child,
            None => return,
        };

        if self.sides.contains(Sides::TOP) {
            ctx.pos.y += 1;
        }

        if self.sides.contains(Sides::LEFT) {
            ctx.pos.x += self.edges[LEFT].width().unwrap_or(0) as i32;
        }

        child.position(ctx.padded_position());
    }

    fn paint(&mut self, mut ctx: PaintCtx<'_, WithSize>) {
        // Draw the child
        let border_size = self.border_size();
        if let Some(child) = self.child.as_mut() {
            let mut clipping_region = ctx.create_region();
            clipping_region.to.x -= border_size.width as i32 / 2;
            clipping_region.to.y -= border_size.height as i32 / 2;

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
            ctx.put(self.edges[TOP_LEFT], self.style, pos);
        } else if self.sides.contains(Sides::TOP) {
            ctx.put(self.edges[TOP], self.style, pos);
        } else if self.sides.contains(Sides::LEFT) {
            ctx.put(self.edges[LEFT], self.style, pos);
        }

        // Top right
        let pos = LocalPos::new(width.saturating_sub(1), 0);
        if self.sides.contains(Sides::RIGHT | Sides::TOP) {
            ctx.put(self.edges[TOP_RIGHT], self.style, pos);
        } else if self.sides.contains(Sides::TOP) {
            ctx.put(self.edges[TOP], self.style, pos);
        } else if self.sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[RIGHT], self.style, pos);
        }

        // Bottom left
        let pos = LocalPos::new(0, height.saturating_sub(1));
        if self.sides.contains(Sides::LEFT | Sides::BOTTOM) {
            ctx.put(self.edges[BOTTOM_LEFT], self.style, pos);
        } else if self.sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BOTTOM], self.style, pos);
        } else if self.sides.contains(Sides::LEFT) {
            ctx.put(self.edges[LEFT], self.style, pos);
        }

        // Bottom right
        let pos = LocalPos::new(width.saturating_sub(1), height.saturating_sub(1));
        if self.sides.contains(Sides::RIGHT | Sides::BOTTOM) {
            ctx.put(self.edges[BOTTOM_RIGHT], self.style, pos);
        } else if self.sides.contains(Sides::BOTTOM) {
            ctx.put(self.edges[BOTTOM], self.style, pos);
        } else if self.sides.contains(Sides::RIGHT) {
            ctx.put(self.edges[RIGHT], self.style, pos);
        }

        // Top
        if self.sides.contains(Sides::TOP) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, 0);
                ctx.put(self.edges[TOP], self.style, pos);
            }
        }

        // Bottom
        if self.sides.contains(Sides::BOTTOM) {
            for i in 1..width.saturating_sub(1) {
                let pos = LocalPos::new(i, height.saturating_sub(1));
                ctx.put(self.edges[BOTTOM], self.style, pos);
            }
        }

        // Left
        if self.sides.contains(Sides::LEFT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(0, i);
                ctx.put(self.edges[LEFT], self.style, pos);
            }
        }

        // Right
        if self.sides.contains(Sides::RIGHT) {
            for i in 1..height.saturating_sub(1) {
                let pos = LocalPos::new(width.saturating_sub(1), i);
                ctx.put(self.edges[RIGHT], self.style, pos);
            }
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

    fn update(&mut self, attributes: Attributes) {
        attributes.update_style(&mut self.style);
        for (k, _) in &attributes {
            match k.as_str() {
                fields::WIDTH => self.width = attributes.width(),
                fields::HEIGHT => self.height = attributes.height(),
                fields::BORDER_STYLE => self.edges = attributes.border_style().edges(),
                fields::SIDES => self.sides = attributes.sides(),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::widgets::testing::test_widget;
    use crate::widgets::Constraints;

    fn test_border(sides: Sides, expected: &str) {
        test_widget(Border::new(&BorderStyle::Thin, sides, None, None), expected);
    }

    #[test]
    fn border() {
        test_border(
            Sides::ALL,
            r#"
            ┌───┐
            │   │
            │   │
            └───┘
            "#,
        );
    }

    #[test]
    fn border_top() {
        test_border(
            Sides::TOP,
            r#"
            ─────
            "#,
        );
    }

    #[test]
    fn border_top_bottom() {
        test_border(
            Sides::TOP | Sides::BOTTOM,
            r#"
            ─────
            ─────
            "#,
        );
    }

    #[test]
    fn border_left() {
        test_border(
            Sides::LEFT,
            r#"
            │
            │
            "#,
        );
    }

    #[test]
    fn border_right() {
        test_border(
            Sides::RIGHT,
            r#"
                │
                │
            "#,
        );
    }

    #[test]
    fn border_bottom_right() {
        test_border(
            Sides::TOP | Sides::LEFT,
            r#"
            ┌───
            │   
            │   
            "#,
        );
    }

    #[test]
    fn style_changes_via_attributes() {
        let mut border = Border::thick(10, 3).into_container(NodeId::auto());
        border.update(Attributes::new("italic", true));
        assert!(border.to::<Border>().style.attributes.contains(crate::display::Attributes::ITALIC));
    }

    #[test]
    fn min_width_height() {
        let mut border = Border::thick(10, 3);
        border.min_width = Some(10);
        border.min_height = Some(3);
        let mut border = border.into_container(NodeId::auto());
        border.layout(Constraints::unbounded(), false);
        let actual = border.size();
        assert_eq!(Size::new(10, 3), actual);
    }
}
