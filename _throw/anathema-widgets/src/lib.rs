use anathema_state::Subscriber;

pub use crate::nodes::component::Component;
pub use crate::nodes::element::Layout;
pub use crate::nodes::{Element, WidgetContainer, WidgetGenerator, WidgetKind, eval_blueprint, update_widget};
pub use crate::paint::{GlyphMap, WidgetRenderer};
pub use crate::widget::{
    AnyWidget, Attributes, ComponentParents, Components, DirtyWidgets, Factory, FloatingWidgets, ForEach,
    LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Style, Widget, WidgetId, WidgetTree,
    WidgetTreeView,
};

pub type ChangeList = anathema_store::regionlist::RegionList<32, WidgetId, Subscriber>;

pub mod components;
mod container;
pub mod error;
pub mod layout;
mod nodes;
pub mod paint;
pub mod query;
pub mod tabindex;
pub mod tree;
mod widget;

#[cfg(test)]
mod testing;
