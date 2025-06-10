use anathema_state::StateId;
use anathema_store::storage::strings::StringId;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{AssocEventMapping, ComponentBlueprintId};

use crate::WidgetId;
use crate::components::{AnyComponent, ComponentKind};

#[derive(Debug)]
pub struct Component<'bp> {
    pub name: &'bp str,
    pub name_id: StringId,
    pub body: &'bp [Blueprint],
    pub dyn_component: Box<dyn AnyComponent>,
    pub state_id: StateId,
    /// Used to identify the component in the component registry.
    /// This id will not be unique for prototypes
    pub component_id: ComponentBlueprintId,
    pub widget_id: WidgetId,
    pub parent: Option<WidgetId>,
    pub kind: ComponentKind,
    pub assoc_functions: &'bp [AssocEventMapping],
    pub tabindex: u16,
}

impl<'bp> Component<'bp> {
    pub fn new(
        name: &'bp str,
        name_id: StringId,
        body: &'bp [Blueprint],
        dyn_component: Box<dyn AnyComponent>,
        state_id: StateId,
        component_id: ComponentBlueprintId,
        widget_id: WidgetId,
        kind: ComponentKind,
        assoc_functions: &'bp [AssocEventMapping],
        parent: Option<WidgetId>,
    ) -> Self {
        Self {
            name,
            name_id,
            body,
            dyn_component,
            state_id,
            component_id,
            widget_id,
            kind,
            assoc_functions,
            parent,
            tabindex: 0,
        }
    }

    pub(crate) fn state_id(&self) -> StateId {
        self.state_id
    }
}
