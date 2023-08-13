use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

pub use store::{Store, StoreMut, StoreRef, ReadOnly};

pub use crate::notifier::{Listen, Listeners};
pub use crate::path::{Path, PathId};
pub use crate::scopes::ScopeId;
pub use crate::slab::Slab;
pub use crate::values::{AsSlice, Truthy, Container, ValueRef};

mod store;
mod generation;
pub mod hashmap;
mod notifier;
mod path;
mod scopes;
mod slab;
mod values;
