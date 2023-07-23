use crate::path::PathId;
use crate::slab::Slab;
use crate::ValueRef;
use crate::hashmap::{HashMap, IntMap};

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
            root: Scope::new(),
            scopes: Slab::with_capacity(cap),
        }
    }

    pub fn new() -> Self {
        Self {
            root: Scope::new(),
            scopes: Slab::empty(),
        }
    }

    pub(crate) fn new_scope(&mut self) -> ScopeId {
        self.scopes.push(Scope::new()).into()
    }

    pub(crate) fn insert(
        &mut self,
        path_id: PathId,
        value: ValueRef<T>,
        scope: impl Into<Option<ScopeId>>,
    ) {
        let scope = scope
            .into()
            .and_then(|id| self.scopes.get_mut(id.0))
            .unwrap_or_else(|| &mut self.root);

        scope.insert(path_id, value);
    }

    pub fn remove(&mut self, scope_id: impl Into<ScopeId>) {
        self.scopes.remove(scope_id.into().0);
    }

    pub(crate) fn get(
        &self,
        path: PathId,
        scope_id: impl Into<Option<ScopeId>>,
    ) -> Option<ValueRef<T>> {
        let scope = scope_id.into().and_then(|id| self.scopes.get(id.0));

        scope
            .and_then(|scope| scope.get(path))
            .or_else(|| self.root.get(path))
            .copied()
    }
}

#[derive(Debug, Clone)]
struct Scope<T>(HashMap<usize, ValueRef<T>>);

impl<T> Scope<T> {
    fn new() -> Self {
        Self(Default::default())
    }

    fn get(&self, path: PathId) -> Option<&ValueRef<T>> {
        self.0.get(&path.0)
    }

    fn insert(&mut self, path: PathId, value: ValueRef<T>) {
        self.0.insert(path.0, value);
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
            let scope_id = scopes.new_scope();
            scopes.insert(path, inner, scope_id);
            let actual = scopes.get(path, scope_id).unwrap();
            assert_eq!(inner, actual);
        }

        let actual = scopes.get(path, None).unwrap();
        assert_eq!(outer, actual);
    }

    #[test]
    fn remove_scope() {
        let path = PathId::from(0);

        let mut scopes = Scopes::new();

        let scope_id = scopes.new_scope();
        scopes.insert(path, ValueRef::<()>::new(1, 0), scope_id);
        let actual = scopes.get(path, scope_id);
        assert!(actual.is_some());
        scopes.remove(scope_id);

        let actual = scopes.get(path, scope_id);
        assert!(actual.is_none());
    }
}
