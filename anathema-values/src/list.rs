use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;

use crate::state::State;
use crate::{Change, Collection, NodeId, Path, StateValue, ValueRef, DIRTY_NODES};

#[derive(Debug)]
pub struct List<T> {
    inner: Vec<StateValue<T>>,
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

    pub fn pop(&mut self) -> Option<StateValue<T>> {
        let ret = self.inner.pop()?;
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

    pub fn remove(&mut self, index: usize) -> StateValue<T> {
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

    pub fn push(&mut self, value: T) {
        self.inner.push(StateValue::new(value));
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
    pub fn get_value(&self, node_id: Option<&NodeId>) -> ValueRef<'_> {
        if let Some(node_id) = node_id.cloned() {
            self.subscribe(node_id);
        }
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
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        match key {
            Path::Index(index) => {
                let Some(value) = self.inner.get(*index) else {
                    return ValueRef::Empty;
                };
                if let Some(node_id) = node_id.cloned() {
                    value.subscribe(node_id);
                }
                value.deref().into()
            }
            Path::Composite(lhs, rhs) => match self.get(lhs, node_id) {
                ValueRef::Map(map) => {
                    map.get(rhs, node_id)
                }
                ValueRef::List(collection) => {
                    collection.get(rhs, node_id)
                }
                _ => ValueRef::Empty,
            },
            Path::Key(_) => ValueRef::Empty,
        }
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
        let path = Path::from("generic_list").compose(0).compose(1);
        let ValueRef::Owned(Owned::Num(x)) = state.get(&path, None) else {
            panic!()
        };
        assert_eq!(x.to_i128(), 2);
    }

    #[test]
    fn create_list() {
        let _list = List::new(vec![1, 2, 3]);
    }
}
