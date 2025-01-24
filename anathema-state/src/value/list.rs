use std::collections::VecDeque;

use super::{Type, Value};
use crate::states::{AnyList, AnyState};
use crate::store::changed;
use crate::{Change, Path, PendingValue, State, Subscriber, ValueRef};

// TODO: Optimisation: changing the list should probably just create one
//       change instead of two for the list.
//
//       Since the list is used with `deref_mut` it creates a `Unique<T>`
//       which will insert a `Change::Updated` entry, even though
//       that's probably superfluous.

#[derive(Debug)]
pub struct List<T> {
    inner: VecDeque<Value<T>>,
}

impl<T: AnyState> List<T> {
    pub fn empty() -> Self {
        Self { inner: VecDeque::new() }
    }

    /// Push a value to the list
    pub fn push(&mut self, value: impl Into<Value<T>>) {
        self.push_back(value)
    }

    /// Push a value to the back of the list
    pub fn push_back(&mut self, value: impl Into<Value<T>>) {
        self.inner.push_back(value.into());
    }

    /// Push a value to the back of the list
    pub fn push_front(&mut self, value: impl Into<Value<T>>) {
        self.inner.push_front(value.into());
    }

    /// Push a value to the list
    pub fn remove(&mut self, index: usize) -> Option<Value<T>> {
        self.inner.remove(index)
    }

    pub fn get(&self, index: usize) -> Option<&Value<T>> {
        self.inner.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value<T>> {
        self.inner.get_mut(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value<T>> {
        self.inner.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value<T>> {
        self.inner.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T: AnyState> Default for List<T> {
    fn default() -> Self {
        Self {
            inner: VecDeque::new()
        }
    }
}

/// A `List` of values.
/// ```
/// # use anathema_state::List;
/// let mut list = List::empty();
/// list.push(123);
/// ```
impl<T: AnyState + 'static> Value<List<T>> {
    pub fn empty() -> Self {
        let list = List { inner: VecDeque::new() };
        Value::new(list)
    }

    /// Push a value to the list
    pub fn push(&mut self, value: impl Into<Value<T>>) {
        self.push_back(value)
    }

    /// Push a value to the back of the list
    pub fn push_back(&mut self, value: impl Into<Value<T>>) {
        let (index, value) = {
            let list = &mut *self.to_mut();
            let index = list.inner.len();
            let value = value.into();
            let pending = value.reference();
            list.inner.push_back(value);

            (index as u32, pending)
        };
        changed(self.key, Change::Inserted(index, value));
    }

    /// Push a value to the front of the list
    pub fn push_front(&mut self, value: impl Into<Value<T>>) {
        let value = value.into();
        let pending = value.reference();
        self.to_mut().inner.push_front(value);
        changed(self.key, Change::Inserted(0, pending));
    }

    /// Insert a value at a given index.
    ///
    /// # Panics
    ///
    /// Will panic if the index is out of bounds
    pub fn insert(&mut self, index: usize, value: impl Into<Value<T>>) {
        let value = value.into();
        let pending = value.reference();
        self.to_mut().inner.insert(index, value);
        changed(self.key, Change::Inserted(index as u32, pending));
    }

    /// Remove a value from the list.
    /// If the value isn't in the list `None` is returned.
    pub fn remove(&mut self, index: usize) -> Option<Value<T>> {
        let value = self.to_mut().inner.remove(index);
        changed(self.key, Change::Removed(index as u32));
        value
    }

    /// Pop a value from the front of the list
    pub fn pop_front(&mut self) -> Option<Value<T>> {
        let value = self.to_mut().inner.pop_front()?;
        changed(self.key, Change::Removed(0));
        Some(value)
    }

    /// Pop a value from the back of the list
    pub fn pop_back(&mut self) -> Option<Value<T>> {
        let key = self.key;

        let list = &mut *self.to_mut();
        let value = list.inner.pop_back()?;
        let index = list.inner.len();

        changed(key, Change::Removed(index as u32));
        Some(value)
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        let list = &mut *self.to_mut();
        list.iter_mut().for_each(|el| f(&mut *el.to_mut()));
    }

    pub fn len(&self) -> usize {
        self.to_ref().len()
    }
}

impl<T: AnyState + 'static> AnyList for List<T> {
    fn lookup(&self, index: usize) -> Option<PendingValue> {
        self.get(index).map(|val| val.reference())
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T: AnyState + 'static> AnyState for List<T> {
    fn type_info(&self) -> Type {
        Type::List
    }

    fn to_any_ref(&self) -> &dyn std::any::Any {
        self
    }

    fn to_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any_list(&self) -> Option<&dyn AnyList> {
        Some(self)
    }
}

impl<T> FromIterator<T> for Value<List<T>>
where
    T: AnyState + 'static,
    Value<T>: From<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = iter.into_iter().map(Into::into).collect::<VecDeque<_>>();
        let list = List { inner };
        Value::new(list)
    }
}

impl<T> FromIterator<T> for List<T>
where
    T: AnyState + 'static,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = iter.into_iter().map(Into::into).collect::<VecDeque<_>>();
        List { inner }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::store::testing::drain_changes;
    use crate::Map;

    #[test]
    fn insert() {
        let mut list = Value::new(List::empty());
        list.push_back(1usize);
        list.push_front(0usize);
        let list = list.to_ref();

        let val = list.get(0).unwrap().to_ref();
        assert_eq!(*val, 0);

        let val = list.get(1).unwrap().to_ref();
        assert_eq!(*val, 1);
    }

    fn setup_map<T: 'static + State>(key: &str, a: T, b: T) -> Map<List<T>> {
        let mut map = Map::empty();
        let mut list = List::empty();
        list.push_back(a);
        list.push_back(b);
        map.insert(key, list);
        map
    }

    #[test]
    fn notify_insert() {
        let mut map = setup_map("a", 1, 2);

        let list = map.get_mut("a").unwrap();
        let list_ref = list.reference();
        list_ref.subscribe(Subscriber::ZERO);

        list.push_back(1);

        let (_, change) = drain_changes().remove(0);
        assert!(matches!(change, Change::Inserted(_, _)));
    }

    #[test]
    fn notify_remove() {
        let mut map = setup_map("a", 1, 2);

        let list = map.get_mut("a").unwrap();
        let list_ref = list.reference();
        list_ref.subscribe(Subscriber::ZERO);
        list.remove(0);

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(_))));
    }

    #[test]
    fn notify_pop_front() {
        let mut map = setup_map("a", 1, 2);

        let list = map.get_mut("a").unwrap();

        let list_ref = list.reference();
        list_ref.subscribe(Subscriber::ZERO);
        list.pop_front();

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(0))));
    }

    #[test]
    fn notify_pop_back() {
        let mut map = setup_map("a", 1, 2);

        let list = map.get_mut("a").unwrap();

        let list_ref = list.reference();
        list_ref.subscribe(Subscriber::ZERO);
        list.pop_back();

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(1))));
    }
}
