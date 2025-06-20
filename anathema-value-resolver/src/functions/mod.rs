use std::collections::HashMap;
use std::fmt::Debug;

use crate::ValueKind;

mod list;
mod number;
mod string;

pub struct Function {
    inner: Box<dyn for<'bp> Fn(&[ValueKind<'bp>]) -> ValueKind<'bp>>,
}

impl Function {
    pub(crate) fn invoke<'bp>(&self, args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
        (self.inner)(args)
    }
}

impl Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fun>")
    }
}

impl<T> From<T> for Function
where
    T: 'static,
    T: for<'bp> Fn(&[ValueKind<'bp>]) -> ValueKind<'bp>,
{
    fn from(value: T) -> Self {
        Self { inner: Box::new(value) }
    }
}

pub struct FunctionTable {
    inner: HashMap<String, Function>,
}

impl FunctionTable {
    pub fn new() -> Self {
        let mut inner = HashMap::new();
        inner.insert("to_upper".into(), Function::from(string::to_upper));
        inner.insert("to_lower".into(), Function::from(string::to_lower));
        inner.insert("contains".into(), Function::from(list::contains));
        Self { inner }
    }

    pub fn insert(&mut self, ident: impl Into<String>, f: impl Into<Function>) {
        self.inner.insert(ident.into(), f.into());
    }

    pub fn lookup(&self, ident: &str) -> Option<&Function> {
        self.inner.get(ident)
    }
}

#[cfg(test)]
mod test {
    use crate::ValueKind;

    pub(crate) fn list<T, U>(items: T) -> ValueKind<'static>
    where
        U: Into<ValueKind<'static>>,
        T: IntoIterator<Item = U>,
    {
        let inner = items.into_iter().map(Into::into).collect::<Box<[ValueKind<'_>]>>();

        ValueKind::List(inner)
    }

    pub(crate) fn value<T>(val: T) -> ValueKind<'static>
    where
        T: Into<ValueKind<'static>>,
    {
        val.into()
    }
}
