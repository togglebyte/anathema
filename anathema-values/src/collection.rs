use std::fmt::Debug;

use crate::state::State;
use crate::NodeId;

pub trait Collection: State + Debug {
    fn len(&self) -> usize;

    fn subscribe(&self, node_id: NodeId);
}
