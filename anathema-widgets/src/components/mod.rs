use std::any::Any;

use anathema_state::State;
use anathema_store::slab::Slab;

use self::events::{Event, KeyEvent, MouseEvent};
use crate::Elements;

pub mod events;

pub const ROOT_VIEW: ComponentId = ComponentId(usize::MAX);

pub type ComponentFn = dyn Fn() -> Box<dyn AnyComponent>;
pub type StateFn = dyn Fn() -> Box<dyn State>;

enum ComponentType {
    Component(Option<Box<dyn AnyComponent>>, Option<Box<dyn State>>),
    Prototype(Box<ComponentFn>, Box<StateFn>),
}

pub struct ComponentRegistry(Slab<ComponentId, ComponentType>);

impl ComponentRegistry {
    pub fn new() -> Self {
        Self(Slab::empty())
    }

    /// Both `add_component` and `add_prototype` are using `Slab::insert_at`.
    ///
    /// This is fine as the component ids are generated at the same time.
    pub fn add_component<S: 'static + State>(
        &mut self,
        id: ComponentId,
        component: impl Component + 'static,
        state: S,
    ) {
        let comp_type = ComponentType::Component(Some(Box::new(component)), Some(Box::new(state)));
        self.0.insert_at(id, comp_type);
    }

    pub fn add_prototype<FC, FS, C, S>(&mut self, id: ComponentId, proto: FC, state: FS)
    where
        FC: 'static + Fn() -> C,
        FS: 'static + Fn() -> S,
        C: Component + 'static,
        S: State + 'static,
    {
        let comp_type =
            ComponentType::Prototype(Box::new(move || Box::new(proto())), Box::new(move || Box::new(state())));

        self.0.insert_at(id, comp_type);
    }

    /// # Panics
    ///
    /// Panics if the component isn't registered.
    /// This shouldn't happen as the statement eval should catch this.
    pub fn get(&mut self, id: ComponentId) -> (Option<Box<dyn AnyComponent>>, Option<Box<dyn State>>) {
        match self.0.get_mut(id) {
            Some(component) => match component {
                ComponentType::Component(comp, state) => (comp.take(), state.take()),
                ComponentType::Prototype(proto, state) => (Some(proto()), Some(state())),
            },
            None => panic!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(usize);

impl From<usize> for ComponentId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<ComponentId> for usize {
    fn from(value: ComponentId) -> Self {
        value.0
    }
}

pub trait Component {
    type State: State;
    type Message;

    fn on_blur(&mut self, _state: Option<&mut Self::State>) {}

    fn on_focus(&mut self, _state: Option<&mut Self::State>) {}

    fn on_key(&mut self, _key: KeyEvent, _state: Option<&mut Self::State>, _elements: Elements<'_, '_>) {}

    fn on_mouse(&mut self, _mouse: MouseEvent, _state: Option<&mut Self::State>, _elements: Elements<'_, '_>) {}

    fn message(&mut self, _message: Self::Message, _state: Option<&mut Self::State>) {}

    fn accept_focus(&self) -> bool {
        true
    }
}

impl Component for () {
    type Message = ();
    type State = ();

    fn on_blur(&mut self, _state: Option<&mut Self::State>) {}

    fn on_focus(&mut self, _state: Option<&mut Self::State>) {}

    fn on_key(&mut self, _key: KeyEvent, _state: Option<&mut Self::State>, _: Elements<'_, '_>) {}

    fn on_mouse(&mut self, _mouse: MouseEvent, _state: Option<&mut Self::State>, _widgets: Elements<'_, '_>) {}

    fn message(&mut self, _message: Self::Message, _state: Option<&mut Self::State>) {}

    fn accept_focus(&self) -> bool {
        false
    }
}

pub trait AnyComponent {
    fn any_event(&mut self, ev: Event, state: Option<&mut dyn State>, widgets: Elements<'_, '_>) -> Event;

    fn any_message(&mut self, message: Box<dyn Any>, state: Option<&mut dyn State>);

    fn any_focus(&mut self, state: Option<&mut dyn State>);

    fn any_blur(&mut self, state: Option<&mut dyn State>);

    fn accept_focus_any(&self) -> bool;
}

impl<T> AnyComponent for T
where
    T: Component,
    T: 'static,
{
    fn any_event(&mut self, event: Event, state: Option<&mut dyn State>, widgets: Elements<'_, '_>) -> Event {
        let state = state.and_then(|s| s.to_any_mut().downcast_mut::<T::State>());
        match event {
            Event::Blur => todo!(),
            Event::Focus => todo!(),

            Event::Key(ev) => self.on_key(ev, state, widgets),
            Event::Mouse(ev) => self.on_mouse(ev, state, widgets),

            Event::Resize(_, _) | Event::Noop | Event::Stop => (),
        }
        event
    }

    fn accept_focus_any(&self) -> bool {
        self.accept_focus()
    }

    fn any_message(&mut self, message: Box<dyn Any>, state: Option<&mut dyn State>) {
        let state = state.and_then(|s| s.to_any_mut().downcast_mut::<T::State>());
        let Ok(message) = message.downcast::<T::Message>() else { return };
        self.message(*message, state);
    }

    fn any_focus(&mut self, state: Option<&mut dyn State>) {
        let state = state.and_then(|s| s.to_any_mut().downcast_mut::<T::State>());
        self.on_focus(state);
    }

    fn any_blur(&mut self, state: Option<&mut dyn State>) {
        let state = state.and_then(|s| s.to_any_mut().downcast_mut::<T::State>());
        self.on_blur(state);
    }
}

// pub fn get_component(id: ComponentId) -> (Option<Box<dyn AnyComponent>>, Option<Box<dyn State>>) {
//     ComponentRegistry::get(id)
// }
