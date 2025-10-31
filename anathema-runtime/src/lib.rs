// -----------------------------------------------------------------------------
//   - Macro requirements -
// -----------------------------------------------------------------------------
#[allow(unused_extern_crates)]
extern crate self as anathema;
#[allow(unused_imports)]
pub use crate as state;

use std::fmt::Display;

pub use anathema_compiler::blueprints::Blueprint;
pub use anathema_compiler::expressions::ExpressionId;
pub use anathema_compiler::{Color, FromColor};
pub use anathema_state_derive::State;
use anathema_store::slab::Key;

// -----------------------------------------------------------------------------
//   - Exports for proc macro -
// -----------------------------------------------------------------------------
pub use crate::value::{AnonValue, Type};
pub use self::states::State;
pub use crate::states::{AnyList, AnyMap, TypeId};

mod attributes;
mod components;
mod elements;
mod error;
mod eval;
pub(crate) mod functions;
pub mod states;
mod testing;
pub mod value;
pub mod widgets;
