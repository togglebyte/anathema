use std::sync::Arc;

use crate::hashmap::{HashMap, IntMap};
use crate::path::PathId;
use crate::slab::Slab;
use crate::{Container, ValueRef};

pub enum ScopeValue<T> {
    Static(Arc<T>),
    Dyn(ValueRef<Container<T>>),
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

pub struct Scopes<T> {
    root: Scope<T>,
    scopes: Slab<Scope<T>>,
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

    pub(crate) fn new_scope(&mut self, parent_id: ScopeId) -> ScopeId {
        self.scopes.push(Scope::new(parent_id)).into()
    }

    pub(crate) fn insert(
        &mut self,
        path_id: PathId,
        value: ScopeValue<T>,
        scope: impl Into<Option<ScopeId>>,
    ) {
        let scope = scope
            .into()
            .and_then(|id| self.scopes.get_mut(id.0))
            .unwrap_or_else(|| &mut self.root);

        scope.insert(path_id, value);
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
        scope
            .and_then(|scope| scope.get(path))
            .or_else(|| self.root.get(path))
    }
}

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

    fn new(parent: ScopeId) -> Self {
        Self {
            values: Default::default(),
            parent: Some(parent),
        }
    }

    fn get(&self, path: PathId) -> Option<&ScopeValue<T>> {
        self.values.get(&path.0)
    }

    fn insert(&mut self, path: PathId, value: ScopeValue<T>) {
        self.values.insert(path.0, value);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_value_to_root() {
        let expected = ValueRef::<()>::new(0, 0);
        let path = PathId::from(0);
        let mut scopes = Scopes::new();
        scopes.insert(path, expected, None);

        let actual = scopes.get(path, None).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn add_value_to_inner_scope() {
        let outer = ValueRef::<()>::new(0, 0);
        let path = PathId::from(0);
        let mut scopes = Scopes::new();
        scopes.insert(path, outer, None);

        // The block is only here to express
        // inner scope to the reader of the test.
        // It has no significance on the scope what so ever
        {
            let inner = ValueRef::<()>::new(1, 0);
            let scope_id = scopes.new_scope(None);
            scopes.insert(path, inner, scope_id);
            let actual = scopes.get(path, scope_id).unwrap();
            assert_eq!(inner, actual);
        }

        let actual = scopes.get(path, None).unwrap();
        assert_eq!(outer, actual);
    }

    #[test]
    fn scope_from_scope() {
        let val = ValueRef::<()>::new(0, 0);
        let mut scopes = Scopes::new();
        scopes.insert(0.into(), val, None);

        let depth_1 = scopes.new_scope(None);
        let value_ref_1 = ValueRef::<()>::new(1, 0);
        scopes.insert(0.into(), value_ref_1, depth_1);

        let depth_2 = scopes.new_scope(None);
        scopes.insert(0.into(), ValueRef::<()>::new(2, 0), depth_2);

        let depth_1_1 = scopes.new_scope(depth_1);
        let value_ref_2 = ValueRef::<()>::new(3, 0);
        scopes.insert(1.into(), value_ref_2, depth_1_1);

        assert_eq!(value_ref_1, scopes.get(0.into(), depth_1_1).unwrap());
        assert_eq!(value_ref_2, scopes.get(1.into(), depth_1_1).unwrap());
    }

    #[test]
    fn remove_scope() {
        let path = PathId::from(0);

        let mut scopes = Scopes::new();

        let scope_id = scopes.new_scope(None);
        scopes.insert(path, ValueRef::<()>::new(1, 0), scope_id);
        let actual = scopes.get(path, scope_id);
        assert!(actual.is_some());
        scopes.remove(scope_id);

        let actual = scopes.get(path, scope_id);
        assert!(actual.is_none());
    }
}
