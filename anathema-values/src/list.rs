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

    pub fn subscribe(&self, node_id: NodeId) {
        self.subscribers.borrow_mut().push(node_id);
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
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| {
                nodes
                    .borrow_mut()
                    .push((s.clone(), Change::RemoveIndex(index)))
            });
        }
        Some(ret)
    }

    pub fn pop_back(&mut self) -> Option<StateValue<T>> {
        let ret = self.inner.pop_back()?;
        let index = self.inner.len();
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| {
                nodes
                    .borrow_mut()
                    .push((s.clone(), Change::RemoveIndex(index)))
            });
        }
        Some(ret)
    }

    pub fn remove(&mut self, index: usize) -> Option<StateValue<T>> {
        let ret = self.inner.remove(index);
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| {
                nodes
                    .borrow_mut()
                    .push((s.clone(), Change::RemoveIndex(index)))
            });
        }
        ret
    }

    pub fn push_front(&mut self, value: T) {
        self.inner.push_front(StateValue::new(value));
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::InsertIndex(0))));
        }
    }

    pub fn push_back(&mut self, value: T) {
        self.inner.push_back(StateValue::new(value));
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Push)));
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.inner.insert(index, StateValue::new(value));
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| {
                nodes
                    .borrow_mut()
                    .push((s.clone(), Change::InsertIndex(index)))
            });
        }
    }
}


impl<T: Debug> List<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>,
{
    pub fn get_value(&self, node_id: &NodeId) -> ValueRef<'_> {
        self.subscribe(node_id.clone());
        ValueRef::List(self)
    }
}

impl<T: Debug> Collection for List<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T: Debug> State for List<T>
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
    use crate::Owned;

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
}
