use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Index, IndexMut};

use anathema_geometry::Size;
use anathema_store::slab::{GenSlab, Key, SlabIndex};

use crate::attributes::AllAttributes;
use crate::elements::Element;
use crate::layout::Layout;

// #[derive(Debug, Copy, Clone, PartialEq)]
// pub struct NodeId(Key);
pub type NodeId = Key;

#[derive(Debug)]
pub struct Nodes {
    nodes: GenSlab<Node>,
}

impl Nodes {
    pub(crate) fn empty() -> Self {
        Self {
            nodes: GenSlab::empty(),
        }
    }

    pub(super) fn node(&self, id: NodeId) -> NodeRef<'_> {
        let node = &self.nodes[id];

        let children = Children {
            node_ids: &node.children,
            nodes: &self.nodes,
            index: 0,
        };

        NodeRef {
            id,
            element: node.element.borrow_mut(),
            children,
        }
    }

    pub(crate) fn apply_insert(
        &mut self,
        insert: InsertNode,
        parent: Option<NodeId>,
        layout: &mut Layout,
        all_attributes: &mut AllAttributes,
    ) -> NodeId {
        let node = Node {
            parent,
            element: RefCell::new(insert.element),
            children: vec![],
        };

        let id = self.nodes.insert(node);
        layout.insert(id);
        all_attributes.insert(id);

        for child in insert.children {
            let child_id = self.apply_insert(child, Some(id), layout, all_attributes);
            self.nodes[id].children.push(child_id);
        }

        id
    }
}

impl Index<NodeId> for Nodes {
    type Output = Node;

    fn index(&self, index: NodeId) -> &Self::Output {
        &self.nodes[index]
    }
}

impl IndexMut<NodeId> for Nodes {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

pub struct Children<'a> {
    node_ids: &'a [NodeId],
    nodes: &'a GenSlab<Node>,
    index: usize,
}

impl<'a> Iterator for Children<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.node_ids.len() {
            return None;
        }

        let id = self.node_ids[self.index];
        self.index += 1;
        let node = &self.nodes[id];
        let element = node.element.borrow_mut();
        let children = Children {
            node_ids: &node.children,
            nodes: self.nodes,
            index: 0,
        };
        Some(NodeRef { id, element, children })
    }
}

pub struct NodeRef<'a> {
    id: NodeId,
    element: RefMut<'a, Box<dyn Element>>,
    children: Children<'a>,
}

impl<'a> NodeRef<'a> {
    pub fn layout(mut self, layout: &mut Layout) -> Size {
        let size = self.element.layout(self.children, layout);
        layout.set_size(self.id, size);
        size
    }
}

#[derive(Debug)]
pub struct Node {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub element: RefCell<Box<dyn Element>>,
}

/// An opaque node used to setup nodes for insertion into the tree.
pub struct InsertNode {
    pub children: Vec<InsertNode>,
    pub element: Box<dyn Element>,
}

impl InsertNode {
    pub fn new(el: impl Element) -> Self {
        Self {
            element: Box::new(el),
            children: vec![],
        }
    }

    pub fn add_child(mut self, child: InsertNode) -> Self {
        self.children.push(child);
        self
    }
}

impl<T: Element> From<T> for InsertNode {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
