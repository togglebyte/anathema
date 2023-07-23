use std::fmt;
use std::ops::Deref;

/// Paths are insert and fetch only.
/// Once a path is written into `Paths` it can 
/// not be removed.
pub(crate) struct Paths {
    inner: Vec<Path>,
}

impl Paths {
    pub(crate) fn new() -> Self {
        Self {
            inner: vec![],
        }
    }

    pub(crate) fn get_or_insert(&mut self, path: Path) -> PathId {
        match self.inner.iter().position(|p| path.eq(p)) {
            Some(p) => PathId(p),
            None => {
                let path_id = PathId(self.inner.len());
                self.inner.push(path);
                path_id
            }
        }
    }
}

impl From<Vec<Path>> for Paths {
    fn from(paths: Vec<Path>) -> Self {
        Self {
            inner: paths,
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

/// Path lookup
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathId(usize);

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

