pub(crate) enum Null<T> {
    Value(T),
    Null,
}

impl<T> Null<T> {
    pub fn map<F, U>(self, f: F) -> Null<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Value(val) => Null::Value(f(val)),
            Self::Null => Null::Null,
        }
    }

    pub fn or(self, alt: T) -> T {
        match self {
            Null::Value(val) => val,
            Null::Null => alt,
        }
    }
}

impl<T> From<T> for Null<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Copy> From<&T> for Null<T> {
    fn from(value: &T) -> Self {
        Self::Value(*value)
    }
}
