use std::convert::Infallible;
use std::fmt::Display;
use std::ops::{ControlFlow, Deref};

use anathema_geometry::{LocalPos, Pos, Rect, Size};
use anathema_widgets::expressions::EvalValue;
use anathema_widgets::layout::{Constraints, LayoutCtx, PositionCtx};
use anathema_widgets::paint::{PaintCtx, SizePos};
use anathema_widgets::{
    AnyWidget, AttributeStorage, Attributes, LayoutChildren, PaintChildren, PositionChildren, Widget, WidgetId,
};
use unicode_width::UnicodeWidthChar;

use crate::layout::border::BorderLayout;
use crate::layout::Axis;
use crate::{HEIGHT, MAX_HEIGHT, MAX_WIDTH, MIN_HEIGHT, MIN_WIDTH, WIDTH};

pub const BORDER_STYLE: &str = "border_style";

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
    /// ```ignore
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

impl TryFrom<&EvalValue<'_>> for Sides {
    type Error = ();

    fn try_from(value: &EvalValue<'_>) -> Result<Self, Self::Error> {
        let mut sides = Sides::EMPTY;
        value.str_for_each(|s| sides |= s.into());
        Ok(sides)
    }
}

impl From<&str> for Sides {
    fn from(value: &str) -> Self {
        match value {
            "all" => Sides::ALL,
            "top" => Sides::TOP,
            "left" => Sides::LEFT,
            "right" => Sides::RIGHT,
            "bottom" => Sides::BOTTOM,
            _ => Sides::EMPTY,
        }
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

impl TryFrom<&EvalValue<'_>> for BorderStyle {
    type Error = Infallible;

    fn try_from(value: &EvalValue<'_>) -> Result<Self, Self::Error> {
        let mut style = None::<BorderStyle>;
        value.str_for_each(|s| match s {
            "thin" => style = Some(BorderStyle::Thin),
            "thick" => style = Some(BorderStyle::Thick),
            custom => style = Some(BorderStyle::Custom(custom.into())),
        });

        Ok(style.unwrap_or_default())
    }
}

struct Brush {
    glyph: char,
    width: u8,
}

impl Brush {
    pub fn new(glyph: char, width: u8) -> Self {
        Self { width, glyph }
    }
}

struct BorderPainter {
    top: Line,
    bottom: Line,
    left: Line,
    right: Line,
}

struct Line {
    start_cap: Option<Brush>,
    middle: Option<Brush>,
    end_cap: Option<Brush>,
    start: LocalPos,
    end: u16,
    axis: Axis,
}

impl Line {
    fn will_draw(&self) -> bool {
        self.start_cap.is_some() || self.end_cap.is_some() || self.middle.is_some()
    }

