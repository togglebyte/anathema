use std::any::Any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

use anathema_state::{State, StateId, Value as StateValue};
use anathema_store::slab::Slab;
use anathema_templates::strings::{StringId, Strings};
use anathema_templates::{AssocEventMapping, ComponentBlueprintId};
use anathema_value_resolver::{Attributes, ValueKind};
use deferred::DeferredComponents;
use flume::SendError;

use self::events::{ComponentEvent, KeyEvent, MouseEvent};
use crate::layout::Viewport;
use crate::query::Children;
use crate::widget::Parent;

pub mod deferred;
pub mod events;

pub type ComponentFn = dyn Fn() -> Box<dyn AnyComponent>;
pub type StateFn = dyn FnMut() -> Box<dyn State>;

enum ComponentType {
    Component(Option<Box<dyn AnyComponent>>, Option<Box<dyn State>>),
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
    pub fn add_component<S: State>(&mut self, id: ComponentBlueprintId, component: impl Component + 'static, state: S) {
        let comp_type = ComponentType::Component(Some(Box::new(component)), Some(Box::new(state)));
        self.0.insert_at(id, comp_type);
    }

    pub fn add_prototype<FC, FS, C, S>(&mut self, id: ComponentBlueprintId, proto: FC, mut state: FS)
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
    pub fn get(&mut self, id: ComponentBlueprintId) -> Option<(ComponentKind, Box<dyn AnyComponent>, Box<dyn State>)> {
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
        current_state: Box<dyn State>,
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
    pub fn publish<D: 'static>(&mut self, ident: &str, data: D) {
        // If there is no parent there is no one to emit the event to.
        let Some(parent) = self.parent else { return };

        let Some(ident_id) = self.inner.strings.lookup(ident) else { return };

        let ids = self.assoc_functions.iter().find(|assoc| assoc.internal == ident_id);

        let Some(assoc_event_map) = ids else { return };

        self.inner
            .assoc_events
            .push(self.state_id, parent, *assoc_event_map, self.ident_id, data);
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
    ident_id: StringId,
    state_id: StateId,
    assoc_functions: &'frame [AssocEventMapping],
    assoc_events: &'frame mut AssociatedEvents,
    pub attributes: &'frame mut Attributes<'bp>,
    state: Option<&'frame mut StateValue<Box<dyn State>>>,
    pub emitter: &'frame Emitter,
    pub viewport: &'frame Viewport,
    pub strings: &'frame Strings,
    pub components: &'frame mut DeferredComponents,
}

impl<'frame, 'bp> AnyComponentContext<'frame, 'bp> {
    pub fn new(
        parent: Option<Parent>,
        ident_id: StringId,
        state_id: StateId,
        assoc_functions: &'frame [AssocEventMapping],
        assoc_events: &'frame mut AssociatedEvents,
        components: &'frame mut DeferredComponents,
        attributes: &'frame mut Attributes<'bp>,
        state: Option<&'frame mut StateValue<Box<dyn State>>>,
        emitter: &'frame Emitter,
        viewport: &'frame Viewport,
        strings: &'frame Strings,
    ) -> Self {
        Self {
            parent,
            ident_id,
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

    pub fn parent(&self) -> Option<Parent> {
        self.parent
    }
}

pub struct AssociatedEvent {
    pub state: StateId,
    pub parent: Parent,
    pub sender: StringId,
    event_map: AssocEventMapping,
    data: Box<dyn Any>,
}

impl AssociatedEvent {
    pub fn to_event<'a>(&'a self, internal: &'a str, external: &'a str, sender: &'a str) -> Event<'a> {
        Event {
            external_ident: external,
            internal_ident: internal,
            data: &*self.data,
            sender,
            stop_propagation: false,
        }
    }

    pub fn external(&self) -> StringId {
        self.event_map.external
    }

    pub fn internal(&self) -> StringId {
        self.event_map.internal
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Event<'a> {
    pub sender: &'a str,
    pub external_ident: &'a str,
    pub internal_ident: &'a str,
    data: &'a dyn Any,
    stop_propagation: bool,
}

impl<'a> Event<'a> {
    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }

    pub fn name(&self) -> &str {
        self.external_ident
    }

    /// Cast the event payload to a specific type
    ///
    /// # Panics
    ///
    /// This will panic if the type is incorrect
    pub fn data<T: 'static>(&self) -> &'a T {
        match self.data.downcast_ref() {
            Some(data) => data,
            None => panic!("invalid type when casting event data"),
        }
    }

    /// Try to cast the event payload to a specific type
    pub fn data_checked<T: 'static>(&self) -> Option<&'a T> {
        self.data.downcast_ref()
    }

    pub fn should_stop_propagation(&self) -> bool {
        self.stop_propagation
    }
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

    fn push<T: 'static>(
        &mut self,
        state: StateId,
        parent: Parent,
        assoc_event_map: AssocEventMapping,
        sender: StringId,
        data: T,
    ) {
        self.inner.push(AssociatedEvent {
            state,
            parent,
            sender,
            event_map: assoc_event_map,
            data: Box::new(data),
        })
    }

    pub fn next(&mut self) -> Option<AssociatedEvent> {
        self.inner.pop()
    }
}

pub trait Component: 'static {
    type State: State;
    type Message;

