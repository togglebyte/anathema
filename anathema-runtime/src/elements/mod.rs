use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Index, IndexMut};

use anathema_compiler::blueprints::Blueprint;
use anathema_compiler::expressions::ExpressionId;
use anathema_store::gen_key;
use anathema_store::remotecell::RemoteCell;
use anathema_store::slab::{Generational, Slab};

use crate::components::InternalComponentId;
use crate::elements::controlflow::ControlFlow;
use crate::eval::blueprints::values::{Collection, TemplateValue};
use crate::value::Value;
use crate::widgets::{Root, Widget};

mod controlflow;
mod debug;

gen_key!(pub ElementId);

impl std::hash::Hash for ElementId {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.0.as_raw())
    }
}

fn root_node<'bp>() -> Node<'bp> {
    Node {
        parent: None,
        children: vec![],
        element: Element::Widget(RefCell::new(Box::new(Root))),
    }
}

#[derive(Debug)]
pub struct Node<'bp> {
    pub(super) parent: Option<ElementId>,
    pub(super) children: Vec<ElementId>,
    pub(super) element: Element<'bp>,
}

pub(crate) enum Element<'bp> {
    For { binding: &'bp str, collection: Collection },
    Iteration { loop_counter: Value<u32> },
    ControlFlow,
    Condition(Option<RemoteCell<TemplateValue<'bp>>>),
    With,
    Widget(RefCell<Box<dyn Widget>>),
    Component(InternalComponentId),
}

pub struct Elements<'bp> {
    pub(crate) root: ElementId,
    pub(crate) elements: Generational<ElementId, Node<'bp>>,
    removed_widgets: Vec<ElementId>,
}

impl<'bp> Elements<'bp> {
    pub fn empty() -> Self {
        let elements = Generational::empty();
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
        // NOTE: if this panics we probably need to add `try_remove` back in
        let node = self.elements.remove(id);

        if let Some(parent) = node.parent {
            self.elements[parent].children.retain(|node| parent.ne(node));
        }

        if let Element::Widget(_) = node.element {
            self.removed_widgets.push(id);
        }
    }

    pub(crate) fn root(&self) -> &Node<'bp> {
        &self.elements[self.root]
    }

    pub(crate) fn insert_root(&mut self) -> ElementId {
        self.root = self.elements.insert(root_node());
        self.root
    }
}

impl<'bp> Index<ElementId> for Elements<'bp> {
    type Output = Node<'bp>;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.elements[index]
    }
}

impl<'bp> IndexMut<ElementId> for Elements<'bp> {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        &mut self.elements[index]
    }
}
