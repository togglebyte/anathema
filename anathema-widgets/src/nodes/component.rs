use anathema_state::StateId;
use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::WidgetComponentId;

use crate::components::{AnyComponent, ComponentAttributes, ComponentKind};
use crate::expressions::EvalValue;
use crate::{Value, ValueIndex, WidgetId};

#[derive(Debug)]
pub struct Component<'bp> {
    pub body: &'bp [Blueprint],
    pub dyn_component: Box<dyn AnyComponent>,
    pub state_id: StateId,
    pub component_id: WidgetComponentId,
    pub parent: Option<WidgetComponentId>,
    pub kind: ComponentKind,
    pub assoc_functions: &'bp [(StringId, StringId)],
}

impl<'bp> Component<'bp> {
    pub fn new(
        body: &'bp [Blueprint],
        dyn_component: Box<dyn AnyComponent>,
        state_id: StateId,
        component_id: WidgetComponentId,
        kind: ComponentKind,
        assoc_functions: &'bp [(StringId, StringId)],
        parent: Option<WidgetComponentId>,
    ) -> Self {
        Self {
            body,
            dyn_component,
            state_id,
            component_id,
            kind,
            assoc_functions,
            parent,
        }
    }

    pub(crate) fn state_id(&self) -> StateId {
        self.state_id
    }

    pub fn lookup_assoc_function(&self, internal: StringId) -> Option<StringId> {
        self.assoc_functions
            .iter()
            .find_map(|(int, ext)| match *int == internal {
                true => Some(*ext),
                false => None,
            })
    }
}