    fn draw<F>(&self, f: &mut F)
    where
        F: FnMut(LocalPos, char),
    {
        let mut pos = self.start;
        let mut end = self.end;

        if let Some(brush) = &self.start_cap {
            f(pos, brush.glyph);
            match self.axis {
                Axis::Horizontal => pos.x += brush.width as u16,
                Axis::Vertical => pos.y += 1,
            }
        }

        if let Some(brush) = &self.end_cap {
            let pos = match self.axis {
                Axis::Horizontal => {
                    end -= brush.width as u16;
                    LocalPos::new(end, pos.y)
                }
                Axis::Vertical => {
                    end -= 1;
                    LocalPos::new(pos.x, end)
                }
            };
            f(pos, brush.glyph);
        }

        if let Some(brush) = &self.middle {
            loop {
                match self.axis {
                    Axis::Horizontal => {
                        if pos.x + brush.width as u16 > end {
                            break;
                        }
                        f(pos, brush.glyph);
                        pos.x += brush.width as u16;
                    }
                    Axis::Vertical => {
                        if pos.y + 1 > end {
                            break;
                        }
                        f(pos, brush.glyph);
                        pos.y += 1;
                    }
                }
            }
        }
    }
}

impl BorderPainter {
    fn new(glyphs: &[char; 8], border_size: BorderSize, size: Size) -> Self {
        let mut height = size.height;

        let top = Line {
            start_cap: (border_size.top_left > 0).then(|| Brush::new(glyphs[0], border_size.top_left)),
            middle: (border_size.top > 0).then(|| Brush::new(glyphs[1], border_size.top)),
            end_cap: (border_size.top_right > 0).then(|| Brush::new(glyphs[2], border_size.top_right)),
            start: LocalPos::ZERO,
            axis: Axis::Horizontal,
            end: size.width as u16,
        };

        let bottom = Line {
            start_cap: (border_size.bottom_left > 0).then(|| Brush::new(glyphs[6], border_size.bottom_left)),
            middle: (border_size.bottom > 0).then(|| Brush::new(glyphs[5], border_size.bottom)),
            end_cap: (border_size.bottom_right > 0).then(|| Brush::new(glyphs[4], border_size.bottom_right)),
            start: LocalPos::new(0, height as u16 - 1),
            axis: Axis::Horizontal,
            end: size.width as u16,
        };

        if bottom.will_draw() {
            height -= 1;
        }

        let mut offset = 0;
        if top.will_draw() {
            offset += 1;
        }

        let left = Line {
            start_cap: None,
            middle: (border_size.left > 0).then(|| Brush::new(glyphs[7], border_size.left)),
            end_cap: None,
            start: LocalPos::new(0, offset),
            axis: Axis::Vertical,
            end: height as u16,
        };

        let right = Line {
            start_cap: None,
            middle: (border_size.right > 0).then(|| Brush::new(glyphs[3], border_size.right)),
            end_cap: None,
            start: LocalPos::new((size.width - border_size.right as usize) as u16, offset),
            axis: Axis::Vertical,
            end: height as u16,
        };

        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    fn paint<F>(&mut self, f: &mut F)
    where
        F: FnMut(LocalPos, char),
    {
        self.top.draw(f);
        self.bottom.draw(f);
        self.left.draw(f);
        self.right.draw(f);
    }
}

/// Width of every character that makes up the border
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct BorderSize {
    pub top_left: u8,
    pub top: u8,
    pub top_right: u8,
    pub right: u8,
    pub bottom_right: u8,
    pub bottom: u8,
    pub bottom_left: u8,
    pub left: u8,
}

impl BorderSize {
    pub(crate) fn as_size(&self) -> Size {
        let left_width = self.left.max(self.top_left).max(self.bottom_left);
        let right_width = self.right.max(self.top_right).max(self.bottom_right);

        Size {
            width: (left_width + right_width) as usize,
            height: (self.top + self.bottom) as usize,
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
    border_style: BorderStyle,
    /// Which sides of the border should be rendered.
    sides: Sides,
    /// All the characters for the border, starting from the top left moving clockwise.
    /// This means the top-left corner is `edges[0]`, the top if `edges[1]` and the top right is
    /// `edges[2]` etc.
    edges: [char; 8],
}

impl Border {
    // The additional size of the border
    // to subtract from the constraint.
    fn border_size(&self, sides: Sides) -> BorderSize {
        // Get the size of the border (thickness).
        // This is NOT including the child.

        let mut border_size = BorderSize::default();

        if sides.contains(Sides::LEFT | Sides::TOP) {
            border_size.top_left = self.edges[BORDER_EDGE_TOP_LEFT].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::LEFT | Sides::BOTTOM) {
            border_size.bottom_left = self.edges[BORDER_EDGE_BOTTOM_LEFT].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::RIGHT | Sides::BOTTOM) {
            border_size.bottom_right = self.edges[BORDER_EDGE_BOTTOM_RIGHT].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::RIGHT | Sides::TOP) {
            border_size.top_right = self.edges[BORDER_EDGE_TOP_RIGHT].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::LEFT) {
            border_size.left = self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::RIGHT) {
            border_size.right = self.edges[BORDER_EDGE_RIGHT].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::TOP) {
            border_size.top = self.edges[BORDER_EDGE_TOP].width().unwrap_or(0) as u8;
        }

        if sides.contains(Sides::BOTTOM) {
            border_size.bottom = self.edges[BORDER_EDGE_BOTTOM].width().unwrap_or(0) as u8;
        }

        border_size
    }
}

impl Widget for Border {
    fn layout<'bp>(
        &mut self,
        children: LayoutChildren<'_, '_, 'bp>,
        constraints: Constraints,
        id: WidgetId,
        ctx: &mut LayoutCtx<'_, 'bp>,
    ) -> Size {
        let attributes = ctx.attribs.get(id);
        self.sides = attributes
            .get_val("sides")
            .and_then(|s| Sides::try_from(s.deref()).ok())
            .unwrap_or_default();

        self.border_style = attributes.get_ref(BORDER_STYLE).unwrap_or_default();
        self.edges = self.border_style.edges();

        let mut layout = BorderLayout {
            min_width: attributes.get_usize(MIN_WIDTH),
            min_height: attributes.get_usize(MIN_HEIGHT),
            max_width: attributes.get_usize(MAX_WIDTH),
            max_height: attributes.get_usize(MAX_HEIGHT),
            height: attributes.get_usize(HEIGHT),
            width: attributes.get_usize(WIDTH),
            border_size: self.border_size(self.sides),
        };

        layout.layout(children, constraints, ctx)
    }

    fn position<'bp>(
        &mut self,
        mut children: PositionChildren<'_, '_, 'bp>,
        _: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PositionCtx,
    ) {
        children.for_each(|child, children| {
            if self.sides.contains(Sides::TOP) {
                ctx.pos.y += 1;
            }

            if self.sides.contains(Sides::LEFT) {
                ctx.pos.x += self.edges[BORDER_EDGE_LEFT].width().unwrap_or(0) as i32;
            }

            child.position(children, ctx.pos, attribute_storage, ctx.viewport);
            ControlFlow::Break(())
        });
    }

