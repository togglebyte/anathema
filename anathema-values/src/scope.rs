use std::rc::Rc;

use crate::{NodeId, Path, State, ValueRef};

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

    pub fn lookup(&self, path: &Path) -> ValueRef<'expr> {
        match self {
            Self::Empty => ValueRef::Empty,
            Self::Value(val) if val.0.eq(path) => val.1.clone(),
            Self::Value(_) => ValueRef::Empty,
        }
    }
}

pub struct Scopes<'a, 'expr> {
    scope: &'a LocalScope<'expr>,
    parent: Option<&'a Scopes<'a, 'expr>>,
}

impl<'a, 'expr> Scopes<'a, 'expr> {
    fn new(scope: &'a LocalScope<'expr>) -> Self {
        Self {
            scope,
            parent: None,
        }
    }

    pub fn reparent(&self, scope: &'a LocalScope<'expr>) -> Scopes<'_, 'expr> {
        Scopes {
            scope,
            parent: Some(self),
        }
    }

    fn lookup(&self, path: &Path) -> ValueRef<'expr> {
        match self.scope.lookup(path) {
            ValueRef::Empty => self.parent.map(|p| p.lookup(path)).unwrap_or(ValueRef::Empty),
            val => val,
        }
    }
}

pub struct Context<'a, 'expr> {
    pub state: &'a dyn State,
    pub scopes: Scopes<'a, 'expr>,
}

impl<'a, 'expr> Context<'a, 'expr> {
    pub fn root(state: &'a dyn State) -> Self {
        Self::new(state, &LocalScope::Empty)
    }

    pub fn new(state: &'a dyn State, scope: &'a LocalScope<'expr>) -> Self {
        Self {
            state,
            scopes: Scopes::new(scope),
        }
    }

    pub fn reparent(&'a self, scope: &'a LocalScope<'expr>) -> Context<'a, 'expr> {
        Self {
            state: self.state,
            scopes: self.scopes.reparent(scope),
        }
    }

    // TODO: rename this.
    // It's not really creating a new scope but rather cloning the
    // existing scope to the used when evaluating a new node
    pub fn new_scope(&self) -> LocalScope<'expr> {
        self.scopes.scope.clone()
    }

    /// Lookup a value, if the value belongs to the state it returns a deferred value
    /// instead, to be resolved at a later stage.
    pub fn lookup(&self, path: &Path) -> ValueRef<'expr> {
        match self.scopes.lookup(path) {
            ValueRef::Empty => ValueRef::Deferred(path.clone()),
            val => val,
        }
    }

    // TODO maybe get rid of this if we can make the state return a collection 
    pub fn resolve_collection(&self, path: &Path, node_id: Option<&NodeId>) -> ValueRef<'_> {
        match self.scopes.lookup(path) {
            ValueRef::Empty => self.state.get(path, node_id),
            val => val,
        }
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
        let scope = LocalScope::new("value".into(), ValueRef::Str("hello world"));
        let mut scopes = Scopes::new(&scope);

        let inner_scope = LocalScope::new("value".into(), ValueRef::Str("inner hello"));
        let inner = scopes.reparent(&inner_scope);

        let ValueRef::Str(lhs) = inner.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, "inner hello");

        let ValueRef::Str(lhs) = scope.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, "hello world");
    }

    // #[test]
    // fn dynamic_attribute() {
    //     let mut state = TestState::new();
    //     let mut root = Scope::new(None);
    //     let ctx = Context::new(&mut state, &mut root);
    //     let mut attributes = Attributes::new();
    //     attributes.insert("name".to_string(), ValueExpr::Ident("name".into()));

    //     let id = Some(123.into());
    //     let name = ctx.attribute::<String>("name", id.as_ref(), &attributes);
    //     assert_eq!("Dirk Gently", name.value().unwrap());
    // }

    // #[test]
    // fn context_lookup() {
    //     let state = TestState::new();
    //     let scope = Scope::new(None);
    //     let context = Context::new(&state, &scope);

    //     let path = Path::from("inner").compose("name");
    //     let value = context.lookup_value::<String>(&path, None);
    //     let value: &str = value.value().unwrap();
    //     assert!(matches!(value, "Fiddle McStick"));
    // }

    // #[test]
    // fn singular_state_value() {
    //     let state = TestState::new();
    //     let scope = Scope::new(None);
    //     let context = Context::new(&state, &scope);
    //     let path = Path::from("inner").compose("name");
    // }

    // #[test]
    // fn collection_with_one_state_value() {
    //     let state = TestState::new();
    //     let scope = Scope::new(None);
    //     let context = Context::new(&state, &scope);
    // }
}
