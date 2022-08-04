use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use crate::display::ScreenPos;

mod animation;
mod attributes;
mod ctx;
mod id;
mod layout;
mod value;
mod widget;

pub(crate) mod alignment;
pub(crate) mod border;
pub(crate) mod canvas;
pub(crate) mod expanded;
pub(crate) mod hstack;
pub(crate) mod position;
pub(crate) mod scrollview;
pub(crate) mod spacer;
pub(crate) mod text;
pub(crate) mod vstack;
pub(crate) mod zstack;

pub mod testing;

pub use animation::{Animation, AnimationCtx};
pub use attributes::{fields, Attribute, Attributes};
pub use ctx::{LayoutCtx, PaintCtx, PositionCtx, Unsized, WithSize};
pub use id::NodeId;
pub use value::{Easing, Fragment, Number, Path, Value};
pub use widget::{Widget, WidgetContainer};

/// Determine how a widget should be displayed and laid out
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Display {
    /// Show the widget, this is the default
    Show,
    /// Include the widget as part of the layout but don't render it
    Hide,
    /// Exclude the widget from the layout
    Exclude,
}

/// Axis or direction
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// An alias for [`Axis`].
/// See [`Axis`] for documentation
pub type Direction = Axis;

// -----------------------------------------------------------------------------
//     - Export all widgets -
// -----------------------------------------------------------------------------
pub use layout::text::Wrap;
pub use layout::{Align, Constraints, Padding};
pub use {
    alignment::Alignment,
    border::{Border, BorderStyle, Sides},
    canvas::Canvas,
    expanded::Expand,
    hstack::HStack,
    position::{HorzEdge, Position, VertEdge},
    scrollview::ScrollView,
    spacer::Spacer,
    text::{Text, TextAlignment, TextSpan},
    vstack::VStack,
    zstack::ZStack,
};

// -----------------------------------------------------------------------------
//     - Pos -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const ZERO: Self = Self::new(0, 0);

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
        Self { x: (self.x as f32 * rhs).round() as i32, y: (self.y as f32 * rhs).round() as i32 }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos {
    pub x: usize,
    pub y: usize,
}

impl LocalPos {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl Add for LocalPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        LocalPos { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl Add<Pos> for LocalPos {
    type Output = Self;

    fn add(self, rhs: Pos) -> Self::Output {
        LocalPos { x: self.x + rhs.x as usize, y: self.y + rhs.y as usize }
    }
}

impl Add<ScreenPos> for LocalPos {
    type Output = Self;

    fn add(self, rhs: ScreenPos) -> Self::Output {
        LocalPos { x: self.x + rhs.x as usize, y: self.y + rhs.y as usize }
    }
}

impl TryFrom<LocalPos> for ScreenPos {
    type Error = <u16 as TryFrom<usize>>::Error;

    fn try_from(value: LocalPos) -> Result<ScreenPos, Self::Error> {
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
    pub from: Pos,
    pub to: Pos,
}

impl Region {
    pub const ZERO: Self = Self::new(Pos::ZERO, Pos::ZERO);

    pub const fn new(from: Pos, to: Pos) -> Self {
        Self { from, to }
    }

    pub const fn intersects(&self, other: &Region) -> bool {
        if other.to.x < self.from.x || other.from.x >= self.to.x {
            return false;
        }

        if other.to.y < self.from.y || other.from.y >= self.to.y {
            return false;
        }

        true
    }

    pub const fn contains(&self, pos: Pos) -> bool {
        pos.x >= self.from.x && pos.y >= self.from.y && pos.x < self.to.x && pos.y < self.to.y
    }
}
