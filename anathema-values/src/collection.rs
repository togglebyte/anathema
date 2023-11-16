use std::fmt::Debug;

use crate::{NodeId, Path, ValueRef, State};

pub trait Collection: State + Debug {
    fn len(&self) -> usize;
}
