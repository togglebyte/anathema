use crate::slab::{Slab, SlabIndex, Ticket};

pub mod strings;

pub struct Storage<I, K, V>(Slab<I, (K, V)>);

/// Simple storage backed by a slab, prevents duplicate values
/// and associate values with keys
impl<I, K, V> Storage<I, K, V>
where
    I: SlabIndex,
{
    /// Create an empty store
    pub const fn empty() -> Self {
        Self(Slab::empty())
    }

    /// De-duplicate values.
    /// If the key already exist, just return the index.
    ///
    /// # Note
    ///
    /// This will not overwrite the existing value.
    #[must_use]
    pub fn push(&mut self, key: impl Into<K>, value: impl Into<V>) -> I
    where
        K: PartialEq,
    {
        let value = value.into();
        let key = key.into();
        let index = self.0.iter().find(|(_, (k, _))| key.eq(k)).map(|(i, (_, _))| i);
        index.unwrap_or_else(|| self.0.insert((key, value)))
    }

    /// Insert a key and a value.
    /// If the key already exists the value will be overwritten
    #[must_use]
    pub fn insert(&mut self, key: impl Into<K>, value: impl Into<V>) -> I
    where
        K: PartialEq,
    {
        let value = value.into();
        let key = key.into();
        let index = self.0.iter().find(|(_, (k, _))| key.eq(k)).map(|(i, (_, _))| i);

        match index {
            Some(i) => {
                self.0.get_mut_unchecked(i).1 = value;
                i
            }
            None => self.0.insert((key, value)),
        }
    }

    pub fn checkout(&mut self, index: I) -> Ticket<I, (K, V)> {
        self.0.checkout(index)
    }

    pub fn restore(&mut self, ticket: Ticket<I, (K, V)>) {
        self.0.restore(ticket)
    }

    /// Get a reference by index
    pub fn get(&self, index: I) -> Option<&(K, V)> {
        self.0.get(index)
    }

    /// Get a mutable reference by index
    pub fn get_mut(&mut self, index: I) -> Option<&mut (K, V)> {
        self.0.get_mut(index)
    }

    pub fn index_by_key(&self, key: K) -> Option<I>
    where
        K: PartialEq,
    {
        self.0.iter().filter(|(_, (k, _))| key.eq(k)).map(|(i, _)| i).next()
    }

    /// Get a value by index assuming the value exists.
    ///
    /// # Panics
    ///
    /// If the value doesn't exist
    pub fn get_unchecked(&self, index: I) -> &(K, V) {
        self.0.get(index).expect("missing value")
    }

    pub fn remove(&mut self, index: I) -> Option<(K, V)> {
        self.0.try_remove(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = (I, &(K, V))> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut (K, V)> {
        self.0.iter_values_mut()
    }
}
