use std::fmt::{self, Display};

use crate::error::Result;
use crate::Axis;

mod constraints;

pub(crate) mod expand;
pub(crate) mod horizontal;
pub(crate) mod many;
pub(crate) mod single;
pub(crate) mod spacers;
pub(crate) mod stacked;
pub(crate) mod text;
pub(crate) mod vertical;

use anathema_render::Size;
pub use constraints::Constraints;

use crate::contexts::LayoutCtx;
use crate::gen::generator::Generator;

pub trait Layout {
    fn layout<'widget, 'tpl, 'parent>(
        &mut self,
        ctx: &mut LayoutCtx<'widget, 'tpl, 'parent>,
        size: &mut Size,
    ) -> Result<()>;
}

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct Layouts<'ctx, 'widget, 'tpl, 'parent, T> {
    pub ctx: &'ctx mut LayoutCtx<'widget, 'tpl, 'parent>,
    pub size: Size,
    pub layout: T,
}

impl<'ctx, 'widget, 'tpl, 'parent, T: Layout> Layouts<'ctx, 'widget, 'tpl, 'parent, T> {
    pub fn new(layout: T, ctx: &'ctx mut LayoutCtx<'widget, 'tpl, 'parent>) -> Self {
        Self {
            ctx,
            layout,
            size: Size::ZERO,
        }
    }

    pub fn layout(&mut self) -> Result<&mut Self> {
        self.layout.layout(self.ctx, &mut self.size)?;
        Ok(self)
    }

    pub fn expand_horz(&mut self) -> &mut Self {
        self.size.width = self.ctx.constraints.max_width;
        self
    }

    pub fn expand_vert(&mut self) -> &mut Self {
        self.size.height = self.ctx.constraints.max_height;
        self
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.size)
    }
}

/// Represents the padding of a widget.
/// Padding is not applicable to `text:` widgets.
/// ```ignore
/// # use anathema_widgets::{Text, Border, BorderStyle, Sides, NodeId, Widget, Padding};
/// let mut border = Border::new(&BorderStyle::Thin, Sides::ALL, 8, 5)
///     .into_container(NodeId::anon());
///
/// // Set the padding to 2 on all sides
/// border.padding = Padding::new(2);
///
/// let text = Text::with_text("hi")
///     .into_container(NodeId::anon());
/// border.add_child(text);
/// ```
/// would output
/// ```text
/// ┌──────┐
/// │      │
/// │  hi  │
/// │      │
/// └──────┘
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Padding {
    /// Top padding
    pub top: usize,
    /// Right padding
    pub right: usize,
    /// Bottom padding
    pub bottom: usize,
    /// Left padding
    pub left: usize,
}

impl Padding {
    /// Zero padding
    pub const ZERO: Padding = Self::new(0);

    /// Create a new instance padding
    pub const fn new(padding: usize) -> Self {
        Self {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }

    /// Return the current padding and set the padding to zero
    pub fn take(&mut self) -> Self {
        let mut padding = Padding::ZERO;
        std::mem::swap(&mut padding, self);
        padding
    }

    /// Returns true if all sides are zero
    pub const fn no_padding(&self) -> bool {
        self.top + self.bottom + self.left + self.right == 0
    }
}

/// Aligning a widget "inflates" the parent to its maximum constraints (even if the alignment is
/// [`Align::TopLeft`])
///
/// Given a border widget with [`Constraints`] of 8 x 5 that contains an alignment widget, which in turn
/// contains the text "hi":
///
/// ```text
/// ┌──────┐
/// │      │
/// │      │
/// │    hi│
/// └──────┘
/// ```
/// The same border widget without alignment, and same constraints would output:
/// ```text
/// ┌──┐
/// │hi│
/// └──┘
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Align {
    /// Top
    Top,
    /// Top right
    TopRight,
    /// Right (centre on the vertical axis)
    Right,
    /// Bottom right
    BottomRight,
    /// Bottom (centre on the horizontal axis)
    Bottom,
    /// Bottom left
    BottomLeft,
    /// Left (centre on the vertical axis)
    Left,
    /// Top left
    TopLeft,
    /// Centre
    Centre,
}

impl Display for Align {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Top => write!(f, "top"),
            Self::TopRight => write!(f, "top-right"),
            Self::Right => write!(f, "right"),
            Self::BottomRight => write!(f, "bottom-right"),
            Self::Bottom => write!(f, "bottom"),
            Self::BottomLeft => write!(f, "bottom-left"),
            Self::Left => write!(f, "left"),
            Self::TopLeft => write!(f, "top-left"),
            Self::Centre => write!(f, "centre"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorzEdge {
    /// Position to the left
    Left(i32),
    /// Position to the right
    Right(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertEdge {
    /// Position at the top
    Top(i32),
    /// Position at the bottom
    Bottom(i32),
}
