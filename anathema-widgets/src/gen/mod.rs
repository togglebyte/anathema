use crate::Value;

pub(crate) mod expressions;
pub(crate) mod generator;
pub(crate) mod index;
mod scope;
pub(crate) mod store;

#[cfg(test)]
pub mod testing;

#[derive(Debug)]
pub enum ValueRef<'parent> {
    Owned(Value),
    Borrowed(&'parent Value),
}

impl<'parent> ValueRef<'parent> {
    pub fn value(&self) -> Option<&Value> {
        match self {
            Self::Borrowed(val) => Some(val),
            Self::Owned(val) => Some(val),
        }
    }

    pub fn borrowed(&self) -> Option<&'parent Value> {
        match self {
            Self::Borrowed(val) => Some(val),
            Self::Owned(_) => None,
        }
    }
}

impl<'parent> From<&'parent Value> for ValueRef<'parent> {
    fn from(val: &'parent Value) -> Self {
        ValueRef::Borrowed(val)
    }
}
