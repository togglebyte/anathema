use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use anathema_state::{AnyState, StateId, Value as StateValue};
use anathema_store::slab::Slab;
use anathema_store::storage::strings::{StringId, Strings};
use anathema_templates::ComponentBlueprintId;
use anathema_value_resolver::{Attributes, ValueKind};
use deferred::DeferredComponents;
use flume::SendError;

use self::events::{Event, KeyEvent, MouseEvent};
use crate::layout::Viewport;
use crate::query::Children;
use crate::widget::Parent;

pub mod deferred;
pub mod events;

pub type ComponentFn = dyn Fn() -> Box<dyn AnyComponent>;
pub type StateFn = dyn FnMut() -> Box<dyn AnyState>;

enum ComponentType {
    Component(Option<Box<dyn AnyComponent>>, Option<Box<dyn AnyState>>),
    Prototype(Box<ComponentFn>, Box<StateFn>),
}

/// Store component factories.
/// This is how components are created.
pub struct ComponentRegistry(Slab<ComponentBlueprintId, ComponentType>);

impl ComponentRegistry {
    pub fn new() -> Self {
        Self(Slab::empty())
    }

    /// Both `add_component` and `add_prototype` are using `Slab::insert_at`.
    ///
    /// This is fine as the component ids are generated at the same time.
    pub fn add_component<S: 'static + AnyState>(
        &mut self,
        id: ComponentBlueprintId,
        component: impl Component + 'static,
        state: S,
    ) {
        let comp_type = ComponentType::Component(Some(Box::new(component)), Some(Box::new(state)));
        self.0.insert_at(id, comp_type);
    }

    pub fn add_prototype<FC, FS, C, S>(&mut self, id: ComponentBlueprintId, proto: FC, mut state: FS)
    where
        FC: 'static + Fn() -> C,
        FS: 'static + FnMut() -> S,
        C: Component + 'static,
        S: AnyState + 'static,
    {
        let comp_type =
            ComponentType::Prototype(Box::new(move || Box::new(proto())), Box::new(move || Box::new(state())));

        self.0.insert_at(id, comp_type);
    }

    /// # Panics
    ///
    /// Panics if the component isn't registered.
    /// This shouldn't happen as the statement eval should catch this.
    pub fn get(
        &mut self,
        id: ComponentBlueprintId,
    ) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn AnyState>)> {
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
        id: ComponentBlueprintId,
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

pub struct ViewMessage {
    pub(super) payload: Box<dyn Any + Send + Sync>,
    pub(super) recipient: ComponentBlueprintId,
}

impl ViewMessage {
    pub fn recipient(&self) -> ComponentBlueprintId {
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

pub struct Context<'frame, 'bp, T> {
    inner: AnyComponentContext<'frame, 'bp>,
    _p: PhantomData<T>,
}

impl<'frame, 'bp, T: 'static> Context<'frame, 'bp, T> {
    pub fn new(inner: AnyComponentContext<'frame, 'bp>) -> Self {
        Self { inner, _p: PhantomData }
    }

    /// Publish event
    ///
    /// # Panics
    ///
    /// This will panic if the shared value is exclusively borrowed
    /// at the time of the invocation.
    pub fn publish(&mut self, ident: &str) {
        // If there is no parent there is no one to emit the event to.
        let Some(parent) = self.parent else { return };

        let Some(ident_id) = self.inner.strings.lookup(ident) else { return };

        let ids = self.assoc_functions.iter().find(|(id, _)| *id == ident_id);

        let Some((_, external)) = ids else { return };

        self.inner.assoc_events.push(self.state_id, parent, *external);
    }

    /// Get a value from the component attributes
    pub fn attribute(&self, key: &str) -> Option<&ValueKind<'_>> {
        self.attributes.get(key)
    }

    /// Send a message to a given component
    pub fn emit<M: 'static + Send + Sync>(&self, recipient: ComponentId<M>, value: M) {
        self.emitter
            .emit(recipient, value)
            .expect("this will not fail unless the runtime is droped")
    }
}

