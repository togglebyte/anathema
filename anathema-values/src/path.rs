use std::fmt;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;

fn next() -> PathId {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed).into()
}

/// Path lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PathId(pub usize);

impl From<usize> for PathId {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl Deref for PathId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


/// Paths are insert and fetch only.
/// Once a path is written into `Paths` it should never be removed
/// as the index in the `Vec<Path>` is the path id
pub struct Paths {
    inner: HashMap<Path, PathId>,
}

impl Paths {
    pub(crate) fn empty() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub(crate) fn get(&self, path: &Path) -> Option<PathId> {
        self.inner.get(path).copied()
    }

    pub(crate) fn get_or_insert(&mut self, path: Path) -> PathId {
        *self.inner.entry(path).or_insert_with(next)
    }
}

impl From<Vec<Path>> for Paths {
    fn from(paths: Vec<Path>) -> Self {
        Self {
            inner: paths.into_iter().map(|p| (p, next())).collect(),
        }
    }
}

// // Values can only come from the supplied value,
// // meaning the supplied value is either a vector of values or a hashmap
// fn composite_value_lookup<'a, 'b: 'a>(path: &'a Path, value: &'b Value) -> Option<&'b Value> {
//     match path {
//         Path::Index(index) => value.to_slice().map(|v| &v[*index]),
//         Path::Key(key) => value.to_map()?.get(key),
//         Path::Composite(left, right) => {
//             let inner = composite_value_lookup(left, value)?;
//             composite_value_lookup(right, inner)
//         }
//     }
// }

// -----------------------------------------------------------------------------
//   - Value path -
//   The path to a value in a given context.
//
//   Key     Key    Key
//   parent .child .name
//
//   Key               Index   Key
//   parent_collection .3     .name
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum Path {
    /// The key is an index to an ident inside `Constants`
    Key(String),
    /// Index in a collection
    Index(usize),
    /// Composite key, made up by two or more keys
    Composite(Box<Path>, Box<Path>),
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => write!(f, "K({})", key)?,
            Self::Index(index) => write!(f, "I({})", index)?,
            Self::Composite(left, right) => {
                left.fmt(f)?;
                write!(f, " -> ")?;
                right.fmt(f)?;
            }
        }

        Ok(())
    }
}

impl Path {
    // pub fn lookup_value<'parent>(&self, values: &Values<'parent>) -> Option<&'parent Value> {
    //     match self {
    //         Self::Key(key) => values.get_borrowed_value(key.as_str()),
    //         Self::Composite(left, right) => {
    //             let left = left.lookup_value(values)?;
    //             composite_value_lookup(right, left)
    //         }
    //         _ => None,
    //     }
    // }
}

impl Path {
    pub fn compose(&self, child: impl Into<Path>) -> Self {
        match self {
            Self::Key(_) | Self::Index(_) => {
                Self::Composite(Box::new(self.clone()), Box::new(child.into()))
            }
            Self::Composite(left, right) => {
                Self::Composite(left.clone(), Box::new(right.compose(child.into())))
            }
        }
    }
}

impl From<usize> for Path {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        Self::Key(s.into())
    }
}

impl From<String> for Path {
    fn from(s: String) -> Self {
        Self::Key(s)
    }
}
