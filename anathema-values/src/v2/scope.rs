use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::{Path, State};

// State
// name:       "string here"
// collection: [1, 2, 3]
//
// for name in state.collection {
//     // scope level 1
//     name = 1
//
//     for lark in state.collection {
//         // scope level 2
//         text name
//     }
// }

#[derive(Debug, Clone)]
pub enum ScopeValue {
    Static(Rc<str>),
    List(Rc<[ScopeValue]>),
    Dyn(Path),
}

impl<const N: usize> From<[ScopeValue; N]> for ScopeValue {
    fn from(arr: [ScopeValue; N]) -> Self {
        if N == 1 {
            arr.into_iter()
                .next()
                .expect("this is always going to be an array with a size of one")
        } else {
            ScopeValue::List(Rc::new(arr))
        }
    }
}

// TODO: add a testing flag for this
impl From<i32> for ScopeValue {
    fn from(s: i32) -> Self {
        Self::Static(s.to_string().into())
    }
}

// TODO: add a testing flag for this
impl From<String> for ScopeValue {
    fn from(s: String) -> Self {
        Self::Static(s.into())
    }
}

pub struct Scope<'a> {
    inner: HashMap<Path, Cow<'a, ScopeValue>>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            inner: HashMap::new(),
            parent,
        }
    }

    pub fn scope(&mut self, path: Path, value: Cow<'a, ScopeValue>) {
        self.inner.insert(path, value);
    }

    pub fn lookup_parent(&self, path: &Path) -> Option<&'a ScopeValue> {
        self.parent
            .and_then(|parent| parent.lookup(path))
            // .map(Deref::deref)
    }

    pub fn lookup(&self, path: &Path) -> Option<&ScopeValue> {
        self.inner
            .get(path)
            .map(Deref::deref)
            .or_else(|| self.parent.and_then(|parent| parent.lookup(path)))
    }

    pub fn lookup_list(&self, path: &Path) -> Rc<[ScopeValue]> {
        let value = self
            .inner
            .get(path)
            .map(Deref::deref)
            .or_else(|| self.parent.and_then(|parent| parent.lookup(path)));

        match value {
            Some(ScopeValue::List(value)) => value.clone(),
            _ => Rc::new([]),
        }
    }

    pub fn from_self(&'a self) -> Scope<'a> {
        Scope::new(Some(self))
    }
}

pub struct Context<'a, 'val, S> {
    state: &'a S,
    scope: &'a Scope<'val>,
}

impl<'a, 'val, S: State> Context<'a, 'val, S> {
    pub fn new(state: &'a S, scope: &'a mut Scope<'val>) -> Self {
        Self {
            state,
            scope,
        }
    }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    fn get<T>(&self, path: &Path) -> Option<T> 
        where T: for<'magic> TryFrom<&'magic ScopeValue>
    {
        match self.scope.lookup(&path) {
            Some(val) => match val {
                ScopeValue::Dyn(path) => self.get(path),
                val => T::try_from(val).ok(),
            }
            None => self.state.get_typed(&path),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_value() {
        let mut root = Scope::new(None);
        // root.scope(
        //     "list".into(),
        //     ScopeValue::List(Rc::new([ScopeValue::Static("hello world".into())])),
        // );
        root.scope("lol".into(), Cow::Owned(ScopeValue::Static("lolly".into())));

        let mut inner = Scope::new(Some(&root));
        let value = inner.lookup_parent(&"lol".into()).unwrap();
        // let scope_value = value[0].clone();
        // inner.scope("list".into(), scope_value);
        inner.scope("lark".into(), Cow::Borrowed(value));

        // let ScopeValue::Static(actual) = inner.lookup(&"list".into()).unwrap() else { panic!() };
        // assert_eq!("hello world", &**actual);

        let Cow::Borrowed(ScopeValue::Static(lhs)) = inner.lookup(&"lark".into()).unwrap() else { panic!() };
        let Cow::Owned(ScopeValue::Static(rhs)) = inner.lookup(&"lol".into()).unwrap() else { panic!() };
        assert_eq!(lhs, rhs);

        // let ScopeValue::Static(actual) = inner.lookup(&"lol".into()).unwrap() else { panic!() };
        // assert_eq!("lolly", &**actual);
    }
}
