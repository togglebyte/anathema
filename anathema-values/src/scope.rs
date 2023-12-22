use std::rc::Rc;

use crate::hashmap::HashMap;
use crate::state::State;
use crate::{Path, ValueExpr, ValueRef};

// Scopes can only borrow values with the same lifetime as an expressions.
// Any scoped value that belongs to or contains state can only be scoped as deferred expressions.
#[derive(Debug, Clone, Copy)]
pub enum ScopeValue<'expr> {
    Value(ValueRef<'expr>),
    Deferred(&'expr ValueExpr),
    DeferredList(usize, &'expr ValueExpr),
}

#[derive(Debug, Clone)]
pub struct Scope<'expr>(HashMap<Path, ScopeValue<'expr>>);

impl<'expr> Scope<'expr> {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    fn lookup(&self, lookup_path: &Path) -> Option<&ScopeValue<'expr>> {
        self.0.get(lookup_path)
    }

    pub fn scope(&mut self, path: Path, value: ScopeValue<'expr> ) {
        self.0.insert(path, value);
    }

    pub fn value(&mut self, path: Path, value: ValueRef<'expr>) {
        self.0.insert(path, ScopeValue::Value(value));
    }

    pub fn deferred(&mut self, path: Path, expr: &'expr ValueExpr) {
        self.0.insert(path, ScopeValue::Deferred(expr));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Scopes<'a, 'expr> {
    parent: Option<&'a Scopes<'a, 'expr>>,
    scope: &'a Scope<'expr>,
}

impl<'a, 'expr> Scopes<'a, 'expr> {
    fn new(scope: &'a Scope<'expr>) -> Self {
        Self {
            scope,
            parent: None,
        }
    }

    /// Make the current scope the parent scope, and assign a new scope
    /// as the top most scope.
    pub fn reparent(&self, scope: &'a Scope<'expr>) -> Scopes<'_, 'expr> {
        Scopes {
            scope,
            parent: Some(self),
        }
    }

    pub(crate) fn lookup(&self, lookup_path: &Path) -> Option<&ScopeValue<'expr>> {
        self.scope
            .lookup(lookup_path)
            .or_else(|| self.parent.and_then(|p| p.lookup(lookup_path)))
    }
}

// Lookup is done elsewhere as the resolver affects the lifetime, therefore there
// is no lookup function on the scope.
//
// For a deferred resolver everything has the same lifetime as the expressions,
// for an immediate resolver the lifetime can only be that of the frame, during the layout step
#[derive(Debug)]
pub struct Context<'state, 'expr> {
    pub state: &'state dyn State,
    pub scopes: Scopes<'state, 'expr>,
}

impl<'state, 'expr> Context<'state, 'expr> {
    pub fn root(state: &'state dyn State, scope: &'state Scope<'expr>) -> Self {
        Self::new(state, scope)
    }

    pub fn new(state: &'state dyn State, scope: &'state Scope<'expr>) -> Self {
        Self {
            state,
            scopes: Scopes::new(scope),
        }
    }

    pub fn reparent(&'state self, scope: &'state Scope<'expr>) -> Context<'state, 'expr> {
        Self {
            state: self.state,
            scopes: self.scopes.reparent(scope),
        }
    }

    // pub(crate) fn lookup_path(&self, path: &Path) -> ValueRef<'expr> {
    //     match self.scopes.lookup(path) {
    //         ValueRef::Empty => ValueRef::Deferred(path.clone()),
    //         val => val,
    //     }
    // }

    // TODO: rename this.
    // It's not really creating a new scope but rather cloning the
    // existing scope to be used when evaluating a new node
    pub fn new_scope(&self) -> Scope<'expr> {
        self.scopes.scope.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_value() {
        let scope = ScopeValue::value("value".into(), ValueRef::Str("hello world"));
        let scopes = Scopes::new(&scope);

        let inner_scope = ScopeValue::value("value".into(), ValueRef::Str("inner hello"));
        let inner = scopes.reparent(&inner_scope);

        let &ScopeValue::Value {
            value: ValueRef::Str(lhs),
            ..
        } = inner.lookup(&"value".into())
        else {
            panic!()
        };
        assert_eq!(lhs, "inner hello");

        let &ScopeValue::Value {
            value: ValueRef::Str(lhs),
            ..
        } = scopes.lookup(&"value".into())
        else {
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
