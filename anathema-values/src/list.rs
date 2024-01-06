use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::{Deref, Index, IndexMut};

use crate::state::State;
use crate::{Change, Collection, NodeId, Path, StateValue, ValueRef, DIRTY_NODES};

#[derive(Debug)]
pub struct List<T> {
    inner: VecDeque<StateValue<T>>,
    subscribers: RefCell<Vec<NodeId>>,
}

impl<T> List<T> {
    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn new(inner: impl IntoIterator<Item = T>) -> Self {
        Self {
            inner: inner.into_iter().map(StateValue::new).collect(),
            subscribers: RefCell::new(vec![]),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn pop_front(&mut self) -> Option<StateValue<T>> {
        let ret = self.inner.pop_front()?;
        let index = self.inner.len();
        self.notify(Change::RemoveIndex(index));
        Some(ret)
    }

    pub fn pop_back(&mut self) -> Option<StateValue<T>> {
        let ret = self.inner.pop_back()?;
        let index = self.inner.len();
        self.notify(Change::RemoveIndex(index));
        Some(ret)
    }

    pub fn remove(&mut self, index: usize) -> Option<StateValue<T>> {
        let ret = self.inner.remove(index);
        self.notify(Change::RemoveIndex(index));
        ret
    }

    pub fn push_front(&mut self, value: T) {
        self.inner.push_front(StateValue::new(value));
        self.notify(Change::InsertIndex(0));
    }

    pub fn push_back(&mut self, value: T) {
        self.inner.push_back(StateValue::new(value));
        self.notify(Change::Push);
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.inner.insert(index, StateValue::new(value));
        self.notify(Change::InsertIndex(index));
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> + ExactSizeIterator {
        self.inner.iter().map(|state_val| &**state_val)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + ExactSizeIterator {
        self.inner.iter_mut().map(|state_val| &mut **state_val)
    }

    fn notify(&self, change: Change) {
        for s in self.subscribers.borrow_mut().drain(..) {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), change.clone())));
        }
    }
}

impl<T> Extend<T> for List<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.inner.extend(iter.into_iter().map(|val| val.into()));
        self.notify(Change::Push);
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> List<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>,
{
    pub fn get_value(&self, _node_id: &NodeId) -> ValueRef<'_> {
        ValueRef::List(self)
    }
}

impl<T> Collection for List<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>,
{
    fn len(&self) -> usize {
        self.inner.len()
    }

    fn subscribe(&self, node_id: NodeId) {
        self.subscribers.borrow_mut().push(node_id);
    }
}

impl<T> State for List<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>,
{
    fn state_get(&self, key: &Path, node_id: &NodeId) -> ValueRef<'_> {
        match key {
            Path::Index(index) => {
                let Some(value) = self.inner.get(*index) else {
                    return ValueRef::Empty;
                };
                value.subscribe(node_id.clone());
                value.deref().into()
            }
            Path::Composite(lhs, rhs) => match self.state_get(lhs, node_id) {
                ValueRef::Map(map) => map.state_get(rhs, node_id),
                ValueRef::List(collection) => collection.state_get(rhs, node_id),
                _ => ValueRef::Empty,
            },
            Path::Key(_) => ValueRef::Empty,
        }
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl<I, T> From<I> for List<T>
where
    I: IntoIterator<Item = T>,
{
    fn from(value: I) -> Self {
        Self::new(value)
    }
}

impl<T> Index<usize> for List<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> IndexMut<usize> for List<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::TestState;
    use crate::{drain_dirty_nodes, Owned};

    #[test]
    fn access_list() {
        let state = TestState::new();
        let path = Path::from("generic_list").compose(1);
        let node_id = 0.into();
        let ValueRef::Owned(Owned::Num(x)) = state.state_get(&path, &node_id) else {
            panic!()
        };
        assert_eq!(x.to_i128(), 2);
    }

    #[test]
    fn create_list() {
        let _list = List::new(vec![1, 2, 3]);
    }

    #[test]
    fn iter_mut_marks_values_as_updated() {
        let mut list = List::empty();
        for i in 0..100 {
            list.push_back(i);
            list.inner[i].subscribe(0.into());
        }

        list.iter_mut().next();

        let nodes = drain_dirty_nodes();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn extend_marks_as_pushed() {
        let mut list: List<usize> = List::empty();
        list.subscribe(0.into());
        list.extend(0..100);
        let nodes = drain_dirty_nodes();
        assert_eq!(nodes.len(), 1);
    }
}
