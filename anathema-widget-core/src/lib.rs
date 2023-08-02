use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use anathema_render::ScreenPos;

mod attributes;
pub mod contexts;
pub mod error;
// mod gen;
mod id;
// mod gen2;
pub mod layout;
// mod lookup;
pub mod template;
pub mod views;
mod widget;
// mod path;
mod values;
mod fragment;

// #[cfg(feature = "testing")]
// pub mod testing;

pub use id::{Id, NodeId};
// pub use lookup::{Factory, WidgetFactory};
pub use widget::{AnyWidget, Widget, WidgetContainer, WidgetMeta};
pub use crate::values::{Display, Number, Value, Color};
pub use crate::fragment::{Fragment, TextPath};

pub use crate::attributes::{Attribute, Attributes};
pub use crate::layout::{Padding, Axis, Direction, Align, Pos, LocalPos, Region};

pub type Nodes = anathema_generator::Nodes<WidgetContainer>;
