use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use anathema_render::ScreenPos;
pub use bucket::{Bucket, BucketMut, BucketRef};

pub use crate::path::{Path, PathId};
pub use crate::scopes::ScopeId;
pub use crate::slab::Slab;
pub use crate::values::{AsSlice, List, Map, Truthy, Value, ValueRef};

mod bucket;
mod generation;
mod hashmap;
mod notifier;
mod path;
mod scopes;
mod slab;
mod values;
