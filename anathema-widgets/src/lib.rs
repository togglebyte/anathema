use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use anathema_render::ScreenPos;

mod attributes;
mod contexts;
pub mod error;
mod gen;
mod id;
mod layout;
mod path;
pub mod template;
mod values;
mod widget;

// Widgets
// mod canvas;

mod alignment;
mod border;
mod expand;
mod hstack;
mod lookup;
mod position;
mod spacer;
mod text;
mod viewport;
mod vstack;
mod zstack;

pub struct WidgetLookup;

// TODO: test only, or should this be available
//       under a "test" feature flag maybe
pub mod testing;

pub use anathema_render::Color;
pub use contexts::{DataCtx, PaintCtx, PositionCtx, Unsized, WithSize};
pub use id::{Id, NodeId};
pub use lookup::Lookup;
pub use values::{BorderStyle, Fragment, Number, Sides, TextAlignment, Value};
pub use widget::{AnyWidget, Widget, WidgetContainer};

pub use crate::attributes::{fields, Attribute, Attributes};
pub use crate::gen::generator::Generator;
pub use crate::gen::store::Store;
pub use crate::path::{Path, TextPath};

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

/// Axis
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    Horizontal,
    Vertical,
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

// -----------------------------------------------------------------------------
//     - Export all widgets -
// -----------------------------------------------------------------------------
pub use layout::text::Wrap;
pub use layout::{Align, Constraints, HorzEdge, Padding, VertEdge, Many};

pub use crate::alignment::Alignment;
pub use crate::border::Border;
// pub use crate::canvas::Canvas;
pub use crate::expand::Expand;
pub use crate::hstack::HStack;
pub use crate::position::Position;
pub use crate::spacer::Spacer;
pub use crate::text::{Text, TextSpan};
pub use crate::viewport::Viewport;
pub use crate::vstack::VStack;
pub use crate::zstack::ZStack;

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
