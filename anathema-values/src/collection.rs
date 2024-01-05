use crate::state::State;
use crate::NodeId;

pub trait Collection: State {
    fn len(&self) -> usize;

    fn subscribe(&self, node_id: NodeId);
}
