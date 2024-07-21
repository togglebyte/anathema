use std::any::Any;
use std::marker::PhantomData;
use std::time::Duration;

use anathema_state::{AnyState, State};
use anathema_store::slab::Slab;
use flume::SendError;

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
    pub fn get(&mut self, id: WidgetComponentId) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn AnyState>)> {
        match self.0.get_mut(id) {
            Some(component) => match component {
                ComponentType::Component(comp, state) => Some((ComponentKind::Instance, comp.take()?, state.take()?)),
                ComponentType::Prototype(proto, state) => Some((ComponentKind::Prototype, proto(), state())),
            },
            None => panic!(),
        }
    }

    /// Return a component back to the registry.
    ///
    /// # Panics
    ///
    /// Panics if the component entry doesn't exist or if the entry is for a prototype.
    pub fn return_component(
        &mut self,
        id: WidgetComponentId,
        current_component: Box<dyn AnyComponent>,
        current_state: Box<dyn AnyState>,
    ) {
        match self.0.get_mut(id) {
            Some(component) => match component {
                ComponentType::Component(comp, state) => {
                    *comp = Some(current_component);
                    *state = Some(current_state);
                }
                ComponentType::Prototype(..) => panic!("trying to return a prototype"),
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

#[derive(Debug)]
pub struct ComponentId<T>(pub(crate) WidgetComponentId, pub(crate) PhantomData<T>);

impl<T> From<WidgetComponentId> for ComponentId<T> {
    fn from(value: WidgetComponentId) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> Clone for ComponentId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ComponentId<T> {}

pub struct ViewMessage {
    pub(super) payload: Box<dyn Any + Send + Sync>,
    pub(super) recipient: WidgetComponentId,
}

impl ViewMessage {
    pub fn recipient(&self) -> WidgetComponentId {
        self.recipient
    }

    pub fn payload(self) -> Box<dyn Any + Send + Sync> {
        self.payload
    }
}

#[derive(Debug, Clone)]
pub struct Emitter(pub(crate) flume::Sender<ViewMessage>);

impl From<flume::Sender<ViewMessage>> for Emitter {
    fn from(value: flume::Sender<ViewMessage>) -> Self {
        Self(value)
    }
}

impl Emitter {
    pub fn emit<T: 'static + Send + Sync>(
        &self,
        component_id: ComponentId<T>,
        value: T,
    ) -> Result<(), SendError<ViewMessage>> {
        let msg = ViewMessage {
            payload: Box::new(value),
            recipient: component_id.0,
        };
        self.0.send(msg)
    }

    pub async fn emit_async<T: 'static + Send + Sync>(
        &self,
        component_id: ComponentId<T>,
        value: T,
    ) -> Result<(), SendError<ViewMessage>> {
        let msg = ViewMessage {
            payload: Box::new(value),
            recipient: component_id.0,
        };
        self.0.send_async(msg).await
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Context<'rt> {
    pub emitter: &'rt Emitter,
    pub viewport: Viewport,
}

pub trait Component {
    type State: State;
    type Message;

    #[allow(unused_variables, unused_mut)]
    fn on_blur(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>) {}

    #[allow(unused_variables, unused_mut)]
    fn on_focus(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>) {}

    #[allow(unused_variables, unused_mut)]
    fn on_key(&mut self, key: KeyEvent, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn tick(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>, dt: Duration) {}

    #[allow(unused_variables, unused_mut)]
    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        mut elements: Elements<'_, '_>,
        context: Context<'_>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn resize(&mut self, state: &mut Self::State, mut elements: Elements<'_, '_>, context: Context<'_>) {}

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

#[derive(Debug)]
pub enum ComponentKind {
    Instance,
    Prototype,
}

pub trait AnyComponent {
    fn any_event(
        &mut self,
        ev: Event,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        context: Context<'_>,
    ) -> Event;

    fn any_message(
        &mut self,
        message: Box<dyn Any>,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        context: Context<'_>,
    );

    fn any_tick(
        &mut self,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        context: Context<'_>,
        dt: Duration,
    );

    fn any_focus(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, context: Context<'_>);

    fn any_blur(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, context: Context<'_>);

    fn any_resize(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, context: Context<'_>);

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
        context: Context<'_>,
    ) -> Event {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        match event {
            Event::Blur | Event::Focus => (), // Application focus, not component focus.

            Event::Key(ev) => self.on_key(ev, state, widgets, context),
            Event::Mouse(ev) => self.on_mouse(ev, state, widgets, context),

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
        context: Context<'_>,
    ) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        let Ok(message) = message.downcast::<T::Message>() else { return };
        self.message(*message, state, elements, context);
    }

    fn any_focus(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, context: Context<'_>) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.on_focus(state, elements, context);
    }

    fn any_blur(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, context: Context<'_>) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.on_blur(state, elements, context);
    }

    fn any_tick(
        &mut self,
        state: Option<&mut dyn AnyState>,
        elements: Elements<'_, '_>,
        context: Context<'_>,
        dt: Duration,
    ) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.tick(state, elements, context, dt);
    }

    fn any_resize(&mut self, state: Option<&mut dyn AnyState>, elements: Elements<'_, '_>, context: Context<'_>) {
        let state = state
            .and_then(|s| s.to_any_mut().downcast_mut::<T::State>())
            .expect("components always have a state");
        self.resize(state, elements, context);
    }
}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<dyn AnyComponent>")
    }
}
