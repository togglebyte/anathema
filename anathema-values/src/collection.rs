use std::fmt::Debug;

use crate::state::State;

#[allow(clippy::len_without_is_empty)]
pub trait Collection: State + Debug {
    fn len(&self) -> usize;
}
