use super::{AnonValue, Value};
use crate::states::{AnyMaybe, State, TypeId};

#[derive(Debug)]
pub struct Maybe<V>(Option<Value<V>>);

impl<V> Default for Maybe<V> {
    fn default() -> Self {
        Self(None)
    }
}

impl<V: State> Maybe<V> {
    /// Create a Maybe with no value
    pub fn none() -> Self {
        Self(None)
    }

    /// Create a Maybe with a value
    pub fn some(value: V) -> Self {
        Self(Some(Value::new(value)))
    }

    /// Take the underlying value
    pub fn take(&mut self) -> Option<Value<V>> {
        self.0.take()
    }

    /// Get option of a reference to the underlying value
    pub fn get_ref(&self) -> Option<&Value<V>> {
        self.0.as_ref()
    }

    /// Get option of a mutable reference to the underlying value
    pub fn get_mut(&mut self) -> Option<&mut Value<V>> {
        self.0.as_mut()
    }

    /// Set / update the value
    pub fn set(&mut self, value: V) {
        match &mut self.0 {
            None => self.0 = Some(Value::new(value)),
            Some(existing) => existing.set(value),
        }
    }

    /// Update the current value.
    /// If the input value is `None` the underlying value will be removed.
    /// If the input value is `Some(T)` the underlying value will be replaced.
    pub fn update(&mut self, value: Option<V>) {
        match (self.get_mut(), value) {
            (None, None) => (),
            (None, Some(value)) => *self = Maybe::some(value),
            (Some(_), None) => *self = Maybe::none(),
            (Some(current), Some(new)) => current.set(new),
        }
    }

    pub fn map_mut<F, U>(&mut self, mut f: F) -> Option<U>
    where
        F: FnMut(&mut V) -> U,
    {
        let value = self.0.as_mut()?;
        Some(f(&mut *value.to_mut()))
    }

    pub fn map_ref<F, U>(&self, f: F) -> Option<U>
    where
        F: Fn(&V) -> U,
    {
        let value = self.0.as_ref()?;
        Some(f(&*value.to_ref()))
    }

    pub fn and_then_ref<F, U>(&self, f: F) -> Option<U>
    where
        F: Fn(&V) -> Option<U>,
    {
        let value = self.0.as_ref()?;
        f(&*value.to_ref())
    }

    pub fn and_then<F, U>(&mut self, mut f: F) -> Option<U>
    where
        F: FnMut(&mut V) -> Option<U>,
    {
        let value = self.0.as_mut()?;
        f(&mut *value.to_mut())
    }
}

impl<V> TypeId for Maybe<V> {
    const TYPE: super::Type = super::Type::Maybe;
}

impl<V: State + TypeId> State for Maybe<V> {
    fn type_info(&self) -> super::Type {
        Self::TYPE
    }

    fn as_maybe(&self) -> Option<&dyn AnyMaybe> {
        Some(self)
    }
}

impl<V: State> AnyMaybe for Maybe<V> {
    fn get(&self) -> Option<AnonValue> {
        let value = self.get_ref()?;
        Some(value.reference())
    }
}

impl<V: State + TypeId> From<Option<V>> for Value<Maybe<V>> {
    fn from(value: Option<V>) -> Self {
        Maybe::from(value).into()
    }
}

impl<V: State + TypeId> From<Option<V>> for Maybe<V> {
    fn from(value: Option<V>) -> Self {
        match value {
            Some(val) => Maybe::some(val),
            None => Maybe::none(),
        }
    }
}

impl<V: State + TypeId> Value<Maybe<V>> {
    pub fn map<F, U>(&mut self, f: F) -> Option<U>
    where
        F: FnMut(&mut V) -> U,
    {
        let mut value = self.to_mut();
        value.map_mut(f)
    }
}

#[cfg(test)]
mod test {
    // use super::*;
    // use crate::store::testing::drain_changes;
    // use crate::{Change, Subscriber};

    // #[test]
    // fn nullable_int() {
    //     let value = Maybe::some(1);
    //     let inner = value.get().unwrap();
    //     assert_eq!(1, inner.as_state().unwrap().as_int().unwrap());
    // }

    // #[test]
    // fn nested_nullables() {
    //     let value = Maybe::some(Maybe::some(1));
    //     let one = value.and_then_ref(|inner_map| inner_map.map_ref(|m| *m)).unwrap();
    //     assert_eq!(one, 1);
    // }

    // #[test]
    // fn changing_value() {
    //     let mut value = Value::new(Maybe::<u32>::none());
    //     value.reference().subscribe(Subscriber::ZERO);
    //     value.to_mut().update(Some(1));
    //     let mut changes = drain_changes();
    //     assert!(matches!(changes.remove(0), (_, Change::Changed)));
    // }
}
