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

    pub fn new(inner: Vec<StateValue<T>>) -> Self {
        Self {
            inner,
            subscribers: RefCell::new(vec![]),
        }
    }

    pub fn subscribe(&self, node_id: NodeId) {
        self.subscribers.borrow_mut().push(node_id);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn lookup(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>>
    where
        for<'a> &'a StateValue<T>: Into<ValueRef<'a>>,
    {
        let Path::Index(index) = key else { return None };
        let value = self.inner.get(*index)?;
        if let Some(node_id) = node_id.cloned() {
            value.subscribe(node_id);
        }
        Some(value.into())
    }

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

    pub fn swap(&mut self, a: usize, b: usize) {
        // self.inner.swap(a, b)
        panic!()
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(StateValue::new(value));
        for s in self.subscribers.borrow().iter() {
            DIRTY_NODES.with(|nodes| nodes.borrow_mut().push((s.clone(), Change::Add)));
        }
    }

    pub fn insert(&mut self, index: usize, value: StateValue<T>) {
        // self.inner.insert(index, value)
        panic!()
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(value: Vec<T>) -> Self {
        let inner = value.into_iter().map(StateValue::new).collect();
        Self::new(inner)
    }
}

impl<T: Debug> Collection for Map<T>
where
    for<'a> ValueRef<'a>: From<&'a T>,
{
    fn gets(&self, key: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'_>> {
        match key {
            _ => panic!("this is the next thing to do")
            // Path::Index(_) => {
            //     let value = self.lookup(key, node_id)?;
            //     if let Some(node_id) = node_id.cloned() {
            //         value.subscribe(node_id);
            //     }
            //     Some((&value.inner).into())
            // }
            // Path::Composite(lhs, rhs) => {
            //     let map = self
            //         .lookup(&**lhs, node_id)
            //         .map(|value| (&value.inner).into())?;

            //     match map {
            //         ValueRef::Map(map) => map.gets(rhs, node_id),
            //         _ => None,
            //     }
            // }
            // Path::Index(_) => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_list() {
        let list = List::from(vec![1, 2, 3]);
    }
}