impl<'frame, 'bp, T> Deref for Context<'frame, 'bp, T> {
    type Target = AnyComponentContext<'frame, 'bp>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'frame, 'bp, T> DerefMut for Context<'frame, 'bp, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct AnyComponentContext<'frame, 'bp> {
    parent: Option<Parent>,
    state_id: StateId,
    assoc_functions: &'frame [(StringId, StringId)],
    assoc_events: &'frame mut AssociatedEvents,
    pub attributes: &'frame mut Attributes<'bp>,
    state: Option<&'frame mut StateValue<Box<dyn AnyState>>>,
    pub emitter: &'frame Emitter,
    pub viewport: &'frame Viewport,
    pub strings: &'frame Strings,
    pub components: &'frame mut DeferredComponents,
}

impl<'frame, 'bp> AnyComponentContext<'frame, 'bp> {
    pub fn new(
        parent: Option<Parent>,
        state_id: StateId,
        assoc_functions: &'frame [(StringId, StringId)],
        assoc_events: &'frame mut AssociatedEvents,
        components: &'frame mut DeferredComponents,
        attributes: &'frame mut Attributes<'bp>,
        state: Option<&'frame mut StateValue<Box<dyn AnyState>>>,
        emitter: &'frame Emitter,
        viewport: &'frame Viewport,
        strings: &'frame Strings,
    ) -> Self {
        Self {
            parent,
            state_id,
            assoc_functions,
            assoc_events,
            attributes,
            components,
            state,
            emitter,
            viewport,
            strings,
        }
    }
}

pub struct AssociatedEvent {
    pub state: StateId,
    pub parent: Parent,
    pub external: StringId,
}

// The reason the component can not have access
// to the children during this event is because the parent is borrowing from the
// child's state while this is happening.
pub struct AssociatedEvents {
    inner: Vec<AssociatedEvent>,
}

impl AssociatedEvents {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    fn push(&mut self, state: StateId, parent: Parent, external: StringId) {
        self.inner.push(AssociatedEvent {
            state,
            parent,
            external,
        })
    }

    pub fn next(&mut self) -> Option<AssociatedEvent> {
        self.inner.pop()
    }
}

pub trait Component: 'static {
    type State: AnyState;
    type Message;

    const TICKS: bool = true;

    #[allow(unused_variables, unused_mut)]
    fn on_blur(
        &mut self,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_focus(
        &mut self,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn tick(
        &mut self,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
        dt: Duration,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn resize(
        &mut self,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn receive(
        &mut self,
        ident: &str,
        value: &dyn AnyState,
        state: &mut Self::State,
        mut elements: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    fn accept_focus(&self) -> bool {
        true
    }
}

impl Component for () {
    type Message = ();
    type State = ();

    const TICKS: bool = false;

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
    fn any_event(&mut self, elements: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>, ev: Event) -> Event;

    fn any_message(&mut self, elements: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>, message: Box<dyn Any>);

    fn any_tick(&mut self, elements: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>, dt: Duration);

    fn any_focus(&mut self, elements: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>);

    fn any_blur(&mut self, elements: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>);

    fn any_resize(&mut self, elements: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>);

    fn any_receive(
        &mut self,
        elements: Children<'_, '_>,
        ctx: AnyComponentContext<'_, '_>,
        name: &str,
        value: &dyn AnyState,
    );

    fn any_accept_focus(&self) -> bool;

    fn any_ticks(&self) -> bool;
}

impl<T> AnyComponent for T
where
    T: Component,
    T: 'static,
{
    fn any_event(&mut self, elements: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>, event: Event) -> Event {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        match event {
            Event::Blur | Event::Focus => (), // Application focus, not component focus.
            Event::Key(ev) => self.on_key(ev, &mut *state, elements, context),
            Event::Mouse(ev) => self.on_mouse(ev, &mut *state, elements, context),
            Event::Tick(dt) => self.tick(&mut *state, elements, context, dt),
            Event::Resize(_) => self.resize(&mut *state, elements, context),
            Event::Noop | Event::Stop => (),
        }
        event
    }

    fn any_accept_focus(&self) -> bool {
        self.accept_focus()
    }

    fn any_ticks(&self) -> bool {
        T::TICKS
    }

    fn any_message(&mut self, elements: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>, message: Box<dyn Any>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let Ok(message) = message.downcast::<T::Message>() else { return };
        let context = Context::<T::State>::new(ctx);
        self.message(*message, &mut *state, elements, context);
    }

    fn any_focus(&mut self, elements: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.on_focus(&mut *state, elements, context);
    }

    fn any_blur(&mut self, elements: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.on_blur(&mut *state, elements, context);
    }

    fn any_tick(&mut self, elements: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>, dt: Duration) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.tick(&mut *state, elements, context, dt);
    }

    fn any_resize(&mut self, elements: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.resize(&mut *state, elements, context);
    }

    fn any_receive(
        &mut self,
        elements: Children<'_, '_>,
        mut ctx: AnyComponentContext<'_, '_>,
        name: &str,
        value: &dyn AnyState,
    ) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");

        let context = Context::<T::State>::new(ctx);

        self.receive(name, value, &mut *state, elements, context);
    }
}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<dyn AnyComponent>")
    }
}
