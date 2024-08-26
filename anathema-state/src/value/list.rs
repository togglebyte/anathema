use std::collections::VecDeque;

use super::Value;
use crate::store::changed;
use crate::{Change, CommonVal, Path, PendingValue, State, Subscriber, ValueRef};

#[derive(Debug)]
pub struct List<T> {
    inner: VecDeque<Value<T>>,
}

impl<T: 'static + State> List<T> {
    pub fn empty() -> Value<Self> {
        Value::<Self>::empty()
    }

    pub fn from_iter(iter: impl IntoIterator<Item = T>) -> Value<Self> {
        Value::from_iter(iter)
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

impl<T: State> Default for Value<List<T>> {
    fn default() -> Self {
        List::empty()
    }
}

/// A `List` of values.
/// ```
/// # use anathema_state::List;
/// let mut list = List::empty();
/// list.push(123);
/// ```
impl<T: 'static + State> Value<List<T>> {
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
        let key = self.key;
        let list = &mut *self.to_mut();
        let index = list.inner.len();
        let value = value.into();
        changed(key.sub(), Change::Inserted(index as u32, value.to_pending()));
        list.inner.push_back(value);
    }

    /// Push a value to the front of the list
    pub fn push_front(&mut self, value: impl Into<Value<T>>) {
        let key = self.key;
        let list = &mut *self.to_mut();
        let value = value.into();
        changed(key.sub(), Change::Inserted(0, value.to_pending()));
        list.inner.push_front(value);
    }

    /// Insert a value at a given index.
    ///
    /// # Panics
    ///
    /// Will panic if the index is out of bounds
    pub fn insert(&mut self, index: usize, value: impl Into<Value<T>>) {
        let key = self.key;
        let list = &mut *self.to_mut();
        let value = value.into();
        changed(key.sub(), Change::Inserted(index as u32, value.to_pending()));
        list.inner.insert(index, value);
    }

    /// Remove a value from the list.
    /// If the value isn't in the list `None` is returned.
    pub fn remove(&mut self, index: usize) -> Option<Value<T>> {
        let key = self.key;
        let list = &mut *self.to_mut();
        let value = list.inner.remove(index);
        changed(key.sub(), Change::Removed(index as u32));
        value
    }

    /// Pop a value from the front of the list
    pub fn pop_front(&mut self) -> Option<Value<T>> {
        let key = self.key;
        let list = &mut *self.to_mut();
        let value = list.inner.pop_front();
        if value.is_some() {
            changed(key.sub(), Change::Removed(0));
        }
        value
    }

    /// Pop a value from the back of the list
    pub fn pop_back(&mut self) -> Option<Value<T>> {
        let key = self.key;
        let list = &mut *self.to_mut();

        if list.inner.is_empty() {
            return None;
        }

        let value = list.inner.pop_back();
        if value.is_some() {
            let index = list.inner.len();
            changed(key.sub(), Change::Removed(index as u32));
        }
        value
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

impl<T: 'static + State> State for List<T> {
    fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
        let Path::Index(idx) = path else { return None };

        let value = self.inner.get(idx)?;
        Some(value.value_ref(sub))
    }

    fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
        let Path::Index(idx) = path else { return None };
        let value = self.inner.get(idx)?;
        Some(value.to_pending())
    }

    fn to_common(&self) -> Option<CommonVal<'_>> {
        None
    }

    fn count(&self) -> usize {
        self.inner.len()
    }
}

impl<T> FromIterator<T> for Value<List<T>>
where
    T: 'static + State,
    Value<T>: From<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = iter.into_iter().map(Into::into).collect::<VecDeque<_>>();
        let list = List { inner };
        Value::new(list)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::store::testing::drain_changes;
    use crate::Map;

    #[test]
    fn insert() {
        let mut list = List::empty();
        list.push_back(1usize);
        list.push_front(0usize);

        let r = list.to_ref();

        let val = *r.get(0).unwrap().to_ref();
        assert_eq!(val, 0);

        let val = *r.get(1).unwrap().to_ref();
        assert_eq!(val, 1);
    }

    fn setup_map<T: 'static + State>(key: &str, a: T, b: T) -> Value<Map<List<T>>> {
        let mut map = Map::<List<T>>::empty();
        let mut list = List::empty();
        list.push_back(a);
        list.push_back(b);
        map.insert(key, list);
        map
    }

    #[test]
    fn notify_insert() {
        let mut map = setup_map("a", 1, 2);

        let mut list = map.to_mut();
        let list = list.get_mut("a").unwrap();
        let _vr = list.value_ref(Subscriber::ZERO);
        list.push_back(1);

        let (_, change) = drain_changes().remove(0);
        assert!(matches!(change, Change::Inserted(_, _)));
    }

    #[test]
    fn notify_remove() {
        let mut map = setup_map("a", 1, 2);

        let mut list = map.to_mut();
        let list = list.get_mut("a").unwrap();
        let _vr = list.value_ref(Subscriber::ZERO);
        list.remove(0);

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(_))));
    }

    #[test]
    fn notify_pop_front() {
        let mut map = setup_map("a", 1, 2);

        let mut list = map.to_mut();
        let list = list.get_mut("a").unwrap();

        let _vr = list.value_ref(Subscriber::ZERO);
        list.pop_front();

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(0))));
    }

    #[test]
    fn notify_pop_back() {
        let mut map = setup_map("a", 1, 2);

        let mut list = map.to_mut();
        let list = list.get_mut("a").unwrap();

        let _vr = list.value_ref(Subscriber::ZERO);
        list.pop_back();

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(1))));
    }
}
