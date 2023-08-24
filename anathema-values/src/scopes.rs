use std::sync::Arc;

use crate::hashmap::{HashMap, IntMap};
use crate::path::PathId;
use crate::slab::Slab;
use crate::{Container, ValueRef};

#[derive(Debug, PartialEq)]
pub enum ScopeValue<T> {
    Dyn(ValueRef<Container<T>>),
    Static(Arc<T>),
    List(Box<[ScopeValue<T>]>),
}

impl<T> Clone for ScopeValue<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(val) => Self::Static(val.clone()),
            Self::Dyn(val) => Self::Dyn(*val),
            Self::List(list) => Self::List(list.clone()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct ScopeId(usize);

impl From<usize> for ScopeId {
    fn from(idx: usize) -> Self {
        Self(idx)
    }
}

#[derive(Debug)]
pub struct Scopes<T> {
    root: Scope<T>,
    scopes: Slab<Scope<T>>,
}

impl<T> Default for Scopes<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Scopes<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            root: Scope::root(),
            scopes: Slab::with_capacity(cap),
        }
    }

    pub fn new() -> Self {
        Self {
            root: Scope::root(),
            scopes: Slab::empty(),
        }
    }

    pub(crate) fn new_scope(&mut self, parent_id: impl Into<Option<ScopeId>>) -> ScopeId {
        self.scopes.push(Scope::new(parent_id.into())).into()
    }

    pub(crate) fn insert(
        &mut self,
        path_id: PathId,
        value: ScopeValue<T>,
        scope: impl Into<Option<ScopeId>>,
    ) -> Option<ScopeValue<T>> {
        let scope = scope
            .into()
            .and_then(|id| self.scopes.get_mut(id.0))
            .unwrap_or_else(|| &mut self.root);

        scope.insert(path_id, value)
    }

    pub fn remove(&mut self, scope_id: impl Into<ScopeId>) {
        let _ = self.scopes.remove(scope_id.into().0);
    }

    pub(crate) fn get(
        &self,
        path: PathId,
        scope_id: impl Into<Option<ScopeId>>,
    ) -> Option<&ScopeValue<T>> {
        let scope = scope_id.into().and_then(|id| self.scopes.get(id.0));

        match scope {
            Some(scope) => scope.get(path).or_else(|| self.get(path, scope.parent)),
            None => self.root.get(path),
        }
    }
}

#[derive(Debug, Default)]
struct Scope<T> {
    values: IntMap<ScopeValue<T>>,
    parent: Option<ScopeId>,
}

impl<T> Scope<T> {
    fn root() -> Self {
        Self {
            values: Default::default(),
            parent: None,
        }
    }

    fn new(parent: Option<ScopeId>) -> Self {
        Self {
            values: Default::default(),
            parent,
        }
    }

    fn get(&self, path: PathId) -> Option<&ScopeValue<T>> {
        self.values.get(&path.0)
    }

    fn insert(&mut self, path: PathId, value: ScopeValue<T>) -> Option<ScopeValue<T>> {
        self.values.insert(path.0, value)
    }

    pub(crate) fn remove_dyn(&mut self, path: PathId) -> Option<ValueRef<Container<T>>> {
        match self.values.remove(&path.0) {
            Some(ScopeValue::Dyn(value_ref)) => Some(value_ref),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_value_to_root() {
        let expected = ScopeValue::Static(().into());
        let path = PathId::from(0);
        let mut scopes = Scopes::new();
        scopes.insert(path, expected.clone(), None);

        let actual = scopes.get(path, None).unwrap();
        assert_eq!(expected, *actual);
    }

    #[test]
    fn add_value_to_inner_scope() {
        let outer = ScopeValue::Static(0.into());
        let path = PathId::from(0);
        let mut scopes = Scopes::new();
        scopes.insert(path, outer.clone(), None);

        // The block is only here to express
        // inner scope to the reader of the test.
        // It has no significance on the scope what so ever
        {
            let inner = ScopeValue::Static(1.into());
            let scope_id = scopes.new_scope(None);
            scopes.insert(path, inner.clone(), scope_id);
            let actual = scopes.get(path, scope_id).unwrap();
            assert_eq!(inner, *actual);
        }

        let actual = scopes.get(path, None).unwrap();
        assert_eq!(outer, *actual);
    }

    #[test]
    fn scope_from_scope() {
        let val = ScopeValue::Static(0.into());
        let mut scopes = Scopes::new();
        // Root scope insert
        scopes.insert(0.into(), val, None);

        // Scope 1
        let scope_1 = scopes.new_scope(None);
        let val_1 = ScopeValue::Static(1.into());
        scopes.insert(0.into(), val_1.clone(), scope_1);

        // Scope 2
        let scope_2 = scopes.new_scope(None);
        scopes.insert(0.into(), ScopeValue::Static(3.into()), scope_2);

        // Scope 1.1
        let scope_1_1 = scopes.new_scope(scope_1);
        let val_2 = ScopeValue::Static(3.into());
        scopes.insert(1.into(), val_2.clone(), scope_1_1);

        assert_eq!(val_1, *scopes.get(0.into(), scope_1_1).unwrap());
        assert_eq!(val_2, *scopes.get(1.into(), scope_1_1).unwrap());
    }

    #[test]
    fn remove_scope() {
        let path = PathId::from(0);

        let mut scopes = Scopes::new();

        let scope_id = scopes.new_scope(None);
        scopes.insert(path, ScopeValue::Static(1.into()), scope_id);
        let actual = scopes.get(path, scope_id);
        assert!(actual.is_some());
        scopes.remove(scope_id);

        let actual = scopes.get(path, scope_id);
        assert!(actual.is_none());
    }
}
