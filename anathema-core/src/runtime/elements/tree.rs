use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Index, IndexMut};

use anathema_geometry::Size;
use anathema_store::key;
use anathema_store::slab::{GenSlab, Key, SlabIndex};

use super::Element;
use crate::attributes::AllAttributes;
use crate::layout::Layout;

key!(ElementId);

#[derive(Debug)]
pub struct Elements {
    nodes: GenSlab<ElementId, Node>,
}

impl Elements {
    pub(crate) fn empty() -> Self {
        Self {
            nodes: GenSlab::empty(),
        }
    }

    pub(crate) fn node(&self, id: ElementId) -> NodeRef<'_> {
        let node = &self.nodes[id];

        let children = Children {
            node_ids: &node.children,
            nodes: &self.nodes,
            index: 0,
        };

        panic!()

        // NodeRef {
        //     id,
        //     element: node.element.borrow_mut(),
        //     children,
        // }
    }

    pub(crate) fn apply_insert(
        &mut self,
        insert: InsertNode,
        parent: Option<ElementId>,
        layout: &mut Layout,
        all_attributes: &mut AllAttributes,
    ) -> ElementId {
        let node = Node {
            parent,
            element: panic!(),//RefCell::new(insert.element),
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

impl Index<ElementId> for Elements {
    type Output = Node;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.nodes[index]
    }
}

impl IndexMut<ElementId> for Elements {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

pub struct Children<'a> {
    node_ids: &'a [ElementId],
    nodes: &'a GenSlab<ElementId, Node>,
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
        let element = panic!(); //node.element.borrow_mut();
        let children = Children {
            node_ids: &node.children,
            nodes: self.nodes,
            index: 0,
        };
        Some(NodeRef { id, element, children })
    }
}

pub struct NodeRef<'a> {
    id: ElementId,
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
    pub parent: Option<ElementId>,
    pub children: Vec<ElementId>,
    pub element: ElementId,
}

/// An opaque node used to setup nodes for insertion into the tree.
pub struct InsertNode {
    pub children: Vec<InsertNode>,
    pub element: Box<dyn Element>,
}

impl InsertNode {
    pub fn new(el: Box<dyn Element>) -> Self {
        Self {
            element: el,
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
        Self::new(Box::new(value))
    }
}
