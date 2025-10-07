use anathema_store::slab::Slab;

use crate::elements::Element;

pub struct NodeId(u32);

pub struct Nodes {
    nodes: Slab<NodeId, Node>
}

pub struct Node {
    parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    element: Box<dyn Element>,
}
