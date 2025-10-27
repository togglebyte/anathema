//! TODO: write docs
pub use components::Component;
pub use widgets::iter::Children;
pub use widgets::{RegisteredWidgets, Widget};

pub use crate::runtime::eval::values::TemplateValue;

pub mod components;
pub(crate) mod elements;
mod error;
pub(crate) mod eval;
pub(crate) mod functions;
pub(crate) mod widgets;
