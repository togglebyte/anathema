use std::convert::TryFrom;
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use anathema_render::ScreenPos;

// mod attributes;
pub mod contexts;
pub mod error;
// mod gen;
mod id;
// mod gen2;
mod factory;
pub mod layout;
// pub mod template;
pub mod views;
mod widget;
// mod path;
// mod fragment;
mod notifications;
mod values;

// #[cfg(feature = "testing")]
// pub mod testing;

// pub use id::{Id, NodeId};
// pub use crate::attributes::{Attribute, Attributes};
pub use crate::factory::{WidgetFactory, Factory};
pub use crate::layout::{Align, Axis, Direction, LocalPos, Padding, Pos, Region};
pub use crate::values::{Color, Display, Number, Value};
pub use crate::widget::{AnyWidget, Widget, WidgetContainer, WidgetMeta};

pub type Nodes = anathema_generator::Nodes<WidgetContainer>;
pub type ReadOnly<'a> = anathema_values::ReadOnly<'a, Value>;
pub type BucketRef<'a> = anathema_values::BucketRef<'a, Value>;
