use crate::Path;

/// A `Fragment` can be either a [`Path`] or a `String`.
/// `Fragment`s are usually part of a list to represent a single string value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Fragment {
    /// A string.
    String(String),
    /// A path to a value inside a context.
    Data(Path),
}

impl Fragment {
    /// Is the fragment a string?
    pub fn is_string(&self) -> bool {
        matches!(self, Fragment::String(_))
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
