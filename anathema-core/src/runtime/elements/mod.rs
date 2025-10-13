use std::cell::{Ref, RefCell, RefMut};
use std::ops::Index;

use anathema_store::key;
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::runtime::widgets::{Node as WidgetNode, Widget, Widgets};
use crate::templates::{Blueprint, ExpressionId};

pub mod iter;

key!(ElementId);

#[derive(Debug)]
pub struct Node<'bp> {
    parent: Option<ElementId>,
    pub(super) children: Vec<ElementId>,
    pub(super) element: Element<'bp>,
}

#[derive(Debug)]
pub enum Element<'bp> {
    For {
        binding: &'bp str,
        // collection: Collection<'bp>,
    },
    Widget(RefCell<Box<dyn Widget>>),
}

#[derive(Debug)]
pub struct Elements<'bp> {
    elements: GenSlab<ElementId, Node<'bp>>,
    removed_widgets: Vec<ElementId>,
}

impl<'bp> Elements<'bp> {
    pub fn empty() -> Self {
        Self {
            elements: GenSlab::empty(),
            removed_widgets: vec![],
        }
    }

    pub fn insert(&mut self, element: Element<'bp>, parent: Option<ElementId>) -> ElementId {
        let node = Node {
            parent,
            element,
            children: vec![],
        };

        let id = self.elements.insert(node);

        if let Some(parent) = parent {
            self.elements[parent].children.push(id);
        }

        id
    }

    pub fn remove(&mut self, id: ElementId) {
        let Some(node) = self.elements.remove(id) else { return };

        if let Some(parent) = node.parent {
            self.elements[parent].children.retain(|node| parent.ne(node));
        }

        if let Element::Widget(_) = node.element {
            self.removed_widgets.push(id);
        }
    }
}

impl<'bp> Index<ElementId> for Elements<'bp> {
    type Output = Node<'bp>;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.elements[index]
    }
}

fn widget_tree<'bp>(elements: &Elements<'bp>) -> Widgets {
    let mut widgets = Widgets::empty();

    fn add_child<'bp>(id: ElementId, elements: &Elements<'bp>, children: &mut Vec<ElementId>) {
        let node = &elements[id];
        match &node.element {
            Element::For { .. } => {
                for child in &node.children {
                    add_child(id, elements, children);
                }
            }
            Element::Widget(_) => children.push(id),
        }
    }

    for (id, node) in elements.elements.iter_keys() {
        match &node.element {
            Element::Widget(widget) => {
                let mut children = vec![];
                add_child(id, elements, &mut children);
                let node = WidgetNode::new(id, children);
                widgets.widgets.insert(id, node);
            }
            _ => continue,
        }
    }

    widgets
}
