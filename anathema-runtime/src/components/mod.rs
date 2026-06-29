//! Runtime user defined component registry
use std::marker::PhantomData;

use anathema_compiler::ComponentBlueprintId;
use anathema_store::gen_key;
use anathema_store::secondary_map::{BasicStorage, SecondaryMap};
use anathema_store::slab::{Generational, Key, Slab};

pub use self::component::Component;
use crate::components::component::AnyComponent;
use crate::states::State;
use crate::value::{Value, ValueIndex};


#[derive(Debug)]
pub struct ComponentId<T>(pub(crate) ComponentBlueprintId, pub(crate) PhantomData<T>);

impl<T> From<ComponentBlueprintId> for ComponentId<T> {
    fn from(value: ComponentBlueprintId) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> Clone for ComponentId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ComponentId<T> {}

gen_key!(pub InternalComponentId);

pub(crate) type FnComp = Box<dyn Fn() -> Box<dyn AnyComponent>>;
pub(crate) type FnState = Box<dyn Fn() -> Box<dyn State>>;

mod component;

#[derive(Debug)]
struct Entry {
    component: Box<dyn AnyComponent>,
    state: Value<Box<dyn State>>,
    kind: ComponentKind,
}

#[derive(Debug)]
enum ComponentKind {
    Component,
    PrototypeInstance,
}

enum Lookup {
    Prototype(FnComp, FnState),
    Component(InternalComponentId),
}

/// Runtime component storage
pub struct Components {
    instances: Generational<InternalComponentId, Entry>,
    blueprints: SecondaryMap<BasicStorage<ComponentBlueprintId, Lookup>>,
}

impl Components {
    pub fn empty() -> Self {
        Self {
            instances: Generational::empty(),
            blueprints: SecondaryMap::empty(),
        }
    }

    pub fn insert_component(
        &mut self,
        blueprint_id: ComponentBlueprintId,
        component: impl AnyComponent,
        state: impl State,
    ) -> InternalComponentId {
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

    pub(crate) fn by_blueprint_id(&mut self, id: ComponentBlueprintId) -> InternalComponentId {
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

    pub(crate) fn get_state(&self, component_id: InternalComponentId) -> Option<&Value<Box<dyn State>>> {
        let inst = self.instances.get(component_id)?;
        Some(&inst.state)
    }

    pub(crate) fn get_state_mut(&mut self, component_id: InternalComponentId) -> Option<&mut Value<Box<dyn State>>> {
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
