use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::{Attributes, NodeId, Path, State, ValueRef};

#[derive(Debug)]
pub enum Value<T> {
    Static(T),
    /// Any value associated with the state is subject to change
    Cached {
        val: Option<T>,
        path: Path,
    },
    Empty,
}

impl<T> Value<T> {
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Static(val) => Some(val),
            Self::Cached { val, .. } => val.as_ref(),
            Self::Empty => None,
        }
    }

    pub fn reload(&mut self, state: &mut impl State) {
        panic!()
    }
}

#[derive(Debug, Clone)]
pub enum LocalScope<'expr> {
    Empty,
    Value(Rc<(Path, ValueRef<'expr>)>),
}

impl<'expr> LocalScope<'expr> {
    pub fn new(path: Path, value: ValueRef<'expr>) -> Self {
        Self::Value(Rc::new((path, value)))
    }

    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn lookup(&self, path: &Path) -> Option<ValueRef<'expr>> {
        match self {
            Self::Empty => None,
            Self::Value(val) if val.0.eq(path) => Some(val.1.clone()),
            Self::Value(_) => None,
        }
    }
}

pub struct Scopes<'a, 'expr> {
    scope: LocalScope<'expr>,
    parent: Option<&'a Scopes<'a, 'expr>>,
}

impl<'a, 'expr> Scopes<'a, 'expr> {
    fn new(scope: LocalScope<'expr>) -> Self {
        Self {
            scope,
            parent: None
        }
    }

    pub fn reparent(&self, scope: LocalScope<'expr>) -> Scopes<'_, 'expr> {
        Scopes {
            scope,
            parent: Some(self)
        }
    }

    fn lookup(&self, path: &Path) -> Option<ValueRef<'expr>> {
        self.scope
            .lookup(path)
            .or_else(|| self.parent.and_then(|p| p.lookup(path)))
    }
}

pub struct UpdateScope<'a, 'expr> {
    scope: Option<&'a LocalScope<'expr>>,
    parent: Option<&'a UpdateScope<'a, 'expr>>
}

impl<'a, 'expr> UpdateScope<'a, 'expr> {
    pub fn root() -> Self {
        Self {
            scope: None,
            parent: None,
        }
    }

    pub fn reparent(&'a self, scope: &'a LocalScope<'expr>) -> UpdateScope<'_, 'expr> {
        Self {
            scope: Some(scope),
            parent: Some(self)
        }
    }

    fn lookup(&self, path: &Path) -> Option<ValueRef<'expr>> {
        self.scope
            .and_then(|scope| scope.lookup(path))
            .or_else(|| self.parent.and_then(|p| p.lookup(path)))
    }
}

pub struct Context<'a, 'expr> {
    pub state: &'a dyn State,
    pub scopes: Scopes<'a, 'expr>,
}

impl<'a, 'expr> Context<'a, 'expr> {
    pub fn root(state: &'a dyn State) -> Self {
        Self::new(state, LocalScope::Empty)
    }

    pub fn new(state: &'a dyn State, scope: LocalScope<'expr>) -> Self {
        Self {
            state,
            scopes: Scopes::new(scope),
        }
    }

    pub fn reparent(&'a self, scope: LocalScope<'expr>) -> Context<'a, 'expr> {
        Self {
            state: self.state,
            scopes: self.scopes.reparent(scope),
        }
    }

    // pub fn reparent(&self, scope: LocalScope<'expr>) -> Context<'_, 'expr> {

    pub fn new_scope(&self) -> LocalScope<'expr> {
        self.scopes.scope.clone()
    }

    pub fn lookup_value<T>(&self, path: &Path, node_id: Option<&NodeId>) -> Value<T>
    where
        for<'b> T: TryFrom<ValueRef<'b>>,
    {
        panic!()
        // // TODO: come back and unwack this one.
        // //       the top two arms just differ because of the path, but the `Deferred` owns the path
        // match self.scope.lookup(path) {
        //     Some(ValueRef::Deferred(path)) => Value::Cached {
        //         val: self
        //             .state
        //             .get(&path, node_id)
        //             .and_then(|val_ref| T::try_from(val_ref).ok()),
        //         path: path.clone(),
        //     },
        //     None => Value::Cached {
        //         val: self
        //             .state
        //             .get(&path, node_id)
        //             .and_then(|val_ref| T::try_from(val_ref).ok()),
        //         path: path.clone(),
        //     },
        //     Some(value_ref) => match T::try_from(value_ref) {
        //         Ok(val) => Value::Static(val),
        //         Err(_) => Value::Empty,
        //     },
        // }
    }

    // pub(super) fn lookup_old(&self, path: &Path, node_id: Option<&NodeId>) -> Option<ValueRef<'expr>> {
    //     self.scope
    //         .lookup(path)
    //         .or_else(|| self.state.get(path, node_id))
    // }

    /// Lookup a value, if the value belongs to the state it returns a deferred value
    /// instead, to be resolved at a later stage.
    pub fn lookup(&self, path: &Path) -> Option<ValueRef<'expr>> {
        self.scopes
            .lookup(path)
            .or_else(|| Some(ValueRef::Deferred(path.clone())))
    }

    pub fn attribute<T: ?Sized>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &'expr Attributes,
    ) -> Value<T>
    where
        T: Clone,
        for<'b> T: TryFrom<ValueRef<'b>>,
    {
        let Some(value) = attributes.get(key.as_ref()) else {
            return Value::Empty;
        };
        value.resolve(self, node_id)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;
    use crate::ValueExpr;

    type Sub = usize;

    #[test]
    fn scope_value() {
        let mut scope = Scope::new(None);
        scope.scope("value".into(), ValueRef::Str("hello world"));

        let mut inner = scope.reparent();

        inner.scope("value".into(), ValueRef::Str("inner hello"));
        let ValueRef::Str(lhs) = inner.lookup(&"value".into()).unwrap() else {
            panic!()
        };

        assert_eq!(lhs, "inner hello");

        let ValueRef::Str(lhs) = scope.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, "hello world");
    }

    #[test]
    fn dynamic_attribute() {
        let mut state = TestState::new();
        let mut root = Scope::new(None);
        let ctx = Context::new(&mut state, &mut root);
        let mut attributes = Attributes::new();
        attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

        let id = Some(123.into());
        let name = ctx.attribute::<String>("name", id.as_ref(), &attributes);
        assert_eq!("Dirk Gently", name.value().unwrap());
    }

    #[test]
    fn context_lookup() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);

        let path = Path::from("inner").compose("name");
        let value = context.lookup_value::<String>(&path, None);
        let value: &str = value.value().unwrap();
        assert!(matches!(value, "Fiddle McStick"));
    }

    #[test]
    fn singular_state_value() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);
        let path = Path::from("inner").compose("name");
    }

    #[test]
    fn collection_with_one_state_value() {
        let state = TestState::new();
        let scope = Scope::new(None);
        let context = Context::new(&state, &scope);
    }
}
