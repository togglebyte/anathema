use anathema_render::ScreenPos;

pub mod contexts;
pub mod error;
mod id;
mod factory;
pub mod layout;
mod widget;
mod values;
pub mod generator;

// #[cfg(feature = "testing")]
// pub mod testing;

pub use crate::factory::{WidgetFactory, Factory};
pub use crate::layout::{Align, Axis, Direction, LocalPos, Padding, Pos, Region};
pub use crate::values::{Color, Display};
pub use crate::widget::{AnyWidget, Widget, WidgetContainer, WidgetMeta};
pub use generator::Nodes;
