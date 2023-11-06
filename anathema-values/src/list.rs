use std::fmt::Debug;
use std::ops::Deref;

use super::*;
use crate::Path;

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

    // pub fn lookup(&self, path: &Path, _node_id: Option<&NodeId>) -> Option<&StateValue<T>>
    // where
    //     for<'a> ValueRef<'a>: From<&'a T>,
    // {
    //     let Path::Index(index) = path else {
    //         return None;
    //     };
    //     self.inner.get(*index)
    // }

    pub fn pop(&mut self) -> Option<StateValue<T>> {
        let ret = self.inner.pop()?;
        let index = self.inner.len();
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Remove(index))));
        }
        Some(ret)
    }

    pub fn remove(&mut self, index: usize) -> StateValue<T> {
        let ret = self.inner.remove(index);
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Remove(index))));
        }
        ret
    }

    pub fn swap(&mut self, _a: usize, _b: usize) {
        // self.inner.swap(a, b)
        panic!()
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(StateValue::new(value));
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Add)));
        }
    }

    pub fn insert(&mut self, _index: usize, _value: StateValue<T>) {
        // self.inner.insert(index, value)
        panic!()
    }
}

impl<T: Debug> Collection for List<T>
where
    for<'a> ValueRef<'a>: From<&'a T>,
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T> State for List<T>
where
    for<'a> ValueRef<'a>: From<&'a T>,
{
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        match key {
            Path::Index(index) => {
                let value = self.inner.get(*index)?;
                if let Some(node_id) = node_id.cloned() {
                    value.subscribe(node_id);
                }
                Some(value.deref().into())
            }
            Path::Composite(lhs, rhs) => match self.get(lhs, node_id)? {
                ValueRef::Map(collection) | ValueRef::List(collection) => {
                    collection.get(rhs, node_id)
                }
                _ => None,
            },
            Path::Key(_) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::TestState;

    #[test]
    fn access_list() {
        let state = TestState::new();
        let path = Path::from("generic_list").compose(0).compose(1);
        let ValueRef::Owned(Owned::Num(x)) = state.get(&path, None).unwrap() else {
            panic!()
        };
        assert_eq!(x.to_i128(), 2);
    }

    #[test]
    fn create_list() {
        let _list = List::new(vec![1, 2, 3]);
    }
}
