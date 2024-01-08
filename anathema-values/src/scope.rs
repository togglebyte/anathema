use std::fmt::Debug;

use crate::hashmap::HashMap;
use crate::state::State;
use crate::{NodeId, Path, ValueExpr, ValueRef};

// TODO: technically the `InnerContext` is acting more as a scope
//       and the `Scope` is just a wrapper around the storage.
//       This could perhaps benefit from being renamed.

// Scopes can only borrow values with the same lifetime as an expressions.
// Any scoped value that belongs to or contains state can only be scoped as deferred expressions.
#[derive(Debug, Clone, Copy)]
pub enum ScopeValue<'expr> {
    Value(ValueRef<'expr>),
    Deferred(&'expr ValueExpr),
    DeferredList(usize, &'expr ValueExpr),
}

#[derive(Clone, Debug)]
pub struct ScopeStorage<'expr>(HashMap<Path, ScopeValue<'expr>>);

impl<'expr> ScopeStorage<'expr> {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    fn get(&self, lookup_path: &Path) -> Option<ScopeValue<'expr>> {
        self.0.get(lookup_path).copied()
    }

    pub fn insert(&mut self, path: impl Into<Path>, value: ScopeValue<'expr>) {
        self.0.insert(path.into(), value);
    }

    pub fn value(&mut self, path: impl Into<Path>, value: ValueRef<'expr>) {
        self.insert(path, ScopeValue::Value(value));
    }

    pub fn deferred(&mut self, path: impl Into<Path>, expr: &'expr ValueExpr) {
        self.insert(path, ScopeValue::Deferred(expr));
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Scope<'frame, 'expr> {
    store: &'frame ScopeStorage<'expr>,
    parent: Option<&'frame Scope<'frame, 'expr>>,
}

impl<'frame, 'expr> Scope<'frame, 'expr> {
    fn get(&self, lookup_path: &Path) -> Option<ScopeValue<'expr>> {
        self.store
            .get(lookup_path)
            .or_else(|| self.parent.and_then(|p| p.get(lookup_path)))
    }
}

#[derive(Copy, Clone)]
struct InnerContext<'frame, 'expr> {
    state: &'frame dyn State,
    scope: Option<&'frame Scope<'frame, 'expr>>,
    parent: Option<&'frame InnerContext<'frame, 'expr>>,
}

impl<'frame, 'expr> InnerContext<'frame, 'expr> {
    fn pop(&self) -> Option<&Self> {
        self.parent
    }
}

impl Debug for InnerContext<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InnterContext")
            .field("state", &"<state>")
            .field("scope", &self.scope)
            .field("parent", &self.parent)
            .finish()
    }
}

// Lookup is done elsewhere as the resolver affects the lifetime, therefore there
// is no lookup function on the scope.
//
// For a deferred resolver everything has the same lifetime as the expressions,
// for an immediate resolver the lifetime can only be that of the frame, during the layout step
#[derive(Copy, Clone, Debug)]
pub struct Context<'frame, 'expr> {
    inner: InnerContext<'frame, 'expr>,
}

impl<'frame, 'expr> Context<'frame, 'expr> {
    pub fn root(state: &'frame dyn State) -> Self {
        Self {
            inner: InnerContext {
                state,
                scope: None,
                parent: None,
            },
        }
    }

    pub fn with_scope(&self, scope: &'frame Scope<'frame, 'expr>) -> Self {
        let inner = InnerContext {
            state: self.inner.state,
            parent: self.inner.parent,
            scope: Some(scope),
        };

        Self { inner }
    }

    pub fn with_state(&'frame self, state: &'frame dyn State) -> Self {
        let inner = InnerContext {
            state,
            parent: Some(&self.inner),
            scope: None,
        };

        Self { inner }
    }

    pub fn new_scope(&self, store: &'frame ScopeStorage<'expr>) -> Scope<'frame, 'expr> {
        Scope {
            store,
            parent: self.inner.scope,
        }
    }

    // TODO: rename this
    pub fn lookup(&'frame self) -> ContextRef<'frame, 'expr> {
        ContextRef { inner: &self.inner }
    }

    pub fn clone_scope(&self) -> ScopeStorage<'expr> {
        match self.inner.scope {
            None => ScopeStorage::new(),
            Some(scope) => scope.store.clone(),
        }
    }
}

pub struct ContextRef<'frame, 'expr> {
    inner: &'frame InnerContext<'frame, 'expr>,
}

impl<'frame, 'expr> ContextRef<'frame, 'expr> {
    pub fn pop(&self) -> Option<Self> {
        Some(Self {
            inner: self.inner.pop()?,
        })
    }

    pub fn state(&self, path: &Path, node_id: &NodeId) -> ValueRef<'frame> {
        self.inner.state.state_get(path, node_id)
    }

    pub fn scope(&self, path: &Path) -> Option<ScopeValue<'expr>> {
        let scope = self.inner.scope?;
        scope.get(path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn scope_value() {
        let mut store = ScopeStorage::new();
        store.value("value", ValueRef::Str("hello world"));
        let scope = Scope {
            store: &store,
            parent: None,
        };

        {
            let mut store = ScopeStorage::new();
            store.value("value", ValueRef::Str("inner hello"));
            let scope = Scope {
                store: &store,
                parent: Some(&scope),
            };

            let ScopeValue::Value(ValueRef::Str(lhs)) = scope.get(&"value".into()).unwrap() else {
                panic!()
            };
            assert_eq!(lhs, "inner hello");
        }

        let ScopeValue::Value(ValueRef::Str(lhs)) = scope.get(&"value".into()).unwrap() else {
            panic!()
        };

        assert_eq!(lhs, "hello world");
    }
}
