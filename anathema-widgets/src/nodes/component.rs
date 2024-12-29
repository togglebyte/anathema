use anathema_state::StateId;
use anathema_store::storage::strings::StringId;
use anathema_templates::blueprints::Blueprint;
use anathema_templates::ComponentBlueprintId;

use crate::components::{AnyComponent, ComponentKind};
use crate::WidgetId;

#[derive(Debug)]
pub struct Component<'bp> {
    pub name: &'bp str,
    pub body: &'bp [Blueprint],
    pub dyn_component: Box<dyn AnyComponent>,
    pub state_id: StateId,
    /// Used to identify the component in the component registry.
    /// This id will not be unique for prototypes
    // TODO: do we need the component id?
    pub component_id: ComponentBlueprintId,
    pub widget_id: WidgetId,
    pub parent: Option<WidgetId>,
    pub kind: ComponentKind,
    pub assoc_functions: &'bp [(StringId, StringId)],
}

impl<'bp> Component<'bp> {
    pub fn new(
        name: &'bp str,
        body: &'bp [Blueprint],
        dyn_component: Box<dyn AnyComponent>,
        state_id: StateId,
        component_id: ComponentBlueprintId,
        widget_id: WidgetId,
        kind: ComponentKind,
        assoc_functions: &'bp [(StringId, StringId)],
        parent: Option<WidgetId>,
    ) -> Self {
        Self {
            name,
            body,
            dyn_component,
            state_id,
            component_id,
            widget_id,
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