    fn paint<'bp>(
        &mut self,
        mut children: PaintChildren<'_, '_, 'bp>,
        _id: WidgetId,
        attribute_storage: &AttributeStorage<'bp>,
        mut ctx: PaintCtx<'_, SizePos>,
    ) {
        let border_size = self.border_size(self.sides);

        children.for_each(|child, children| {
            let ctx = ctx.to_unsized();
            child.paint(children, ctx, attribute_storage);
            ControlFlow::Break(())
        });

        // Draw the border
        // Only draw corners if there are connecting sides:
        // e.g Sides::Left | Sides::Top
        //
        // Don't draw corners if there are no connecting sides:
        // e.g Sides::Top | Sides::Bottom

        if ctx.local_size.width == 0 || ctx.local_size.height == 0 {
            return;
        }

        let mut painter = BorderPainter::new(&self.edges, border_size, ctx.local_size);
        let mut paint = |pos, glyph| {
            ctx.place_glyph(glyph, pos);
        };

        painter.paint(&mut paint);
    }

    fn inner_bounds(&self, mut pos: Pos, mut size: Size) -> Rect {
        let bs = self.border_size(self.sides);
        pos.x += bs.top_left.max(bs.bottom_left).max(bs.left) as i32;
        pos.y += bs.top as i32;
        size.width = size
            .width
            .saturating_sub(bs.top_right.max(bs.bottom_right).max(bs.right) as usize);
        size.height = size.height.saturating_sub(bs.bottom as usize);
        Rect::from((pos, size))
    }
}

pub(crate) fn make(attributes: &Attributes<'_>) -> Box<dyn AnyWidget> {
    let border_style: BorderStyle = attributes.get_ref(BORDER_STYLE).unwrap_or_default();

    let sides = attributes
        .get_val("sides")
        .and_then(|s| Sides::try_from(s.deref()).ok())
        .unwrap_or_default();

    let text = Border {
        sides,
        edges: border_style.edges(),
        border_style,
    };
    Box::new(text)
}
//}

#[cfg(test)]
mod test {
    use crate::testing::TestRunner;

    #[test]
    fn thin_border() {
        let tpl = "border [width: 6, height: 4]";

        let expected = "
            ╔════════╗
            ║┌────┐  ║
            ║│    │  ║
            ║│    │  ║
            ║└────┘  ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn thick_border() {
        let tpl = "border [width: 6, height: 4, border_style: 'thick']";

        let expected = "
            ╔════════╗
            ║╔════╗  ║
            ║║    ║  ║
            ║║    ║  ║
            ║╚════╝  ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn custom_border() {
        let tpl = "border [width: 6, height: 4, border_style: '╔─╗│╝─╚│']";

        let expected = "
            ╔════════╗
            ║╔────╗  ║
            ║│    │  ║
            ║│    │  ║
            ║╚────╝  ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn border_top() {
        let tpl = "border [sides: 'top', width: 6, height: 4, border_style: '╔─╗│╝─╚│']";

        let expected = "
            ╔════════╗
            ║──────  ║
            ║        ║
            ║        ║
            ║        ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn border_top_bottom() {
        let tpl = "border [sides: 'bottom', width: 6, height: 4, border_style: '╔─╗│╝─╚│']";

        let expected = "
            ╔════════╗
            ║        ║
            ║        ║
            ║        ║
            ║──────  ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn border_left() {
        let tpl = "border [sides: 'left', width: 6, height: 4, border_style: '╔─╗│╝─╚│']";

        let expected = "
            ╔════════╗
            ║│       ║
            ║│       ║
            ║│       ║
            ║│       ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn border_right() {
        let tpl = "border [sides: 'right', width: 6, height: 4, border_style: '╔─╗│╝─╚│']";

        let expected = "
            ╔════════╗
            ║     │  ║
            ║     │  ║
            ║     │  ║
            ║     │  ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn border_top_left() {
        let tpl = "border [sides: ['top', 'left'], width: 6, height: 4, border_style: '╔─╗│╝─╚│']";

        let expected = "
            ╔════════╗
            ║╔─────  ║
            ║│       ║
            ║│       ║
            ║│       ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn border_bottom_right() {
        let tpl = "border [sides: ['bottom', 'right'], width: 6, height: 4]";

        let expected = "
            ╔════════╗
            ║     │  ║
            ║     │  ║
            ║     │  ║
            ║─────┘  ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn unsized_empty_border() {
        let tpl = "
            border [sides: '']
                text 'hi'
        ";

        let expected = "
            ╔════════╗
            ║hi      ║
            ║        ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 4)).instance().render_assert(expected);
    }

    #[test]
    fn sized_by_child() {
        let tpl = "
            border 
                text 'hello world'
            ";

        let expected = "
            ╔════════╗
            ║┌──────┐║
            ║│hello │║
            ║│world │║
            ║└──────┘║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }

    #[test]
    fn fixed_size() {
        let tpl = "
            border [width: 3 + 2, height: 2 + 2]
                text 'hello world'
            ";

        let expected = "
            ╔════════╗
            ║┌───┐   ║
            ║│hel│   ║
            ║│lo │   ║
            ║└───┘   ║
            ║        ║
            ║        ║
            ╚════════╝
        ";

        TestRunner::new(tpl, (8, 6)).instance().render_assert(expected);
    }
}
