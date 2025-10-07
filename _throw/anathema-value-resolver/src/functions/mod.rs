use std::collections::HashMap;
use std::fmt::{Debug, Display};

use crate::ValueKind;

mod list;
mod number;
mod string;

#[derive(Debug)]
pub enum Error {
    FunctionAlreadyRegistered(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::FunctionAlreadyRegistered(name) => write!(f, "function `{name}` is already registered"),
        }
    }
}

impl std::error::Error for Error {}

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
        inner.insert("truncate".into(), Function::from(string::truncate));
        inner.insert("to_str".into(), Function::from(string::to_str));
        inner.insert("pad".into(), Function::from(string::pad));
        inner.insert("width".into(), Function::from(string::width));
        inner.insert("to_int".into(), Function::from(number::to_int));
        inner.insert("to_float".into(), Function::from(number::to_float));
        inner.insert("round".into(), Function::from(number::round));
        inner.insert("contains".into(), Function::from(list::contains));
        inner.insert("len".into(), Function::from(list::len));
        Self { inner }
    }

    pub fn insert(&mut self, ident: impl Into<String>, f: impl Into<Function>) -> Result<(), Error> {
        let key = ident.into();
        if self.inner.contains_key(&key) {
            return Err(Error::FunctionAlreadyRegistered(key));
        }
        self.inner.insert(key, f.into());
        Ok(())
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
