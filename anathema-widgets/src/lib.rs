pub use scope::{DebugScope, Scope};
pub use values::ValueIndex;

pub use crate::nodes::eval::EvalContext;
pub use crate::nodes::{eval_blueprint, try_resolve_future_values, update_tree, Element, Stringify, WidgetKind};
pub use crate::values::{Value, Values};
pub use crate::widget::{
    AnyWidget, AttributeStorage, Attributes, Elements, Factory, FloatingWidgets, LayoutChildren, PaintChildren,
    PositionChildren, Query, Widget, WidgetFactory, WidgetId, WidgetRenderer, WidgetTree,
};

pub mod components;
mod container;
pub mod debug;
pub mod error;
pub mod expressions;
pub mod layout;
mod nodes;
pub mod paint;
mod scope;
#[cfg(test)]
mod testing;
mod values;
mod widget;

// Drag'n'drop
// It won't make sense to have built in drag'n'drop
// given that widgets can be created as a result of a for-loop
// Removing the widget from the for-loop makes no sense.
//
// It would make more sense to have an event that enables copy / move of data
