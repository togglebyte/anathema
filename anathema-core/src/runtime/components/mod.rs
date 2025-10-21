use anathema_state::State;

pub use self::component::{AnyComponent, Component};

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
}

impl Components {
    pub fn empty() -> Self {
        Self {
        }
    }

    pub(crate) fn by_blueprint_id(&self) {}

    pub(crate) fn by_component_id(&self) {}
}
