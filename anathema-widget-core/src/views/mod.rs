use std::any::Any;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::fmt::Debug;

use anathema_values::hashmap::HashMap;
use anathema_values::{NodeId, State};
use parking_lot::Mutex;
use kempt::Set;

use crate::{Event, Nodes};
use crate::error::{Error, Result};

pub type ViewFn = dyn Fn() -> Box<dyn AnyView> + Send;

enum ViewFactory {
    View(Option<Box<dyn AnyView>>),
    Prototype(Box<ViewFn>)
}

static TAB_INDEX: AtomicUsize = AtomicUsize::new(0);
static TAB_VIEWS: Mutex<Set<NodeId>> = Mutex::new(Set::new());
static VIEWS: Mutex<Set<NodeId>> = Mutex::new(Set::new());
static REGISTERED_VIEWS: OnceLock<Mutex<HashMap<String, ViewFactory>>> = OnceLock::new();

pub struct RegisteredViews;

impl RegisteredViews {
    pub fn add_view(key: String, view: impl AnyView + 'static) {
        Self::add(key, ViewFactory::View(Some(Box::new(view))));
    }

    pub fn add_prototype<T, F>(key: String, f: F)
    where
        F: Send + 'static + Fn() -> T,
        T: 'static + View + Debug + Send,
    {
        Self::add(key, ViewFactory::Prototype(Box::new(move || Box::new(f()))));
    }

    fn add(key: String, view: ViewFactory) {
        REGISTERED_VIEWS
            .get_or_init(Default::default)
            .lock()
            .insert(key, view);
    }


    pub fn get(id: &str) -> Result<Box<dyn AnyView>> {
        let mut views = REGISTERED_VIEWS.get_or_init(Default::default).lock();
        let view = views.get_mut(id);

        match view {
            None => Err(Error::ViewNotFound),
            // Some(f) => Ok(f()),
            Some(ViewFactory::Prototype(prototype)) => Ok(prototype()),
            Some(ViewFactory::View(view)) => {
                match view.take() {
                    Some(view) => Ok(view),
                    None => Err(Error::ViewConsumed),
                }
            }
        }
    }
}

pub struct TabIndex;

impl TabIndex {
    pub fn next() {
        TAB_INDEX.fetch_add(1, Ordering::Relaxed);

        let len = TAB_VIEWS.lock().len();

        if TAB_INDEX.load(Ordering::Relaxed) == len {
            TAB_INDEX.store(0, Ordering::Relaxed);
        }
    }

    pub fn prev() {
        TAB_INDEX.fetch_sub(1, Ordering::Relaxed);

        let len = TAB_VIEWS.lock().len();

        if TAB_INDEX.load(Ordering::Relaxed) == 0 {
            TAB_INDEX.store(len - 1, Ordering::Relaxed);
        }
    }

    pub(crate) fn insert(node_id: NodeId) {
        TAB_VIEWS
            .lock()
            .insert(node_id);
    }

    fn remove(node_id: &NodeId) {
        TAB_VIEWS
            .lock()
            .remove(node_id);
    }

    pub(crate) fn remove_all<'a>(node_ids: impl Iterator<Item = &'a NodeId>) {
        let mut views = TAB_VIEWS.lock();
        node_ids.for_each(|id| {
            views.remove(id);
        });
    }

    pub(crate) fn add_all<'a>(node_ids: impl Iterator<Item = &'a NodeId>) {
        let mut views = TAB_VIEWS.lock();

        node_ids.cloned().for_each(|id| {
            views.insert(id);
        });
    }

    pub fn current() -> Option<NodeId> {
        let index = TAB_INDEX.load(Ordering::Relaxed);
        let all = TAB_VIEWS.lock().clone();
        TAB_VIEWS.lock().member(index).cloned()
    }
}

pub struct Views;

impl Views {
    pub fn all() -> Vec<NodeId> {
        VIEWS.lock().iter().cloned().collect()
    }

    pub fn for_each<F>(f: F) 
        where F: FnMut(&NodeId)
    {
        VIEWS.lock().iter().for_each(f);
    }

    pub(crate) fn insert(node_id: NodeId) {
        VIEWS.lock().insert(node_id);
    }

    fn remove(node_id: &NodeId) {
        VIEWS.lock().remove(node_id);
    }
}

pub trait View {
    type State: 'static;

    fn on_event(&mut self, event: Event, nodes: &mut Nodes<'_>) {
    }

    fn get_state(&self) -> &dyn State {
        &()
    }

    fn tick(&mut self) {
    }

    fn focus(&mut self) {
    }

    fn blur(&mut self) {
    }
}

pub trait AnyView : Debug + Send {
    fn on_any_event(&mut self, ev: Event, nodes: &mut Nodes<'_>);

    fn get_any_state(&self) -> &dyn State;

    fn tick_any(&mut self);

    fn focus_any(&mut self);

    fn blur_any(&mut self);
}

impl<T> AnyView for T
where
    T: View + Debug + Send,
{
    fn on_any_event(&mut self, event: Event, nodes: &mut Nodes<'_>) {
        self.on_event(event, nodes);
    }

    fn get_any_state(&self) -> &dyn State {
        self.get_state()
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::view;

    #[test]
    fn events() {
        let v = view("a-view");
    }
}
