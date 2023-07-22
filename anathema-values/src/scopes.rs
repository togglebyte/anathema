use crate::bucket::Bucket;
use crate::path::PathId;
use crate::{Value, ValueRef};

#[derive(Clone)]
pub struct Scopes<T>(Vec<Scope<T>>);

impl<T> Scopes<T> {
    pub fn with_capacity(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    pub fn new() -> Self {
        Self(vec![])
    }

    fn get(&self, path: PathId) -> Option<&ValueRef<T>> {
        self.0
            .iter()
            .rev()
            .filter_map(|scope| scope.get(path))
            .next()
    }
}

#[derive(Debug, Clone)]
struct Scope<T>(Vec<(PathId, ValueRef<T>)>);

impl<T> Scope<T> {
    fn get(&self, path: PathId) -> Option<&ValueRef<T>> {
        self.0
            .iter()
            .filter_map(|(p, val)| (path.eq(p)).then_some(val))
            .next()
    }
}

// pub struct Context<'a, T: 'a> {
//     bucket: &'a Bucket<'a, T>,
//     scopes: ScopeValues<T>,
// }

// impl<'a, T: 'a> Context<'a, T> {
//     fn get(&self, path_id, PathId) -> Option<&T> {
//         self.scopes.get(path_id).or_else(|| self.bucket.get(value_ref))
//     }
// }
