use std::ops::ControlFlow;

use anathema_store::slab::Key;
use anathema_store::tree::visitor::NodeVisitor;

use crate::{WidgetContainer, WidgetKind};

pub struct DebugTree {
    level: usize,
    pub output: String,
}

impl DebugTree {
    pub fn new() -> Self {
        Self {
            level: 0,
            output: String::new(),
        }
    }

    fn write(&mut self, s: &str, key: Key) {
        let indent = " ".repeat(self.level * 4);
        self.output.push_str(&format!("{}:{} ", key.index(), key.gen()));
        self.output.push_str(&indent);
        self.output.push_str(s);
        self.output.push('\n');
    }
}

impl<'a> NodeVisitor<WidgetContainer<'a>> for DebugTree {
    fn visit(&mut self, value: &mut WidgetContainer<'_>, _: &[u16], value_id: Key) -> ControlFlow<bool> {
        match &value.kind {
            WidgetKind::Element(element) => self.write(element.ident, value_id),
            WidgetKind::For(_) => self.write("<for>", value_id),
            WidgetKind::Iteration(_) => self.write("<iter>", value_id),
            WidgetKind::ControlFlow(_) => self.write("<control flow>", value_id),
            WidgetKind::ControlFlowContainer(_) => self.write("<control flow container>", value_id),
            WidgetKind::Component(_) => self.write("<component>", value_id),
            WidgetKind::Slot => self.write("<slot>", value_id),
        }

        ControlFlow::Continue(())
    }

    fn push(&mut self) {
        self.level += 1;
    }

    fn pop(&mut self) {
        self.level -= 1;
    }
}
