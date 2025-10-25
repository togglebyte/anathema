use std::cell::{Ref, RefCell, RefMut};
use std::ops::Index;

use anathema_store::key;
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::runtime::components::ComponentId;
use crate::runtime::widgets::{Node as WidgetNode, Widget, Widgets};
use crate::templates::{Blueprint, ExpressionId};

pub mod iter;
mod debug;

key!(ElementId, Debug, PartialEq, Copy, Clone, Eq);

impl std::hash::Hash for ElementId {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.0.into())
    }
}

impl nohash_hasher::IsEnabled for ElementId {}

#[derive(Debug)]
pub struct Node<'bp> {
    pub(super) parent: Option<ElementId>,
    pub(super) children: Vec<ElementId>,
    pub(super) element: Element<'bp>,
}

pub enum Element<'bp> {
    For {
        binding: &'bp str,
        // collection: Collection<'bp>,
    },
    Widget(RefCell<Box<dyn Widget>>),
    Component(ComponentId),
}

pub struct Elements<'bp> {
    root: ElementId,
    pub(crate) elements: GenSlab<ElementId, Node<'bp>>,
    removed_widgets: Vec<ElementId>,
}

impl<'bp> Elements<'bp> {
    pub fn empty() -> Self {
        let elements = GenSlab::empty();
        Self {
            root: elements.next_id(),
            elements,
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

// fn widget_tree<'bp>(elements: &Elements<'bp>) -> Widgets {
//     let mut widgets = Widgets::empty();

//     fn add_child<'bp>(id: ElementId, elements: &Elements<'bp>, children: &mut Vec<ElementId>) {
//         let node = &elements[id];
//         match &node.element {
//             Element::For { .. } | Element::Component(_) => {
//                 for child in &node.children {
//                     add_child(id, elements, children);
//                 }
//             }
//             Element::Widget(_) => children.push(id),
//         }
//     }

//     for (id, node) in elements.elements.iter_keys() {
//         match &node.element {
//             Element::Widget(widget) => {
//                 let mut children = vec![];
//                 add_child(id, elements, &mut children);
//                 let node = WidgetNode::new(id, children);
//                 widgets.widgets.insert(id, node);
//             }
//             _ => continue,
//         }
//     }

//     widgets
// }
