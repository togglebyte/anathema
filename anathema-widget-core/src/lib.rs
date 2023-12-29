pub mod contexts;
pub mod error;
mod event;
pub mod expressions;
mod factory;
pub mod layout;
pub mod nodes;
mod style;
pub mod views;
mod widget;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

pub use anathema_render::Color;
pub use nodes::{Node, Nodes};

pub use crate::event::{Event, Events, KeyCode, KeyModifiers};
pub use crate::factory::{Factory, FactoryContext, WidgetFactory};
pub use crate::layout::{
    Align, Axis, Direction, Display, LayoutNode, LayoutNodes, LocalPos, Padding, Pos, Region,
};
pub use crate::style::WidgetStyle;
pub use crate::views::View;
pub use crate::widget::{AnyWidget, Widget, WidgetContainer};
