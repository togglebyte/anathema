//! Runtime user defined component registry
use anathema::State;
use anathema_store::key;
use anathema_store::slab::{GenSlab, SecondaryMap};

pub use self::component::Component;
use crate::runtime::components::component::AnyComponent;
use crate::runtime::ValueIndex;
use crate::state::Value;
use crate::templates::ComponentBlueprintId;

key!(ComponentId, Debug, Copy, Clone);

pub(crate) type FnComp = Box<dyn Fn() -> Box<dyn AnyComponent>>;
pub(crate) type FnState = Box<dyn Fn() -> Box<dyn State<ValueIndex>>>;

mod component;

struct Entry {
    component: Box<dyn AnyComponent>,
    state: Value<Box<dyn State<ValueIndex>>>,
    kind: ComponentKind,
}

enum ComponentKind {
    Component,
    PrototypeInstance,
}

enum Lookup {
    Prototype(FnComp, FnState),
    Component(ComponentId),
}

/// Runtime component storage
pub(crate) struct Components {
    instances: GenSlab<ComponentId, Entry>,
    blueprints: SecondaryMap<ComponentBlueprintId, Lookup>,
}

impl Components {
    pub fn empty() -> Self {
        Self {
            instances: GenSlab::empty(),
            blueprints: SecondaryMap::empty(),
        }
    }

    pub(crate) fn insert_component(
        &mut self,
        blueprint_id: ComponentBlueprintId,
        component: impl AnyComponent,
        state: impl State<ValueIndex>,
    ) -> ComponentId {
        let entry = Entry {
            component: Box::new(component),
            state: Value::new(Box::new(state)),
            kind: ComponentKind::Component,
        };
        let component_id = self.instances.insert(entry);
        self.blueprints.insert(blueprint_id, Lookup::Component(component_id));
        component_id
    }

    pub(crate) fn insert_prototype(&mut self, blueprint_id: ComponentBlueprintId, component: FnComp, state: FnState) {
        self.blueprints
            .insert(blueprint_id, Lookup::Prototype(component, state));
    }

    pub(crate) fn by_blueprint_id(&mut self, id: ComponentBlueprintId) -> ComponentId {
        match self.blueprints.get(id) {
            Some(Lookup::Component(id)) => *id,
            Some(Lookup::Prototype(comp, state)) => {
                let entry = Entry {
                    component: comp(),
                    state: Value::new(state()),
                    kind: ComponentKind::PrototypeInstance,
                };
                self.instances.insert(entry)
            }
            None => todo!(),
        }
    }

    pub(crate) fn get_state(&self, component_id: ComponentId) -> Option<&Value<Box<dyn State<ValueIndex>>>> {
        let inst = self.instances.get(component_id)?;
        Some(&inst.state)
    }

    pub(crate) fn get_state_mut(&mut self, component_id: ComponentId) -> Option<&mut Value<Box<dyn State<ValueIndex>>>> {
        let inst = self.instances.get_mut(component_id)?;
        Some(&mut inst.state)
    }
}

impl Default for Components {
    fn default() -> Self {
        Self::empty()
    }
}

impl std::fmt::Debug for Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<components>")
    }
}
