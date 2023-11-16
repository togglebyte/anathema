use std::ops::Deref;
use std::fmt::Debug;

use super::*;
use crate::hashmap::HashMap;
use crate::Path;

#[derive(Debug)]
pub struct Map<T> {
    inner: HashMap<String, StateValue<T>>,
    subscribers: RefCell<Vec<NodeId>>,
}

impl<T> Map<T> {
    pub fn empty() -> Self {
        Self::new::<String>(HashMap::new())
    }

    pub fn new<K: Into<String>>(inner: impl IntoIterator<Item = (K, T)>) -> Self {
        let inner = inner
            .into_iter()
            .map(|(k, v)| (k.into(), StateValue::new(v)));
        Self {
            inner: HashMap::from_iter(inner),
            subscribers: RefCell::new(vec![]),
        }
    }

    pub fn subscribe(&self, node_id: NodeId) {
        self.subscribers.borrow_mut().push(node_id);
    }

    pub fn remove(&mut self, key: String) -> Option<StateValue<T>> {
        let ret = self.inner.remove(&key);
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::RemoveKey(key.clone()))));
        }
        ret
    }

    pub fn insert(&mut self, key: String, value: T) {
        self.inner.insert(key.clone(), StateValue::new(value));
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::InsertKey(key.clone()))));
        }
    }

}

impl<T: Debug> Collection for Map<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>
{
    fn len(&self) -> usize {
        self.inner.len()
    }
}


impl<T> State for Map<T>
where
    for<'a> &'a T: Into<ValueRef<'a>>
{
    fn get(&self, key: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        match key {
            Path::Key(key) => {
                let Some(value) = self.inner.get(key) else { return ValueRef::Empty };
                if let Some(node_id) = node_id.cloned() {
                    value.subscribe(node_id);
                }
                value.deref().into()
            }
            Path::Composite(lhs, rhs) => match self.get(lhs, node_id) {
                ValueRef::Map(collection) | ValueRef::List(collection) => {
                    collection.get(rhs, node_id)
                }
                _ => ValueRef::Empty,
            },
            Path::Index(_) => ValueRef::Empty,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::TestState;

    #[test]
    fn access_map() {
        let state = TestState::new();
        let path = Path::from("generic_map").compose("inner").compose("second");
        let ValueRef::Owned(Owned::Num(x)) = state.get(&path, None).unwrap() else {
            panic!()
        };
        assert_eq!(x.to_i128(), 2);
    }
}
