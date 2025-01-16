// NOTE:
//
// Attributes should be combined with &HStrings so actual strings can be fetched
// This makes it possible to get string slices as well as other values
//
// Values needs to be owned by the time they get to the widget
//
// Values need to contain lists for things like border sides, so it's okay to evaluate an entire
// collection for that
//
// Values do not need to be copy, since they can contain lists
//
// DynValues are required or expression resolution needs to happen for each change,
// which would be inefficient
//
// == Values for widgets ==
// Widgets needs to be able to get values for its properties, like word wrapping etc.
// This needs to come from strings.
//
// == Values for templates ==
// == Values for state ==
//
// Subscribe to values
// * All final values in the evaluation chain should be subscribed to
// * Collections should be subscribed to when they are part of the value (in case a value is
//   removed from the collection this would shift the selected value)
// * Maps don't have to have the map itself subscribed to
// * If the key is a dynamic value then that has to be subscribed to as well, which
//   should happen normally

// TODO: make everything private and enable as needed

use anathema_strings::HStrings;
use anathema_templates::Expression;
use immediate::ImmediateResolver;

pub use crate::scope::Scope;

pub mod collection;
pub mod context;
pub mod expression;
pub mod immediate;
pub mod null;
pub mod scope;
pub mod value;

#[cfg(test)]
pub(crate) mod testing;

pub trait Resolver<'bp> {
    type Output;

    fn resolve(&self, expr: &'bp Expression) -> Self::Output;
}
