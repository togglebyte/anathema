use anathema_state::AnyState;
use anathema_templates::WidgetComponentId;

use crate::components::{AnyComponent, ComponentKind, ComponentRegistry};
use crate::{AttributeStorage, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap, WidgetId};

pub struct Sidecar<'a, 'bp> {
    pub factory: &'a Factory,
    pub dirty_widgets: &'a DirtyWidgets,
    pub attribute_storage: &'a mut AttributeStorage<'bp>,
    pub components: &'a mut Components,
    pub changelist: &'a mut ChangeList,
    pub floating_widgets: &'a mut FloatingWidgets,
    pub component_registry: &'a mut ComponentRegistry,
}

impl Sidecar<'_, '_> {
    pub fn get_component(
        &mut self,
        component_id: WidgetComponentId,
    ) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn AnyState>)> {
        self.component_registry.get(component_id)
    }

}
