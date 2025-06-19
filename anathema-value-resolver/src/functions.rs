use std::collections::HashMap;
use std::fmt::Debug;

use crate::ValueKind;

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
        inner.insert("add".into(), Function::from(add));
        inner.insert("to_upper".into(), Function::from(to_upper));
        inner.insert("to_lower".into(), Function::from(to_lower));
        Self { inner }
    }

    pub fn insert(&mut self, ident: impl Into<String>, f: impl Into<Function>) {
        self.inner.insert(ident.into(), f.into());
    }

    pub fn lookup(&self, ident: &str) -> Option<&Function> {
        self.inner.get(ident)
    }
}

fn add<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 2 {
        return ValueKind::Null;
    }

    let values = args[0].as_int().zip(args[1].as_int());

    match values {
        Some((lhs, rhs)) => ValueKind::Int(lhs + rhs),
        None => ValueKind::Null,
    }
}

fn to_upper<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let mut buffer = String::new();
    args[0].strings(|s| {
        buffer.push_str(&s.to_uppercase());
        true
    });

    ValueKind::Str(buffer.into())
}

fn to_lower<'bp>(args: &[ValueKind<'bp>]) -> ValueKind<'bp> {
    if args.len() != 1 {
        return ValueKind::Null;
    }

    let mut buffer = String::new();
    args[0].strings(|s| {
        buffer.push_str(&s.to_lowercase());
        true
    });

    ValueKind::Str(buffer.into())
}
