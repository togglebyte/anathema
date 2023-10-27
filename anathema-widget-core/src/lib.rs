use anathema_render::Style;
use anathema_values::{Attributes, Context, NodeId};

pub mod contexts;
pub mod error;
mod factory;
pub mod generator;
pub mod layout;
mod values;
mod widget;

// #[cfg(feature = "testing")]
// pub mod testing;

pub use generator::Nodes;

pub use crate::factory::{Factory, WidgetFactory, FactoryContext};
pub use crate::layout::{Align, Axis, Direction, LocalPos, Padding, Pos, Region};
pub use crate::values::{Color, Display};
pub use crate::widget::{AnyWidget, Widget, WidgetContainer};
