use std::fmt::Debug;

use crate::state::State;

pub trait Collection: State + Debug {
    fn len(&self) -> usize;
}
