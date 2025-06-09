use super::{PendingValue, Value};
use crate::states::{AnyList, AnyMaybe};
use crate::{AnyMap, Color, Hex, State, TypeId};

pub type Nullable<T> = Maybe<T>;

#[derive(Debug, Default)]
pub struct Maybe<T>(Option<Value<T>>);

impl<T: State> Maybe<T> {
    pub fn none() -> Self {
        Self(None)
    }

    pub fn some(value: T) -> Self {
        Self(Some(Value::new(value)))
    }

    pub fn get_ref(&self) -> Option<&Value<T>> {
        self.0.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut Value<T>> {
        self.0.as_mut()
    }

    pub fn set(&mut self, value: T) {
        match &mut self.0 {
            None => self.0 = Some(Value::new(value)),
            Some(existing) => existing.set(value),
        }
    }

    pub fn map<F, U>(&mut self, mut f: F) -> Option<U>
    where
        F: FnMut(&mut T) -> U,
    {
        let value = self.0.as_mut()?;
        Some(f(&mut *value.to_mut()))
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
        match value {
            Some(val) => Maybe::some(val).into(),
            None => Maybe::none().into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blah() {
        let maybe = Maybe::some(1);
        let value = maybe.get_ref().unwrap();
        let state = value.to_ref();
        assert_eq!(1, value.as_int().unwrap());
    }

    #[test]
    fn nested_nullables() {
        let value = Maybe::some(1);
        assert_eq!(1, value.as_int().unwrap());
    }
}
