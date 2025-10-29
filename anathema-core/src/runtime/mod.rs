//! TODO: write docs
use anathema::SubKey;
pub use components::Component;
pub use widgets::iter::Children;
pub use widgets::{RegisteredWidgets, Widget};

pub(crate) use self::changes::ValueIndex;
pub use crate::runtime::eval::values::TemplateValue;

mod changes;
pub mod components;
pub(crate) mod elements;
mod error;
pub(crate) mod eval;
pub(crate) mod functions;
pub(crate) mod widgets;
