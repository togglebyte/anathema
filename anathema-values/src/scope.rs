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

    pub fn scope(&mut self, path: impl Into<Path>, value: ScopeValue<'expr>) {
        self.0.insert(path.into(), value);
    }

    pub fn value(&mut self, path: impl Into<Path>, value: ValueRef<'expr>) {
        self.0.insert(path.into(), ScopeValue::Value(value));
    }

    pub fn deferred(&mut self, path: impl Into<Path>, expr: &'expr ValueExpr) {
        self.0.insert(path.into(), ScopeValue::Deferred(expr));
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
    pub internal_state: Option<&'state dyn State>,
    pub scopes: Scopes<'state, 'expr>,
}

impl<'state, 'expr> Context<'state, 'expr> {
    pub fn new(state: &'state dyn State, scope: &'state Scope<'expr>) -> Self {
        Self {
            state,
            internal_state: None,
            scopes: Scopes::new(scope),
        }
    }

    pub fn reparent(&'state self, scope: &'state Scope<'expr>) -> Context<'state, 'expr> {
        Self {
            state: self.state,
            internal_state: self.internal_state,
            scopes: self.scopes.reparent(scope),
        }
    }

    pub fn clone_scope(&self) -> Scope<'expr> {
        self.scopes.scope.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_value() {
        let mut scope = Scope::new();
        scope.value("value", ValueRef::Str("hello world"));
        let scopes = Scopes::new(&scope);

        let mut inner_scope = Scope::new();
        inner_scope.value("value", ValueRef::Str("inner hello"));

        {
            let scopes = scopes.reparent(&inner_scope);

            let &ScopeValue::Value(ValueRef::Str(lhs)) = scopes.lookup(&"value".into()).unwrap()
            else {
                panic!()
            };
            assert_eq!(lhs, "inner hello");
        }

        let &ScopeValue::Value(ValueRef::Str(lhs)) = scopes.lookup(&"value".into()).unwrap() else {
            panic!()
        };
        assert_eq!(lhs, "hello world");
    }
}
