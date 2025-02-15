use anathema_strings::HStrings;
use anathema_templates::Expression;
use immediate::Resolver;

pub use crate::attributes::{AttributeStorage, Attributes, ValueKey};
pub use crate::context::ResolverCtx;
pub use crate::scope::Scope;
pub use crate::value::{resolve, resolve_collection, Collection, Value, ValueKind};

mod attributes;
mod context;
mod expression;
mod immediate;
mod scope;
mod value;

#[cfg(test)]
pub(crate) mod testing;
