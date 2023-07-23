use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};

pub(crate) type GenerationId = usize;

// -----------------------------------------------------------------------------
//   - Generation -
// -----------------------------------------------------------------------------
/// The generation of a values.
/// If the value is updated the generation should not change,
/// however if the value is removed and another values
/// takes its place then the generation has be incremented by one.
pub struct Generation<T> {
    pub(crate) gen: GenerationId,
    value: T,
}

impl<T> Generation<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { gen: 0, value }
    }

    pub(crate) fn next(gen: GenerationId, value: T) -> Self {
        Self { gen, value }
    }

    pub(crate) fn compare_generation(&self, gen: GenerationId) -> bool {
        self.gen == gen
    }
}

impl<T> Deref for Generation<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Generation<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Debug> Debug for Generation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Generation")
            .field("gen", &self.gen)
            .field("value", &self.value)
            .finish()
    }
}

impl<T: PartialEq> PartialEq<Self> for Generation<T> {
    fn eq(&self, other: &Self) -> bool {
        self.gen == other.gen && self.value == other.value
    }
}

impl<T: PartialEq> PartialEq<usize> for Generation<T> {
    fn eq(&self, other: &usize) -> bool {
        self.gen.eq(other)
    }
}
