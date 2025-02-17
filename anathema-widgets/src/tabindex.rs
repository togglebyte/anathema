use std::ops::ControlFlow;

use anathema_store::tree::visitor::NodeVisitor;

use crate::{WidgetContainer, WidgetId};

pub enum Direction {
    F,
    B,
}

pub struct NextIndex {
    dir: Direction,
    current: Option<Box<[u16]>>,
}

impl<'bp> NodeVisitor<WidgetContainer<'bp>> for NextIndex {
    fn visit(&mut self, container: &mut WidgetContainer<'bp>, path: &[u16], value_id: WidgetId) -> ControlFlow<bool> {
        match &container.kind {
            crate::WidgetKind::Component(component) => {
                // let index = component.
            }
            _ => ControlFlow::Continue(())
        }

    }
}
