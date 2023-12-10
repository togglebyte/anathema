use std::fmt::{self, Display as RustDisplay};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use anathema_render::{ScreenPos, Size};
use anathema_values::{
    impl_dyn_value, Context, DynValue, NodeId, Resolver, Value, ValueExpr, ValueRef, ValueResolver,
};
pub use layoutnodes::{LayoutNode, LayoutNodes};

pub use self::constraints::Constraints;
pub use self::padding::Padding;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::nodes::Nodes;

mod constraints;
mod layoutnodes;
mod padding;

pub trait Layout {
    fn layout<'nodes, 'expr, 'state>(
        &mut self,
        nodes: &mut LayoutNodes<'nodes, 'expr, 'state>,
    ) -> Result<Size>;
}

// -----------------------------------------------------------------------------
//   - Layouts -
// -----------------------------------------------------------------------------
pub struct Layouts<'ctx, T> {
    pub layout_ctx: &'ctx LayoutCtx,
    pub size: Size,
    pub layout: T,
}

impl<'ctx, T: Layout> Layouts<'ctx, T> {
    pub fn new(layout: T, layout_ctx: &'ctx LayoutCtx) -> Self {
        Self {
            layout_ctx,
            layout,
            size: Size::ZERO,
        }
    }

    pub fn layout<'e>(&mut self, children: &mut Nodes<'e>, data: &Context<'_, 'e>) -> Result<Size> {
        panic!()
        // self.layout.layout(children, self.layout_ctx, data)
    }

    pub fn expand_horz(&mut self, mut size: Size) -> Size {
        size.width = self.layout_ctx.constraints.max_width;
        size
    }

    pub fn expand_vert(&mut self, mut size: Size) -> Size {
        size.height = self.layout_ctx.constraints.max_height;
        size
    }

    pub fn expand_all(&mut self, mut size: Size) -> Size {
        size = self.expand_horz(size);
        self.expand_vert(size)
    }

    // pub fn size(&self) -> Size {
    //     self.size
    // }
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
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum Align {
    /// Top left
    #[default]
    TopLeft,
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
    /// Centre
    Centre,
}

// TODO can we remove this?
// TB: 2023-11-20
// impl TryFrom<&str> for Align {
//     type Error = ();

//     fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
//         let wrap = match value {
//             "top" => Self::Top,
//             "top-right" => Self::TopRight,
//             "right" => Self::Right,
//             "bottom-right" => Self::BottomRight,
//             "bottom" => Self::Bottom,
//             "bottom-left" => Self::BottomLeft,
//             "left" => Self::Left,
//             "top-left" => Self::Left,
//             "centre" | "center" => Self::Centre,
//             _ => Self::TopLeft,
//         };
//         Ok(wrap)
//     }
// }

impl RustDisplay for Align {
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

impl_dyn_value!(Align);

impl TryFrom<ValueRef<'_>> for Align {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> std::result::Result<Self, Self::Error> {
        let wrap = match value {
            ValueRef::Str("top") => Self::Top,
            ValueRef::Str("top-right") => Self::TopRight,
            ValueRef::Str("right") => Self::Right,
            ValueRef::Str("bottom-right") => Self::BottomRight,
            ValueRef::Str("bottom") => Self::Bottom,
            ValueRef::Str("bottom-left") => Self::BottomLeft,
            ValueRef::Str("left") => Self::Left,
            ValueRef::Str("top-left") => Self::TopLeft,
            ValueRef::Str("centre" | "center") => Self::Centre,
            _ => Self::Top,
        };
        Ok(wrap)
    }
}

/// Determine how a widget should be displayed and laid out
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Display {
    /// Show the widget, this is the default
    #[default]
    Show,
    /// Include the widget as part of the layout but don't render it
    Hide,
    /// Exclude the widget from the layout and paint step.
    Exclude,
}

impl TryFrom<ValueRef<'_>> for Display {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> std::result::Result<Self, Self::Error> {
        let wrap = match value {
            ValueRef::Str("hide") => Self::Hide,
            ValueRef::Str("exclude") => Self::Exclude,
            _ => Self::Show,
        };
        Ok(wrap)
    }
}

impl_dyn_value!(Display);

#[derive(Debug)]
pub enum HorzEdge {
    /// Position to the left
    Left(Value<i32>),
    /// Position to the right
    Right(Value<i32>),
}

#[derive(Debug)]
pub enum VertEdge {
    /// Position at the top
    Top(Value<i32>),
    /// Position at the bottom
    Bottom(Value<i32>),
}

