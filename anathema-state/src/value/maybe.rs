use super::{PendingValue, Value};
use crate::states::AnyMaybe;
use crate::{State, TypeId};

pub type Nullable<T> = Maybe<T>;

#[derive(Debug)]
pub struct Maybe<T>(Option<Value<T>>);

impl<T> Default for Maybe<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: State> Maybe<T> {
    /// Create a Maybe with no value
    pub fn none() -> Self {
        Self(None)
    }

    /// Create a Maybe with a value
    pub fn some(value: T) -> Self {
        Self(Some(Value::new(value)))
    }

    /// Take the underlying value
    pub fn take(&mut self) -> Option<Value<T>> {
        self.0.take()
    }

    /// Get option of a reference to the underlying value
    pub fn get_ref(&self) -> Option<&Value<T>> {
        self.0.as_ref()
    }

    /// Get option of a mutable reference to the underlying value
    pub fn get_mut(&mut self) -> Option<&mut Value<T>> {
        self.0.as_mut()
    }

    /// Set / update the value
    pub fn set(&mut self, value: T) {
        match &mut self.0 {
            None => self.0 = Some(Value::new(value)),
            Some(existing) => existing.set(value),
        }
    }

    /// Update the current value.
    /// If the input value is `None` the underlying value will be removed.
    /// If the input value is `Some(T)` the underlying value will be replaced.
    pub fn update(&mut self, value: Option<T>) {
        match (self.get_mut(), value) {
            (None, None) => (),
            (None, Some(value)) => *self = Maybe::some(value),
            (Some(_), None) => *self = Maybe::none(),
            (Some(current), Some(new)) => current.set(new),
        }
    }

    pub fn map_mut<F, U>(&mut self, mut f: F) -> Option<U>
    where
        F: FnMut(&mut T) -> U,
    {
        let value = self.0.as_mut()?;
        Some(f(&mut *value.to_mut()))
    }

    pub fn map_ref<F, U>(&self, f: F) -> Option<U>
    where
        F: Fn(&T) -> U,
    {
        let value = self.0.as_ref()?;
        Some(f(&*value.to_ref()))
    }

    pub fn and_then_ref<F, U>(&self, f: F) -> Option<U>
    where
        F: Fn(&T) -> Option<U>,
    {
        let value = self.0.as_ref()?;
        f(&*value.to_ref())
    }

    pub fn and_then<F, U>(&mut self, mut f: F) -> Option<U>
    where
        F: FnMut(&mut T) -> Option<U>,
    {
        let value = self.0.as_mut()?;
        f(&mut *value.to_mut())
    }
}

impl<T> TypeId for Maybe<T> {
    const TYPE: super::Type = super::Type::Maybe;
}

impl<T: State + TypeId> State for Maybe<T> {
    fn type_info(&self) -> super::Type {
        Self::TYPE
    }

    fn as_maybe(&self) -> Option<&dyn AnyMaybe> {
        Some(self)
    }
}

impl<T: State> AnyMaybe for Maybe<T> {
    fn get(&self) -> Option<PendingValue> {
        let value = self.get_ref()?;
        Some(value.reference())
    }
}

impl<T: State + TypeId> From<Option<T>> for Value<Maybe<T>> {
    fn from(value: Option<T>) -> Self {
        Maybe::from(value).into()
    }
}

impl<T: State + TypeId> From<Option<T>> for Maybe<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(val) => Maybe::some(val),
            None => Maybe::none(),
        }
    }
}

impl<T: State + TypeId> Value<Maybe<T>> {
    pub fn map<F, U>(&mut self, f: F) -> Option<U>
    where
        F: FnMut(&mut T) -> U,
    {
        let mut value = self.to_mut();
        value.map_mut(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nullable_int() {
        let value = Maybe::some(1);
        let inner = value.get().unwrap();
        assert_eq!(1, inner.as_state().unwrap().as_int().unwrap());
    }

    #[test]
    fn nested_nullables() {
        let value = Maybe::some(Maybe::some(1));
        let one = value.and_then_ref(|inner_map| inner_map.map_ref(|m| *m)).unwrap();
        assert_eq!(one, 1);
    }
}
