use std::fmt::Debug;

use crate::{NodeId, Path, ValueRef};

pub trait Collection: Debug {
    fn gets(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>>;
}