    const TICKS: bool = true;

    #[allow(unused_variables, unused_mut)]
    fn on_blur(
        &mut self,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_focus(
        &mut self,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_key(
        &mut self,
        key: KeyEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_mouse(
        &mut self,
        mouse: MouseEvent,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_tick(
        &mut self,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        context: Context<'_, '_, Self::State>,
        dt: Duration,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_resize(
        &mut self,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
        mut context: Context<'_, '_, Self::State>,
    ) {
    }

    #[allow(unused_variables, unused_mut)]
    fn on_event(
        &mut self,
        event: &mut Event<'_>,
        state: &mut Self::State,
        mut children: Children<'_, '_>,
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
    fn any_event(
        &mut self,
        children: Children<'_, '_>,
        ctx: AnyComponentContext<'_, '_>,
        ev: ComponentEvent,
    ) -> ComponentEvent;

    fn any_message(&mut self, children: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>, message: Box<dyn Any>);

    fn any_tick(&mut self, children: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>, dt: Duration);

    fn any_focus(&mut self, children: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>);

    fn any_blur(&mut self, children: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>);

    fn any_resize(&mut self, children: Children<'_, '_>, ctx: AnyComponentContext<'_, '_>);

    fn any_component_event(
        &mut self,
        children: Children<'_, '_>,
        ctx: AnyComponentContext<'_, '_>,
        value: &mut Event<'_>,
    );

    fn any_accept_focus(&self) -> bool;

    fn any_ticks(&self) -> bool;
}

impl<T> AnyComponent for T
where
    T: Component,
    T: 'static,
{
    fn any_event(
        &mut self,
        children: Children<'_, '_>,
        mut ctx: AnyComponentContext<'_, '_>,
        event: ComponentEvent,
    ) -> ComponentEvent {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        match event {
            ComponentEvent::Blur | ComponentEvent::Focus => (), // Application focus, not component focus.
            ComponentEvent::Key(ev) => self.on_key(ev, &mut *state, children, context),
            ComponentEvent::Mouse(ev) => self.on_mouse(ev, &mut *state, children, context),
            ComponentEvent::Tick(dt) => self.on_tick(&mut *state, children, context, dt),
            ComponentEvent::Resize(_) => self.on_resize(&mut *state, children, context),
            ComponentEvent::Noop | ComponentEvent::Stop => (),
        }
        event
    }

    fn any_accept_focus(&self) -> bool {
        self.accept_focus()
    }

    fn any_ticks(&self) -> bool {
        T::TICKS
    }

    fn any_message(&mut self, children: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>, message: Box<dyn Any>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let Ok(message) = message.downcast::<T::Message>() else { return };
        let context = Context::<T::State>::new(ctx);
        self.on_message(*message, &mut *state, children, context);
    }

    fn any_focus(&mut self, children: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.on_focus(&mut *state, children, context);
    }

    fn any_blur(&mut self, children: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.on_blur(&mut *state, children, context);
    }

    fn any_tick(&mut self, children: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>, dt: Duration) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.on_tick(&mut *state, children, context, dt);
    }

    fn any_resize(&mut self, children: Children<'_, '_>, mut ctx: AnyComponentContext<'_, '_>) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");
        let context = Context::<T::State>::new(ctx);
        self.on_resize(&mut *state, children, context);
    }

    fn any_component_event(
        &mut self,
        children: Children<'_, '_>,
        mut ctx: AnyComponentContext<'_, '_>,
        event: &mut Event<'_>,
    ) {
        let mut state = ctx
            .state
            .take()
            .map(|s| s.to_mut_cast::<T::State>())
            .expect("components always have a state");

        let context = Context::<T::State>::new(ctx);

        self.on_event(event, &mut *state, children, context);
    }
}

impl std::fmt::Debug for dyn AnyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<dyn AnyComponent>")
    }
}
