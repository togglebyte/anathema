use std::fmt::Debug;

use crate::{NodeId, Path, ValueRef, State};

pub trait Collection: State + Debug {
    // fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>>;

    fn len(&self) -> usize;
}
