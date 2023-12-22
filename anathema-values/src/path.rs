use std::fmt::{self, Display};
use std::ops::{Deref, Sub};

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

impl Display for PathId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<pid({})>", self.0)
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

    // Strip the first part of the path away.
    // This is used when looking up values from maps and lists
    // declared in the template.
    pub(super) fn sub(&self, rhs: &Path) -> Option<&Path> {
        if let Path::Composite(l_lhs, l_rhs) = self {
            match (&**l_lhs, rhs) {
                (Path::Key(l_key), Path::Key(r_key)) if l_key == r_key => Some(l_rhs),
                (Path::Index(l_idx), Path::Index(r_idx)) if l_idx == r_idx => Some(l_rhs),
                _ => None,
            }
        } else {
            None
        }
    }

    pub(super) fn rhs(&self) -> Option<&Path> {
        if let Path::Composite(_, rhs) = self {
            Some(rhs)
        } else {
            None
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
