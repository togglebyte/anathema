//! The reason the `List<T>` has the insert and remove implementation on `Value<List<T>>` instead of
//! having it directly on `List<T>`, unlike `Map<T>` is because the list generates different
//! changes and are used with for-loops, unlike the map.
use std::collections::VecDeque;
use std::ops::DerefMut;

use super::{Shared, Type, Unique, Value};
use crate::states::{AnyList, State};
use crate::store::changed;
use crate::store::values::{get_unique, try_make_shared};
use crate::{Change, PendingValue};

#[derive(Debug)]
pub struct List<T> {
    inner: VecDeque<Value<T>>,
}

impl<T: State> List<T> {
    pub const fn empty() -> Self {
        Self { inner: VecDeque::new() }
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

impl<T: State> Default for List<T> {
    fn default() -> Self {
        Self { inner: VecDeque::new() }
    }
}

/// A `List` of values.
/// ```
/// # use anathema_state::{Value, List};
/// let mut list: Value<List<u32>> = List::empty().into();
/// list.push(123);
/// ```
impl<T: State> Value<List<T>> {
    fn with_mut<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut List<T>) -> U,
    {
        let mut inner = get_unique(self.key.owned());

        let list: &mut dyn State = inner.val.deref_mut();
        let list: &mut dyn std::any::Any = list;
        let list: &mut List<T> = list.downcast_mut().expect("the type should never change");

        let ret_val = f(list);

        crate::store::values::return_owned(self.key.owned(), inner);

        ret_val
    }

    pub fn empty() -> Self {
        let list = List { inner: VecDeque::new() };
        Value::new(list)
    }

    pub fn get<'a>(&'a self, index: usize) -> Option<Shared<'a, T>> {
        let list = &*self.to_ref();
        let value = list.get(index)?;
        let key = value.key;

        let (key, value) = try_make_shared(key.owned())?;
        let shared = Shared::new(key, value);
        Some(shared)
    }

    pub fn get_mut<'a>(&'a mut self, index: usize) -> Option<Unique<'a, T>> {
        let list = &*self.to_ref();
        let value = list.get(index)?;

        let key = value.key;
        let value = Unique {
            value: Some(get_unique(key.owned())),
            key,
            _p: std::marker::PhantomData,
        };
        Some(value)
    }

    /// Push a value to the list
    pub fn push(&mut self, value: T) {
        self.push_back(value)
    }

    /// Push a value to the back of the list
    pub fn push_back(&mut self, value: T) {
        let value = Value::new(value);

        let index = self.with_mut(|list| {
            let index = list.len();
            list.inner.push_back(value);
            index as u32
        });

        changed(self.key, Change::Inserted(index));
    }

    /// Push a value to the front of the list
    pub fn push_front(&mut self, value: impl Into<Value<T>>) {
        let value = value.into();
        self.with_mut(|list| list.inner.push_front(value));
        changed(self.key, Change::Inserted(0));
    }

    /// Insert a value at a given index.
    ///
    /// # Panics
    ///
    /// Will panic if the index is out of bounds
    pub fn insert(&mut self, index: usize, value: impl Into<Value<T>>) {
        let value = value.into();
        self.with_mut(|list| list.inner.insert(index, value));
        changed(self.key, Change::Inserted(index as u32));
    }

    /// Remove a value from the list.
    /// If the value isn't in the list `None` is returned.
    pub fn remove(&mut self, index: usize) -> Option<Value<T>> {
        let value = self.with_mut(|list| list.inner.remove(index));
        changed(self.key, Change::Removed(index as u32));
        value
    }

    /// Pop a value from the front of the list
    pub fn pop_front(&mut self) -> Option<Value<T>> {
        let value = self.with_mut(|list| list.inner.pop_front());
        if value.is_some() {
            changed(self.key, Change::Removed(0));
        }
        value
    }

    /// Pop a value from the back of the list
    pub fn pop_back(&mut self) -> Option<Value<T>> {
        let value = self.with_mut(|list| list.inner.pop_back());
        if value.is_some() {
            let index = self.len();
            changed(self.key, Change::Removed(index as u32));
        }
        value
    }

    /// Alias for `pop_back`
    pub fn pop(&mut self) -> Option<Value<T>> {
        self.pop_back()
    }

    /// Calls a closure on each element of the list.
    /// Each element will be marked as changed.
    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T),
    {
        self.with_mut(|list| {
            list.inner.iter_mut().for_each(|val| {
                f(&mut *val.to_mut());
            })
        });
    }

    pub fn len(&self) -> usize {
        self.to_ref().len()
    }

    pub fn merge(&mut self, other: &mut Self) {
        while let Some(value) = other.pop_front() {
            let index = self.with_mut(|list| {
                let index = list.len();
                list.inner.push_back(value);
                index as u32
            });

            changed(self.key, Change::Inserted(index));
        }
    }

    pub fn is_empty(&self) -> bool {
        self.to_ref().is_empty()
    }
}

impl<T: State> AnyList for List<T> {
    fn lookup(&self, index: usize) -> Option<PendingValue> {
        self.get(index).map(|val| val.reference())
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<T: State> State for List<T> {
    fn type_info(&self) -> Type {
        Type::List
    }

    fn as_any_list(&self) -> Option<&dyn AnyList> {
        Some(self)
    }
}

impl<T> FromIterator<T> for Value<List<T>>
where
    T: State,
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
    T: State,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = iter.into_iter().map(Into::into).collect::<VecDeque<_>>();
        List { inner }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::Subscriber;
    use crate::store::testing::drain_changes;

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

    #[test]
    fn notify_insert() {
        let mut list = Value::new(List::<u32>::empty());
        list.reference().subscribe(Subscriber::ZERO);
        list.push_back(1);

        let (_, change) = drain_changes().remove(0);
        assert!(matches!(change, Change::Inserted(_)));
    }

    #[test]
    fn notify_remove() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(1);
        list.reference().subscribe(Subscriber::ZERO);
        list.remove(0);

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(_))));
    }

    #[test]
    fn notify_pop_front() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(1);
        list.push_back(2);
        list.reference().subscribe(Subscriber::ZERO);
        let front = list.pop_front();

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(0))));
        assert_eq!(*front.unwrap().to_ref(), 1);
    }

    #[test]
    fn notify_pop_back() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(0);
        list.push_back(1);
        list.reference().subscribe(Subscriber::ZERO);
        list.pop_back();

        let change = drain_changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(1))));
    }

    #[test]
    fn pop_empty_list() {
        let mut list = Value::new(List::<u32>::empty());
        list.reference().subscribe(Subscriber::ZERO);
        list.pop_back();
        let changes = drain_changes();
        assert!(changes.is_empty());

        list.pop_front();
        let changes = drain_changes();
        assert!(changes.is_empty());
    }
}
