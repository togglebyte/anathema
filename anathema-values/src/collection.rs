use std::fmt::Debug;

use crate::{NodeId, Path, ValueRef};
use crate::state::State;

pub trait Collection: State + Debug {
    fn len(&self) -> usize;
}