/// Axis
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl_dyn_value!(Axis);

impl TryFrom<ValueRef<'_>> for Axis {
    type Error = ();

    fn try_from(value: ValueRef<'_>) -> std::result::Result<Self, Self::Error> {
        match value {
            ValueRef::Str("horz" | "horizontal") => Ok(Self::Horizontal),
            ValueRef::Str("vert" | "vertical") => Ok(Self::Vertical),
            _ => Err(()),
        }
    }
}

/// Direction
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

impl TryFrom<&str> for Direction {
    type Error = ();

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "forward" => Ok(Self::Forward),
            "backward" => Ok(Self::Backward),
            _ => Err(()),
        }
    }
}

// -----------------------------------------------------------------------------
//     - Pos -
// -----------------------------------------------------------------------------
/// A position in global space
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    /// X coordinate
    pub x: i32,
    /// Y coordinate
    pub y: i32,
}

impl Pos {
    /// Zero
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new instance with the given x and y coordinates
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl From<(i32, i32)> for Pos {
    fn from(val: (i32, i32)) -> Self {
        Self::new(val.0, val.1)
    }
}

impl From<(u16, u16)> for Pos {
    fn from(val: (u16, u16)) -> Self {
        Self::new(val.0 as i32, val.1 as i32)
    }
}

impl From<(usize, usize)> for Pos {
    fn from(val: (usize, usize)) -> Self {
        Self::new(val.0 as i32, val.1 as i32)
    }
}

impl Add for Pos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add<LocalPos> for Pos {
    type Output = Self;

    fn add(self, rhs: LocalPos) -> Self::Output {
        Pos::new(self.x + rhs.x as i32, self.y + rhs.y as i32)
    }
}

impl Mul<f32> for Pos {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: (self.x as f32 * rhs).round() as i32,
            y: (self.y as f32 * rhs).round() as i32,
        }
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Pos) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Pos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Pos) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

// -----------------------------------------------------------------------------
//     - Local position -
// -----------------------------------------------------------------------------
/// Positions in a local space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    /// X coordinate
    pub x: usize,
    /// Y coordinate
    pub y: usize,
}

impl LocalPos {
    /// Zero...
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new set of coordinates in local space
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Add for LocalPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        LocalPos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add<Pos> for LocalPos {
    type Output = Self;

    fn add(self, rhs: Pos) -> Self::Output {
        LocalPos {
            x: self.x + rhs.x as usize,
            y: self.y + rhs.y as usize,
        }
    }
}

impl Add<ScreenPos> for LocalPos {
    type Output = Self;

    fn add(self, rhs: ScreenPos) -> Self::Output {
        LocalPos {
            x: self.x + rhs.x as usize,
            y: self.y + rhs.y as usize,
        }
    }
}

impl TryFrom<LocalPos> for ScreenPos {
    type Error = <u16 as TryFrom<usize>>::Error;

    fn try_from(value: LocalPos) -> std::result::Result<ScreenPos, Self::Error> {
        let x: u16 = value.x.try_into()?;
        let y: u16 = value.y.try_into()?;
        Ok(ScreenPos::new(x, y))
    }
}

// -----------------------------------------------------------------------------
//     - Region -
// -----------------------------------------------------------------------------
/// A region in global space
#[derive(Debug, Clone, Copy)]
pub struct Region {
    /// The starting position of the region
    pub from: Pos,
    /// The end position of the region
    pub to: Pos,
}

impl Region {
    /// Zero...
    pub const ZERO: Self = Self::new(Pos::ZERO, Pos::ZERO);

    /// Create a new instance of a region.
    pub const fn new(from: Pos, to: Pos) -> Self {
        Self { from, to }
    }

    /// Check if another region is intersecting with this region
    pub const fn intersects(&self, other: &Region) -> bool {
        if other.to.x < self.from.x || other.from.x >= self.to.x {
            return false;
        }

        if other.to.y < self.from.y || other.from.y >= self.to.y {
            return false;
        }

        true
    }

    /// Check if a region contains a position
    pub fn contains(&self, pos: Pos) -> bool {
        pos.x >= self.from.x && pos.x <= self.to.x && pos.y >= self.from.y && pos.y <= self.to.y
    }

    /// Constrain a region to fit within another region
    pub fn constrain(&mut self, other: &Region) {
        self.from.x = self.from.x.max(other.from.x);
        self.from.y = self.from.y.max(other.from.y);
        self.to.x = self.to.x.min(other.to.x);
        self.to.y = self.to.y.min(other.to.y);
    }
}
