use std::ops::ControlFlow;

use anathema_state::StateId;
use anathema_store::stack::Stack;
use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::NodePath;
use anathema_widgets::components::ComponentId;
use anathema_widgets::{WidgetId, WidgetKind};

pub struct IndexEntry {
    pub(super) widget_id: WidgetId,
    pub(super) state_id: Option<StateId>,
    pub(super) component_id: ComponentId,
}

pub struct TabIndex {
    components: Stack<IndexEntry>,
    current: usize,
}

impl TabIndex {
    pub fn new() -> Self {
        Self {
            components: Stack::empty(),
            current: 0,
        }
    }

    pub fn next(&mut self) -> Option<&IndexEntry> {
        if self.components.is_empty() {
            return None;
        }

        let prev = self.components.get(self.current);
        self.current += 1;
        if self.current == self.components.len() {
            self.current = 0;
        }
        prev
    }

    pub fn prev(&mut self) -> Option<&IndexEntry> {
        if self.components.is_empty() {
            return None;
        }

        let prev = self.components.get(self.current);
        if self.current == 0 {
            self.current = self.components.len();
        }
        self.current -= 1;

        prev
    }

    pub fn current(&mut self) -> Option<&IndexEntry> {
        self.components.get(self.current)
    }

    pub fn dumb_fetch(&self, component_id: ComponentId) -> Option<&IndexEntry> {
        self.components.iter().find(|entry| entry.component_id == component_id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &IndexEntry> {
        self.components.iter()
    }
}

impl NodeVisitor<WidgetKind<'_>> for TabIndex {
    fn visit(&mut self, value: &mut WidgetKind<'_>, _path: &NodePath, widget_id: WidgetId) -> ControlFlow<()> {
        if let WidgetKind::Component(component) = value {
            self.components.push(IndexEntry {
                widget_id,
                state_id: component.state_id,
                component_id: component.component_id,
            })
        }

        ControlFlow::Continue(())
    }
}
