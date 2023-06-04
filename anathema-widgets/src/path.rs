use std::{fmt, borrow::Cow};

use crate::{Fragment, Value, gen::store::Store};


// Values can only come from the supplied value,
// meaning the supplied value is either a vector of values or a hashmap
fn composite_value_lookup<'a, 'b: 'a>(path: &'a Path, value: &'b Value) -> Option<&'b Value> {
    match path {
        Path::Index(index) => value.to_slice().map(|v| &v[*index]),
        Path::Key(key) => value.to_map()?.get(key),
        Path::Composite(left, right) => {
            let inner = composite_value_lookup(left, value)?;
            composite_value_lookup(right, inner)
        }
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
    // TODO: when do we need the index? If not let's bin it, because 
    // it's causing grief over at the tokenizer (a.1.2 = a, fullstop, float(1.2))
    Index(usize),
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
    pub fn lookup_value<'parent>(&self, values: &Store<'parent>) -> Option<&'parent Value> {
        match self {
            Self::Key(key) => values.get_borrowed(key.as_str()),
            Self::Composite(left, right) => {
                let left = left.lookup_value(values)?;
                composite_value_lookup(right, left)
            }
            _ => None,
        }
    }
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

// -----------------------------------------------------------------------------
//   - Text path -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextPath {
    String(String),
    Fragments(Vec<Fragment>),
}

#[cfg(test)]
impl TextPath {
    pub fn fragment(key: &str) -> Self {
        Self::Fragments(vec![Fragment::Data(Path::Key(key.into()))])
    }
}

impl<T: Into<String>> From<T> for TextPath {
    fn from(s: T) -> Self {
        TextPath::String(s.into())
    }
}
