use crate::bucket::Bucket;
use crate::path::PathId;
use crate::slab::Slab;
use crate::{Value, ValueRef};

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

    pub(crate) fn insert(&mut self, path_id: PathId, value: ValueRef<T>, scope: Option<ScopeId>) {
        let scope = scope
            .and_then(|id| self.scopes.get_mut(id.0))
            .unwrap_or_else(|| &mut self.root);

        panic!("what is it that we are actually supposed to insert here?");
    }

    pub fn remove(&mut self, scope_id: impl Into<ScopeId>) {
        self.scopes.remove(scope_id.into().0);
    }

    fn get(&self, path: PathId, scope_id: Option<ScopeId>) -> Option<ValueRef<T>> {
        let scope = scope_id.and_then(|id| self.scopes.get(id.0));

        scope
            .and_then(|scope| scope.get(path))
            .or_else(|| self.root.get(path))
            .copied()
    }
}

#[derive(Debug, Clone)]
struct Scope<T>(Vec<(PathId, ValueRef<T>)>);

impl<T> Scope<T> {
    fn new() -> Self {
        Self(vec![])
    }

    fn get(&self, path: PathId) -> Option<&ValueRef<T>> {
        self.0
            .iter()
            .filter_map(|(p, val)| (path.eq(p)).then_some(val))
            .next()
    }

    fn insert(&mut self, path: PathId, value: ValueRef<T>) {
        self.0.push((path, value));
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
    fn add_value_to_stack() {
        // let expected = ValueRef::<()>::new(0, 0);
        // let path = PathId::from(0);
        // let mut scopes = Scopes::new();
        // let scope_id = scopes.new_scope();
        // scopes.insert(path, expected);

        // let actual = scopes.get(path).unwrap();
        // assert_eq!(expected, actual);
    }
}
