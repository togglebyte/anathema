use std::ops::ControlFlow;

use anathema_state::StateId;
use anathema_store::stack::Stack;
use anathema_store::tree::visitor::NodeVisitor;
use anathema_templates::WidgetComponentId;
use anathema_widgets::{WidgetId, WidgetKind};

#[derive(Debug, Copy, Clone)]
pub struct IndexEntry {
    pub(super) widget_id: WidgetId,
    pub(super) state_id: StateId,
    pub(super) component_id: WidgetComponentId,
}

pub struct TabIndices {
    inner: Stack<IndexEntry>,
    tabs: Vec<usize>,
    current: usize,
}

impl TabIndices {
    pub fn new() -> Self {
        Self {
            inner: Stack::empty(),
            tabs: vec![],
            current: 0,
        }
    }

    pub fn next(&mut self) -> Option<IndexEntry> {
        if self.inner.is_empty() {
            return None;
        }

        let prev = self.tabs.get(self.current)?;
        let prev = self.inner.get(*prev).copied();
        self.current += 1;
        if self.current == self.tabs.len() {
            self.current = 0;
        }
        prev
    }

    pub fn prev(&mut self) -> Option<IndexEntry> {
        if self.inner.is_empty() {
            return None;
        }

        let prev = self.tabs.get(self.current)?;
        let prev = self.inner.get(*prev).copied();
        if self.current == 0 {
            self.current = self.inner.len();
        }
        self.current -= 1;

        prev
    }

    pub fn current(&mut self) -> Option<IndexEntry> {
        let current = self.tabs.get(self.current)?;
        self.inner.get(*current).copied()
    }

    pub fn by_widget_id(&self, widget_id: WidgetId) -> Option<&IndexEntry> {
        self.inner.iter().find(|entry| entry.widget_id == widget_id)
    }

    pub fn dumb_fetch(&self, component_id: WidgetComponentId) -> Option<&IndexEntry> {
        self.inner.iter().find(|entry| entry.component_id == component_id)
    }

    /// Iterate over all component ids
    pub fn iter(&self) -> impl Iterator<Item = &IndexEntry> {
        self.inner.iter()
    }
}

impl NodeVisitor<WidgetKind<'_>> for TabIndices {
    fn visit(&mut self, value: &mut WidgetKind<'_>, _path: &[u16], widget_id: WidgetId) -> ControlFlow<bool> {
        if let WidgetKind::Component(component) = value {
            if component.dyn_component.accept_focus_any() {
                self.tabs.push(self.inner.len());
            }
            self.inner.push(IndexEntry {
                widget_id,
                state_id: component.state_id,
                component_id: component.component_id,
            })
        }

        ControlFlow::Continue(())
    }
}
