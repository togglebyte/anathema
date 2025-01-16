use anathema_state::Subscriber;

pub use crate::nodes::{
    eval_blueprint, update_widget, Element, Stringify, WidgetContainer, WidgetGenerator, WidgetKind,
};
pub use crate::paint::{GlyphMap, WidgetRenderer};
pub use crate::widget::{
    AnyWidget, ComponentParents, Components, DirtyWidgets, Factory, FloatingWidgets,
    ForEach, LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId, WidgetTree,
    WidgetTreeView,
};

pub type ChangeList = anathema_store::regionlist::RegionList<32, WidgetId, Subscriber>;

#[cfg(test)]
mod testing;

pub mod components;
mod container;
pub mod error;
pub mod layout;
mod nodes;
pub mod paint;
pub mod query;
pub mod tree;
mod widget;
