use std::collections::HashMap;

use anathema_state::StateId;
use anathema_templates::blueprints::Blueprint;

use crate::components::{AnyComponent, WidgetComponentId};
use crate::expressions::EvalValue;
use crate::{Value, ValueIndex};

type ExternalState<'bp> = HashMap<(&'bp str, ValueIndex), Value<'bp, EvalValue<'bp>>>;

#[derive(Debug)]
pub struct Component<'bp> {
    pub body: &'bp [Blueprint],
    pub component: Box<dyn AnyComponent>,
    pub state_id: Option<StateId>,
    pub(crate) external_state: Option<ExternalState<'bp>>,
    pub component_id: WidgetComponentId,
}

impl<'bp> Component<'bp> {
    pub fn new(
        body: &'bp [Blueprint],
        component: Box<dyn AnyComponent>,
        state_id: Option<StateId>,
        external_state: Option<ExternalState<'bp>>,
        component_id: WidgetComponentId,
    ) -> Self {
        Self {
            body,
            component,
            state_id,
            external_state,
            component_id,
        }
    }

    pub(crate) fn state_id(&self) -> Option<StateId> {
        self.state_id
    }
}
