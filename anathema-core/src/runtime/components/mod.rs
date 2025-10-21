use anathema::Value;
use anathema_state::State;

pub use self::component::{AnyComponent, Component};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ComponentId(usize);

type FnComp = Box<dyn Fn() -> Box<dyn AnyComponent>>;
type FnState = Box<dyn Fn() -> Box<dyn State>>;

mod component;

enum Entry {
    Component {
        component: Box<dyn AnyComponent>,
        state: Box<dyn State>,
    },
    PrototypeInstance {
        component: Box<dyn AnyComponent>,
        state: Box<dyn State>,
    },
    Prototype(FnComp, FnState),
}

#[derive(Debug, Default)]
pub struct Components {
    temporary: Vec<(Box<dyn AnyComponent>, Value<Box<dyn State>>)>,
}

impl Components {
    pub fn empty() -> Self {
        Self { temporary: vec![] }
    }

    pub(crate) fn temporary_insert(&mut self, component: impl AnyComponent, state: impl State) -> ComponentId {
        let index = self.temporary.len();
        self.temporary.push((Box::new(component), Value::new(Box::new(state))));
        ComponentId(index)
    }

    pub(crate) fn by_blueprint_id(&self) {}

    pub(crate) fn by_component_id(&self) {}

    pub(crate) fn get_state(&self, component_id: ComponentId) -> Option<&Value<Box<dyn State>>> {
        self.temporary.get(component_id.0).map(|(_, state)| state)
    }
}
