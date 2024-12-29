pub use scope::{DebugScope, Scope};
use values::ValueId;
pub use values::ValueIndex;

pub use crate::nodes::{
    eval_blueprint, update_widget, Element, Stringify, WidgetContainer, WidgetGenerator, WidgetKind,
};
pub use crate::paint::{GlyphMap, WidgetRenderer};
pub use crate::values::{Value, Values};
pub use crate::widget::{
    AnyWidget, AttributeStorage, Attributes, ComponentParents, Components, DirtyWidgets, Factory, FloatingWidgets,
    ForEach, LayoutChildren, LayoutForEach, PaintChildren, PositionChildren, Widget, WidgetId, WidgetTree,
    WidgetTreeView,
};

pub type ChangeList = anathema_store::regionlist::RegionList<32, WidgetId, ValueId>;

#[cfg(test)]
mod testing;

pub mod components;
mod container;
pub mod debug;
pub mod error;
pub mod expressions;
pub mod layout;
mod nodes;
pub mod paint;
pub mod query;
mod scope;
pub mod tree;
mod values;
mod widget;
