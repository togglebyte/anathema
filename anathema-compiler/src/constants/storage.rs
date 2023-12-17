use anathema_values::Slab;

#[derive(Debug)]
pub struct Storage<T>(pub(crate) Slab<T>);

impl<T> Storage<T> {
    pub fn empty() -> Self {
        Self(Slab::empty())
    }

    // De-duplicate values.
    // If the value already exist, just return the value position,
    pub(crate) fn push(&mut self, value: T) -> usize
    where
        T: PartialEq,
    {
        self.0.find(&value).unwrap_or_else(|| self.0.push(value))
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        self.0.get(index)
    }
}
