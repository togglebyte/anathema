use std::cell::{Ref, RefCell, RefMut};

use anathema_store::key;
use anathema_store::slab::{GenSlab, Key, SecondaryMap};

use crate::runtime::elements::{Element, ElementId};
use crate::templates::{Blueprint, ExpressionId};

key!(StatementId);

#[derive(Debug)]
pub struct Node<'bp> {
    parent: Option<StatementId>,
    children: Vec<StatementId>,
    statement: Statement<'bp>,
}

#[derive(Debug)]
pub enum Statement<'bp> {
    For {
        binding: &'bp str,
        // collection: Collection<'bp>,
    },
    Element(RefCell<Box<dyn Element>>),
}

#[derive(Debug)]
pub struct Statements<'bp> {
    statements: GenSlab<StatementId, Node<'bp>>,
    removed_elements: Vec<StatementId>,
}

impl<'bp> Statements<'bp> {
    pub fn empty() -> Self {
        Self {
            statements: GenSlab::empty(),
            removed_elements: vec![],
        }
    }

    pub fn insert(&mut self, statement: Statement<'bp>, parent: Option<StatementId>) -> StatementId {
        let node = Node {
            parent,
            statement,
            children: vec![],
        };

        let id = self.statements.insert(node);

        if let Some(parent) = parent {
            self.statements[parent].children.push(id);
        }

        id
    }

    pub fn remove(&mut self, id: StatementId) {
        let Some(node) = self.statements.remove(id) else { return };

        if let Some(parent) = node.parent {
            self.statements[parent].children.retain(|node| parent.ne(node));
        }

        if let Statement::Element(_) = node.statement {
            self.removed_elements.push(id);
        }
    }
}
