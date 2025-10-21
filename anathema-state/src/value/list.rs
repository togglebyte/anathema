//! The reason the `List<T>` has the insert and remove implementation on `Value<List<T>>` instead of
//! having it directly on `List<T>`, unlike `Map<T>` is because the list generates different
//! changes and are used with for-loops, unlike the map.
use std::collections::VecDeque;
use std::ops::DerefMut;

use super::{AnonValue, Value};
use crate::{states::{AnyList, State}, value::{changes::Change, rcval::{changed, Subs}, Type, ValueMut}, ValueRef};

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
    // This does not trigger a change but still gives mutable access
    // to the underlying list.
    fn with_mut<F, U>(&mut self, f: F) -> U
    where
        F: FnOnce(&mut Subs, &mut List<T>) -> U,
    {
        let (subs, mut inner) = self.untracked_mut();

        let list: &mut dyn State = inner.deref_mut();
        let list: &mut dyn std::any::Any = list;
        let list: &mut List<T> = list.downcast_mut().expect("the type should never change");

        let ret_val = f(subs, list);

        ret_val
    }

    /// Create an empty list
    pub fn empty() -> Self {
        let list = List { inner: VecDeque::new() };
        Value::new(list)
    }

    /// Clear the list
    pub fn clear(&mut self) {
        self.set(List::empty());
    }

    /// Retain all values that matches the predicate
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Value<T>) -> bool,
    {
        let mut index = 0;
        self.with_mut(|subs, list| {
            list.inner.retain(|value| {
                let retain = f(value);
                if !retain {
                    subs.changed(Change::Removed(index));
                }
                index += 1;
                retain
            });
        });
    }

    /// Return all values matching the predicate.
    ///
    /// Note that unlike `Vec::extract_if`, this will allocate a vector for the result
    pub fn extract_if<F>(&mut self, mut f: F) -> Vec<Value<T>>
    where
        F: FnMut(&Value<T>) -> bool,
    {
        self.with_mut(|subs, List { inner: list, .. }| {
            let mut extraction = vec![];

            let mut del = 0;

            for i in 0..list.len() {
                let index = i - del;
                if f(&list[index]) {
                    let value = list.remove(index).expect("value is present");
                    extraction.push(value);
                    subs.changed(Change::Removed(index as u32));
                    del += 1;
                }
            }

            extraction
        })
    }

    /// Push a value to the list
    pub fn push(&mut self, value: T) {
        self.push_back(value)
    }

    /// Push a value to the back of the list
    pub fn push_back(&mut self, value: T) {
        let value = Value::new(value);

        let index = self.with_mut(|_, list| {
            let index = list.len();
            list.inner.push_back(value);
            index as u32
        });

        self.changed(Change::Inserted(index));
    }

    /// Push a value to the front of the list
    pub fn push_front(&mut self, value: impl Into<Value<T>>) {
        let value = value.into();
        self.with_mut(|_, list| list.inner.push_front(value));
        self.changed(Change::Inserted(0));
    }

    /// Insert a value at a given index.
    ///
    /// # Panics
    ///
    /// Will panic if the index is out of bounds
    pub fn insert(&mut self, index: usize, value: impl Into<Value<T>>) {
        let value = value.into();
        self.with_mut(|_, list| list.inner.insert(index, value));
        self.changed(Change::Inserted(index as u32));
    }

    /// Remove a value from the list.
    /// If the value isn't in the list `None` is returned.
    pub fn remove(&mut self, index: usize) -> Option<Value<T>> {
        let value = self.with_mut(|_, list| list.inner.remove(index));
        self.changed(Change::Removed(index as u32));
        value
    }

    /// Pop a value from the front of the list
    pub fn pop_front(&mut self) -> Option<Value<T>> {
        let value = self.with_mut(|_, list| list.inner.pop_front());
        if value.is_some() {
            self.changed(Change::Removed(0));
        }
        value
    }

    /// Pop a value from the back of the list
    pub fn pop_back(&mut self) -> Option<Value<T>> {
        let value = self.with_mut(|_, list| list.inner.pop_back());
        if value.is_some() {
            let index = self.len();
            self.changed(Change::Removed(index as u32));
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
        self.with_mut(|_, list| {
            list.inner.iter_mut().for_each(|val| {
                f(&mut *val.to_mut());
            })
        });
    }

    /// Merge the list with another list.
    pub fn merge(&mut self, other: &mut Self) {
        while let Some(value) = other.pop_front() {
            let index = self.with_mut(|_, list| {
                let index = list.len();
                list.inner.push_back(value);
                index as u32
            });

            self.changed(Change::Inserted(index));
        }
    }

    /// Return the length of the list
    pub fn len(&self) -> usize {
        self.to_ref().len()
    }

    /// Returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.to_ref().is_empty()
    }
}

impl<T: State> AnyList for List<T> {
    fn lookup(&self, index: usize) -> Option<AnonValue> {
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

    use anathema_store::slab::Key;

    use crate::value::changes::Changes;

    use super::*;

    fn changes() -> Vec<(Key, Change)> {
        let mut changes = Changes::empty();
        crate::value::drain_changes(&mut changes);
        changes.into()
    }

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
        list.reference().subscribe(Key::ZERO);
        list.push_back(1);

        let (_, change) = changes().remove(0);
        assert!(matches!(change, Change::Inserted(_)));
    }

    #[test]
    fn notify_remove() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(1);
        list.reference().subscribe(Key::ZERO);
        list.remove(0);

        let change = changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(_))));
    }

    #[test]
    fn notify_pop_front() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(1);
        list.push_back(2);
        list.reference().subscribe(Key::ZERO);
        let front = list.pop_front();

        let change = changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(0))));
        assert_eq!(*front.unwrap().to_ref(), 1);
    }

    #[test]
    fn notify_clear() {
        let mut list = Value::new(List::<u32>::empty());
        list.reference().subscribe(Key::ZERO);
        list.clear();

        let change = changes().remove(0);
        assert!(matches!(change, (_, Change::Changed)));
    }

    #[test]
    fn notify_retain() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        list.reference().subscribe(Key::ZERO);

        list.retain(|val| *val.to_ref() == 1);

        let mut changes = changes();
        assert!(matches!(changes.remove(0), (_, Change::Removed(1))));
        assert!(matches!(changes.remove(0), (_, Change::Removed(2))));
    }

    #[test]
    fn notify_extract_if() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        list.push_back(4);

        list.reference().subscribe(Key::ZERO);

        let result = list.extract_if(|val| *val.to_ref() % 2 == 0);

        let mut changes = changes();
        assert!(matches!(changes.remove(0), (_, Change::Removed(1))));
        assert!(matches!(changes.remove(0), (_, Change::Removed(2))));

        assert_eq!(*result[0].to_ref(), 2);
        assert_eq!(*result[1].to_ref(), 4);
    }

    #[test]
    fn notify_pop_back() {
        let mut list = Value::new(List::<u32>::empty());
        list.push_back(0);
        list.push_back(1);
        list.reference().subscribe(Key::ZERO);
        list.pop_back();

        let change = changes().remove(0);
        assert!(matches!(change, (_, Change::Removed(1))));
    }

    #[test]
    fn pop_empty_list() {
        let mut list = Value::new(List::<u32>::empty());
        list.reference().subscribe(Key::ZERO);
        list.pop_back();
        assert!(changes().is_empty());

        list.pop_front();
        assert!(changes().is_empty());
    }
}
