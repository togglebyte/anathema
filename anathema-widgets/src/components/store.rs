use std::any::Any;
use std::cell::RefCell;

use anathema_state::State;
use anathema_store::slab::Slab;

use crate::{AnyComponent, Component, ComponentId};

thread_local! {
    // static COMPONENTS: RefCell<Slab<ComponentId, ComponentType>> = const { RefCell::new(Slab::empty()) };
}

pub type ComponentFn<T> = dyn Fn() -> Box<dyn AnyComponent<Fiddlesticks = T>>;
pub type StateFn = dyn Fn() -> Box<dyn State>;

enum ComponentType<T> {
    Component(Option<Box<dyn AnyComponent<Fiddlesticks = T>>>, Option<Box<dyn State>>),
    Prototype(Box<ComponentFn<T>>, Box<StateFn>),
}

pub struct RegisteredComponents;

impl RegisteredComponents {
    /// Both `add_component` and `add_prototype` are using `Slab::insert_at`.
    ///
    /// This is fine as the component ids are generated at the same time.
    pub fn add_component<S: 'static + State>(id: ComponentId, component: impl Component + 'static, state: S) {
        let comp_type = ComponentType::Component(Some(Box::new(component)), Some(Box::new(state)));
        // COMPONENTS.with_borrow_mut(|components| components.insert_at(id, comp_type));
    }

    pub fn add_prototype<FC, FS, C, S>(id: ComponentId, proto: FC, state: FS)
    where
        FC: 'static + Fn() -> C,
        FS: 'static + Fn() -> S,
        C: Component + 'static,
        S: State + 'static,
    {
        let comp_type =
            ComponentType::Prototype(Box::new(move || Box::new(proto())), Box::new(move || Box::new(state())));

        // COMPONENTS.with_borrow_mut(|components| components.insert_at(id, comp_type));
    }

    /// # Panics
    ///
    /// Panics if the component isn't registered.
    /// This shouldn't happen as the statement eval should catch this.
    pub fn get(id: ComponentId) -> (Option<Box<dyn AnyComponent>>, Option<Box<dyn State>>) {
        // COMPONENTS.with_borrow_mut(|components| match components.get_mut(id) {
        //     Some(component) => match component {
        //         ComponentType::Component(comp, state) => (comp.take(), state.take()),
        //         ComponentType::Prototype(proto, state) => (Some(proto()), Some(state())),
        //     },
        //     None => panic!(),
        // })
    }
}

pub fn get_component<T>(id: ComponentId) -> (Option<Box<dyn AnyComponent<Fiddlesticks = T>>>, Option<Box<dyn State>>) {
    RegisteredComponents::get(id)
}
