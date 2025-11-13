use std::cell::{Ref, RefCell, RefMut};
use std::ops::Index;

use anathema_compiler::blueprints::Blueprint;
use anathema_compiler::expressions::ExpressionId;
use anathema_store::key;
use anathema_store::remotecell::RemoteCell;
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::components::ComponentId;
use crate::elements::controlflow::ControlFlow;
use crate::eval::values::{Collection, TemplateValue};
use crate::value::Value;
use crate::widgets::{Node as WidgetNode, Widget, Widgets};

mod controlflow;
mod debug;
pub mod iter;

key!(ElementId, Debug, PartialEq, Copy, Clone, Eq);

impl std::hash::Hash for ElementId {
    fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u32(self.0.into())
    }
}

#[derive(Debug)]
pub struct Node<'bp> {
    pub(super) parent: Option<ElementId>,
    pub(super) children: Vec<ElementId>,
    pub(super) element: Element<'bp>,
}

pub enum Element<'bp> {
    For {
        binding: &'bp str,
        collection: Collection,
    },
    Iteration {
        loop_counter: Value<u32>,
    },
    ControlFlow,
    Condition(Option<RemoteCell<TemplateValue<'bp>>>),
    With,
    Widget(RefCell<Box<dyn Widget<'bp>>>),
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
