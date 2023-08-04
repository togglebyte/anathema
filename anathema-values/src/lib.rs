use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use anathema_render::ScreenPos;
pub use bucket::{Bucket, BucketRef, BucketMut};

pub use crate::path::{Path, PathId};
pub use crate::values::{ValueRef, List, Map, Value, Truthy, AsSlice};
pub use crate::scopes::ScopeId;
pub use crate::slab::Slab;

mod bucket;
mod generation;
mod hashmap;
mod path;
mod scopes;
mod slab;
mod notifier; 
mod values;
