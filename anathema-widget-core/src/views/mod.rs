use std::cell::RefCell;
use std::fmt::Debug;
use std::sync::OnceLock;

use anathema_values::hashmap::HashMap;
use anathema_values::{NodeId, State};
use kempt::Map;
use parking_lot::Mutex;

use crate::error::{Error, Result};
use crate::{Event, Nodes};

pub type ViewFn = dyn Fn() -> Box<dyn AnyView> + Send;

enum ViewFactory {
    View(Option<Box<dyn AnyView>>),
    Prototype(Box<ViewFn>),
}

static REGISTERED_VIEWS: OnceLock<Mutex<HashMap<usize, ViewFactory>>> = OnceLock::new();

thread_local! {
    static VIEWS: RefCell<Map<NodeId, Option<u32>>> = RefCell::new(Map::new());
}

pub struct RegisteredViews;

impl RegisteredViews {
    pub fn add_view(key: usize, view: impl AnyView + 'static) {
        Self::add(key, ViewFactory::View(Some(Box::new(view))));
    }

    pub fn add_prototype<T, F>(key: usize, f: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View + Debug + Send,
    {
        Self::add(key, ViewFactory::Prototype(Box::new(move || Box::new(f()))));
    }

    fn add(key: usize, view: ViewFactory) {
        REGISTERED_VIEWS
            .get_or_init(Default::default)
            .lock()
            .insert(key, view);
    }

    pub fn get(id: usize) -> Result<Box<dyn AnyView>> {
        let mut views = REGISTERED_VIEWS.get_or_init(Default::default).lock();
        let view = views.get_mut(&id);

        match view {
            None => Err(Error::ViewNotFound),
            // Some(f) => Ok(f()),
            Some(ViewFactory::Prototype(prototype)) => Ok(prototype()),
            Some(ViewFactory::View(view)) => match view.take() {
                Some(view) => Ok(view),
                None => Err(Error::ViewConsumed),
            },
        }
    }
}

/// NodeIds for views and their tab index
pub struct Views;

impl Views {
    /// Pass a closure that will be called with every node id that belongs
    /// to a view.
    pub fn all<F>(mut f: F) -> Option<NodeId>
    where
        F: FnMut(&mut Map<NodeId, Option<u32>>) -> Option<NodeId>,
    {
        VIEWS.with_borrow_mut(|views| f(views))
    }

    #[doc(hidden)]
    pub fn for_each<F>(mut f: F)
    where
        F: FnMut(&NodeId, Option<u32>),
    {
        VIEWS.with_borrow(|views| {
            views
                .iter()
                .map(|field| (field.key(), field.value))
                .for_each(|(key, value)| f(key, value));
        })
    }

    pub(crate) fn insert(node_id: NodeId, tabindex: Option<u32>) {
        VIEWS.with_borrow_mut(|views| views.insert(node_id, tabindex));
    }

    pub(crate) fn update(node_id: &NodeId, tabindex: Option<u32>) {
        VIEWS.with_borrow_mut(|views| {
            if let Some(old_index) = views.get_mut(node_id) {
                *old_index = tabindex;
            }
        });
    }

    #[cfg(feature = "testing")]
    #[doc(hidden)]
    pub fn test_insert(node_id: impl Into<NodeId>, tab_index: Option<u32>) {
        Self::insert(node_id.into(), tab_index)
    }

    #[cfg(feature = "testing")]
    #[doc(hidden)]
    pub fn test_clear() {
        VIEWS.with_borrow_mut(|views| views.clear());
    }
}

pub trait View {
    /// Called once a view receives an event.
    /// `nodes` represents all the nodes inside the view.
    fn on_event(&mut self, _event: Event, _nodes: &mut Nodes<'_>) {}

    /// Internal state will always take precedence over external state.
    /// It is not possible to shadow internal state.
    /// This is required to pass internal state to the templates.
    /// Without this no internal state will accessible in the templates.
    fn state(&self) -> &dyn State {
        &()
    }

    /// This function is called every frame
    fn tick(&mut self) {}

    /// This function is called once the view receives focus.
    /// This requires that the view is either the root view, which means it
    /// will receive this call exactly once,
    /// or it is a view with a tab index.
    fn focus(&mut self) {}

    /// This is called when the tab index changes and this view loses focus.
    fn blur(&mut self) {}
}

impl View for () {}

pub trait AnyView: Send {
    fn on_any_event(&mut self, ev: Event, nodes: &mut Nodes<'_>);

    fn get_any_state(&self) -> &dyn State;

    fn tick_any(&mut self);

    fn focus_any(&mut self);

    fn blur_any(&mut self);
}

impl<T> AnyView for T
where
    T: View + Send,
{
    fn on_any_event(&mut self, event: Event, nodes: &mut Nodes<'_>) {
        self.on_event(event, nodes);
    }

    fn get_any_state(&self) -> &dyn State {
        self.state()
    }

    fn tick_any(&mut self) {
        self.tick();
    }

    fn blur_any(&mut self) {
        self.blur();
    }

    fn focus_any(&mut self) {
        self.focus();
    }
}
