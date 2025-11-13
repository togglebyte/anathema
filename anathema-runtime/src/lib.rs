// -----------------------------------------------------------------------------
//   - Macro requirements -
// -----------------------------------------------------------------------------
#[allow(unused_extern_crates)]
extern crate self as anathema;
use std::fmt::Display;

pub use anathema_compiler::blueprints::Blueprint;
pub use anathema_compiler::expressions::ExpressionId;
pub use anathema_compiler::{Color, FromColor};
pub use anathema_state_derive::State;
use anathema_store::slab::Key;

pub use self::states::State;
pub use crate::attributes::Attributes;
pub use crate::eval::values::TemplateValue;
pub use crate::states::{AnyList, AnyMap, TypeId};

// -----------------------------------------------------------------------------
//   - Exports for proc macro -
// -----------------------------------------------------------------------------
#[allow(unused_imports)]
pub use crate as state;
pub use crate::value::{AnonValue, Type};

mod attributes;
mod components;
mod elements;
mod error;
mod eval;
pub(crate) mod functions;
mod paint;
pub mod states;
mod testing;
pub mod value;
pub mod widgets;
