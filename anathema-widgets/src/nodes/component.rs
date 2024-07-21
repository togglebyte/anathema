use std::collections::HashMap;

use anathema_state::StateId;
use anathema_templates::blueprints::Blueprint;

use crate::components::{AnyComponent, ComponentKind, WidgetComponentId};
use crate::expressions::EvalValue;
use crate::{Value, ValueIndex};

type ExternalState<'bp> = HashMap<(&'bp str, ValueIndex), Value<'bp, EvalValue<'bp>>>;

#[derive(Debug)]
pub struct Component<'bp> {
    pub body: &'bp [Blueprint],
    pub component: Box<dyn AnyComponent>,
    pub state_id: StateId,
    pub(crate) external_state: Option<ExternalState<'bp>>,
    pub component_id: WidgetComponentId,
    pub kind: ComponentKind,
}

impl<'bp> Component<'bp> {
    pub fn new(
        body: &'bp [Blueprint],
        component: Box<dyn AnyComponent>,
        state_id: StateId,
        external_state: Option<ExternalState<'bp>>,
        component_id: WidgetComponentId,
        kind: ComponentKind,
    ) -> Self {
        Self {
            body,
            component,
            state_id,
            external_state,
            component_id,
            kind,
        }
    }

    pub(crate) fn state_id(&self) -> StateId {
        self.state_id
    }
}
