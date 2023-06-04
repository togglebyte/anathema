use std::collections::HashMap;

use anathema_compiler::Constants;
use anathema_widgets::{Path, Value};

use crate::NodeGen;

fn value_by_path<'value>(
    value: &'value Value,
    path: &Path,
    consts: &Constants,
) -> Option<&'value Value> {
    match (path, value) {
        (Path::Index(index), Value::List(list)) => list.get(*index),
        (Path::Key(key), Value::Map(map)) => map.get(consts.lookup_ident(*key)?),
        (Path::Composite(left, right), Value::List(list)) => {
            let index = left.as_index()?;
            let value = list.get(index)?;
            value_by_path(value, right, consts)
        }
        (Path::Composite(left, right), Value::Map(map)) => {
            let key = left.as_key()?;
            let value = map.get(consts.lookup_ident(key)?)?;
            value_by_path(value, right, consts)
        }
        _ => None,
    }
}

pub fn make_context<'gen>(node_generators: &'gen [NodeGen]) -> Context<'gen> {
    let mut context = Context::new();
    context
}

/// VM execution context.
/// Contains all the values provided by the user.
pub struct Context<'gen> {
    inner: HashMap<&'gen str, Value>,
}

impl<'gen> Context<'gen> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::default(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.inner.get_mut(key)
    }

    pub fn insert(&mut self, key: &'gen str, value: Value) {
        self.inner.insert(key, value);
    }

    // /// Find a value by path.
    // /// If the value is either a `List` or a `Map` and the path
    // /// contains composite key, get the value recursively through `value_by_path`
    // pub fn lookup(&self, path: &Path) -> Option<&Value> {
    //     match path {
    //         Path::Composite(root, right) => {
    //             if Path::Root.ne(root) {
    //                 return None; // this is an invalid path
    //             }

    //             match right.as_ref() {
    //                 Path::Composite(left, right) => {
    //                     let key = left.as_key()?;
    //                     let val = self.inner.get(key)?;
    //                     value_by_path(val, right)
    //                 }
    //                 Path::Key(key) => self.inner.get(key),
    //                 _ => None,
    //             }
    //         }
    //         Path::Key(key) => self.inner.get(key),
    //         Path::Root | Path::Index(_) => None, // the root map can only take composite keys
    //     }
    // }

    // pub fn insert(&mut self, key: &str, value: impl Into<Value>) {
    //     self.inner_mut().insert(key.into(), value.into());
    // }
}

pub struct SubContext<'ctx> {
    root: &'ctx Context<'ctx>,
    scopes: Vec<(&'ctx str, &'ctx Value)>,
}

impl<'ctx> SubContext<'ctx> {
    pub fn new(root: &'ctx Context<'ctx>) -> Self {
        Self {
            root,
            scopes: vec![],
        }
    }

    pub fn get(&self, key: &str) -> Option<&'ctx Value> {
        let scoped = self
            .scopes
            .iter()
            .rev()
            .find_map(|(k, v)| (*k == key).then_some(*v));

        scoped.or_else(|| self.root.get(key))
    }

    pub fn push(&mut self, key: &'ctx str, value: &'ctx Value) {
        self.scopes.push((key, value))
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn value_lookup() {
        let mut ctx = Context::new();
        let value = Value::Bool(true);
        ctx.insert("value", value.clone());
        assert_eq!(ctx.get("value").cloned().unwrap(), value);
    }

    #[test]
    fn value_mutate() {
        let mut ctx = Context::new();
        ctx.insert("value", Value::Bool(true));
        let val = ctx.get_mut("value").unwrap();
        *val = Value::Bool(false);
        assert_eq!(ctx.get("value").cloned().unwrap(), Value::Bool(false));
    }
}
