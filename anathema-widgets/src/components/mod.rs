use std::any::Any;
use std::time::Duration;

use anathema_state::{AnyState, State};
use anathema_store::slab::Slab;

use self::events::{Event, KeyEvent, MouseEvent};
use crate::layout::Viewport;
use crate::Elements;

pub mod events;

pub const ROOT_VIEW: WidgetComponentId = WidgetComponentId(usize::MAX);

pub type ComponentFn = dyn Fn() -> Box<dyn AnyComponent>;
pub type StateFn = dyn FnMut() -> Box<dyn AnyState>;

enum ComponentType {
    Component(Option<Box<dyn AnyComponent>>, Option<Box<dyn AnyState>>),
    Prototype(Box<ComponentFn>, Box<StateFn>),
}

pub struct ComponentRegistry(Slab<WidgetComponentId, ComponentType>);

impl ComponentRegistry {
    pub fn new() -> Self {
        Self(Slab::empty())
    }

    /// Both `add_component` and `add_prototype` are using `Slab::insert_at`.
    ///
    /// This is fine as the component ids are generated at the same time.
    pub fn add_component<S: 'static + State>(
        &mut self,
        id: WidgetComponentId,
        component: impl Component + 'static,
        state: S,
    ) {
        let comp_type = ComponentType::Component(Some(Box::new(component)), Some(Box::new(state)));
        self.0.insert_at(id, comp_type);
    }

    pub fn add_prototype<FC, FS, C, S>(&mut self, id: WidgetComponentId, proto: FC, mut state: FS)
    where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> S,
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
    pub fn get(&mut self, id: WidgetComponentId) -> (Option<Box<dyn AnyComponent>>, Option<Box<dyn AnyState>>) {
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
pub struct WidgetComponentId(usize);

impl From<usize> for WidgetComponentId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<WidgetComponentId> for usize {
    fn from(value: WidgetComponentId) -> Self {
        value.0
    }
}

pub trait Component {
    type State: State;
    type Message;

    #[allow(unused_variables, unused_mut)]
    fn on_blur(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, viewport: Viewport) {}

    #[allow(unused_variables, unused_mut)]
    fn on_focus(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, viewport: Viewport) {}

    #[allow(unused_variables, unused_mut)]
    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State, mut elements: Elements<'_, '_>, viewport: Viewport) {}

    #[allow(unused_variables, unused_mut)]
    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        viewport: Viewport,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn tick(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, viewport: Viewport, dt: Duration) {}

    #[allow(unused_variables, unused_mut)]
    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        viewport: Viewport,
    ) {
    }

    fn accept_focus(&self) -> bool {
        true
    }
}

impl Component for () {
    type Message = ();
    type State = ();

    fn accept_focus(&self) -> bool {
        false
    }
}

pub trait AnyComponent {
    fn any_event(
        &mut self,
        ev: Event,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        viewport: Viewport,
    ) -> Event;

    fn any_message(
        &mut self,
        message: Box<dyn Any>,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        viewport: Viewport,
    );

    fn any_tick(
        &mut self,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        viewport: Viewport,
        dt: Duration,
    );

    fn any_focus(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, viewport: Viewport);

    fn any_blur(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, viewport: Viewport);

    fn accept_focus_any(&self) -> bool;
}

impl<T> AnyComponent for T
where
    T: Component,
    T: 'static,
{
    fn any_event(
        &mut self,
        event: Event,
        state: Option<&mut dyn AnyState>,
        widgets: Elements<'_, '_>,
        viewport: Viewport,
    ) -> Event {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        match event {
            Event::Blur | Event::Focus => (), // Application focus, not component focus.

            Event::Key(ev) => self.on_key(ev, state, widgets, viewport),
            Event::Mouse(ev) => self.on_mouse(ev, state, widgets, viewport),

            Event::Resize(_, _) | Event::Noop | Event::Stop => (),
        }
        event
    }

    fn accept_focus_any(&self) -> bool {
        self.accept_focus()
    }

    fn any_message(
        &mut self,
        message: Box<dyn Any>,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        viewport: Viewport,
    ) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        let Ok(message) = message.downcast::<T::Message>() else { return };
        self.message(*message, state, elements, viewport);
    }

    fn any_focus(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, viewport: Viewport) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.on_focus(state, elements, viewport);
    }

    fn any_blur(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, viewport: Viewport) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.on_blur(state, elements, viewport);
    }

    fn any_tick(
        &mut self,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        viewport: Viewport,
        dt: Duration,
    ) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.tick(state, elements, viewport, dt);
    }
}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<dyn AnyComponent>")
    }
}
