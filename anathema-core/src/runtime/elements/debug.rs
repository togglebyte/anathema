use std::fmt::{Debug, Formatter};

use super::Elements;
use crate::runtime::elements::{Element, ElementId, Node};

impl Debug for Elements<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        self.fmt_node(f, self.root, 0)
    }
}

impl Elements<'_> {
    fn fmt_node(&self, f: &mut Formatter<'_>, id: ElementId, depth: usize) -> std::fmt::Result {
        let node = match self.elements.get(id) {
            None => return Ok(()),
            Some(node) => node,
        };

        write!(f, "{:?} | ", anathema_store::slab::Key::from(id))?;
        for _ in 0..depth {
            write!(f, "    ")?;
        }

        writeln!(f, "{:?}", node.element)?;

        for id in &node.children {
            self.fmt_node(f, *id, depth + 1)?;
        }

        Ok(())
    }
}

impl Debug for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::For { binding, collection } => write!(f, "<for {binding}>"),
            Element::Iteration { loop_counter } => write!(f, "<iter {loop_counter}>"),
            Element::Widget(widget) => write!(f, "{:?}", widget.borrow()),
            Element::Component(component_id) => write!(f, "<component {component_id:?}>"),
        }
    }
}
