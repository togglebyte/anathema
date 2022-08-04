use std::collections::hash_map::{Entry, HashMap};

use display::Color;
use widgets::{Align, Axis, BorderStyle, Display, Number, Path, Sides, Value, Wrap};

use crate::error::Result;
use crate::WidgetNode;

/// Caching includes
#[derive(Debug, Default)]
pub struct IncludeCache(HashMap<String, Vec<WidgetNode>>);

/// Track the include depth and maintain the include cache to prevent
/// multiple reads from disk.
#[derive(Debug)]
pub struct NodeCtx<'cache> {
    include_cache: &'cache mut IncludeCache,
    pub(crate) include_level: usize,
}

impl<'cache> NodeCtx<'cache> {
    /// Create a new instance of a `NodeCtx`.
    pub fn new(include_cache: &'cache mut IncludeCache) -> Self {
        Self { include_cache, include_level: 0 }
    }

    pub(crate) fn includes(&mut self, path: String) -> Result<Vec<WidgetNode>> {
        let entry = self.include_cache.0.entry(path);
        match entry {
            Entry::Occupied(o) => Ok(o.get().clone()),
            Entry::Vacant(v) => {
                let s = std::fs::read_to_string(v.key())?;
                let nodes = crate::parse(&s)?;
                Ok(v.insert(nodes).clone())
            }
        }
    }
}

/// A sub context for an inner scope.
/// `SubContext` wraps the parent context and the lookup order is as follows:
/// first the sub context, and if the value is not present then try the parent context.
#[derive(Debug)]
pub struct SubContext<'ctx> {
    inner: &'ctx DataCtx,
    sub: DataCtx,
}

impl<'ctx> SubContext<'ctx> {
    /// Create a new sub context with the given context as the parent context.
    pub fn new(ctx: &'ctx DataCtx) -> Self {
        Self { inner: ctx, sub: DataCtx::empty() }
    }

    pub fn insert(&mut self, key: &str, value: impl Into<Value>) {
        self.sub.insert(key, value);
    }

    /// Clone self and generate a new sub context with the new values
    pub fn sub(&self, key: &str, value: Value) -> Self {
        let mut sub = self.sub.clone();
        sub.insert(key, value);
        Self { inner: self.inner, sub }
    }

    /// Look up a value by `Path`
    pub fn by_path(&self, path: &Path) -> Option<&Value> {
        match self.sub.by_path(path) {
            Some(val) => Some(val),
            None => self.inner.by_path(path),
        }
    }
}

macro_rules! mut_ref_push_diff {
    ($fn:ident, $ret:tt, $variant:ident) => {
        /// Get a mutable reference to a `$ret`
        pub fn $fn(&mut self, key: &str) -> Option<&mut $ret> {
            match self.values.get_mut(key)? {
                Value::$variant(value) => {
                    self.diff.push(key.into());
                    Some(value)
                }
                _ => None,
            }
        }
    };
}

/// Contain values that are available inside templates.
#[derive(Debug, Clone, PartialEq)]
pub struct DataCtx {
    values: HashMap<String, Value>,
    diff: Vec<String>,
}

impl DataCtx {
    /// Create a new data context with a given key / value.
    pub fn with_value(key: &str, value: impl Into<Value>) -> Self {
        let mut ctx = Self { values: HashMap::new(), diff: Vec::new() };
        ctx.insert(key, value.into());
        ctx
    }

    /// Insert a hashmap
    pub fn insert_map(&mut self, map: HashMap<String, Value>) {
        for (key, value) in map {
            self.insert(&key, value);
        }
    }

    /// Lookup a value by key.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    /// Get a mutable reference to a value.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.values.get_mut(key)
    }

    /// Find a value by path.
    pub fn by_path(&self, path: &Path) -> Option<&Value> {
        let value = self.get(path.name.as_str())?;
        match value {
            Value::Map(_) if path.child.is_none() => Some(value),
            Value::Map(_) => {
                let path = path.child.as_ref()?;
                Value::lookup(path, value)
            }
            _ => Some(value),
        }
    }

    /// Create an empty data context.
    pub fn empty() -> Self {
        DataCtx { values: HashMap::new(), diff: Vec::new() }
    }

    /// Insert a key / value into the data context.
    /// This will add the key to the diff lookup.
    pub fn insert(&mut self, key: &str, value: impl Into<Value>) {
        let value = value.into();
        self.set(key, value);
        self.diff.push(key.into());
    }

    /// Set a value without generating a diff insert.
    pub fn set(&mut self, key: &str, value: impl Into<Value>) {
        let value = value.into();
        self.values.insert(key.to_string(), value);
    }

    /// Remove a value.
    /// This does not generate a diff.
    pub fn remove(&mut self, key: &str) {
        self.values.remove(key);
    }

    /// Drain the diffs and return a new `DataCtx` containing the differences.
    pub fn diff(&mut self) -> DataCtx {
        let mut ctx = DataCtx::empty();
        for key in self.diff.drain(..) {
            let value = match self.values.get(&key) {
                Some(v) => v.clone(),
                None => continue,
            };
            ctx.values.insert(key, value);
        }

        ctx
    }

    /// Is the context empty?
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn get_u64_mut(&mut self, key: &str) -> Option<&mut u64> {
        match self.values.get_mut(key)? {
            Value::Number(Number::Unsigned(num)) => {
                self.diff.push(key.into());
                Some(num)
            }
            _ => None,
        }
    }

    pub fn get_i64_mut(&mut self, key: &str) -> Option<&mut i64> {
        match self.values.get_mut(key)? {
            Value::Number(Number::Signed(num)) => {
                self.diff.push(key.into());
                Some(num)
            }
            _ => None,
        }
    }

    mut_ref_push_diff!(get_alignment_mut, Align, Alignment);
    mut_ref_push_diff!(get_axis_mut, Axis, Axis);
    mut_ref_push_diff!(get_bool_mut, bool, Bool);
    mut_ref_push_diff!(get_border_style_mut, BorderStyle, BorderStyle);
    mut_ref_push_diff!(get_color_mut, Color, Color);
    mut_ref_push_diff!(get_path_mut, Path, DataBinding);
    mut_ref_push_diff!(get_display_mut, Display, Display);
    mut_ref_push_diff!(get_sides_mut, Sides, Sides);
    mut_ref_push_diff!(get_string_mut, String, String);
    mut_ref_push_diff!(get_wrap_mut, Wrap, Wrap);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lookup_by_path() {
        let mut ctx = DataCtx::empty();
        let user: HashMap<String, Value> = HashMap::from([("name".to_string(), Value::String("bill".to_string()))]);
        ctx.insert("user", Value::Map(user));

        let mut path = Path::new("user");
        path.child = Some(Box::new(Path::new("name")));

        let value = ctx.by_path(&path).unwrap();
        let expected = Value::String("bill".into());
        let actual = value.clone();
        assert_eq!(expected, actual);
    }
}
