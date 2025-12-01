use crate::slab::{Slab, SlabIndex};

#[derive(Debug, Copy, Clone)]
struct SparseIndex(u32);

/// Sparse slab.
///
/// Iterating over values is faster than a basic slab on account of all the values
/// are stored together.
///
/// Insert and remove is slower than a basic slab.
#[derive(Debug, Clone)]
pub struct SparseSlab<I, V> {
    value_stack: Vec<V>,
    slab_index_stack: Vec<I>,
    keys: Slab<I, SparseIndex>,
}

impl<I, V> SparseSlab<I, V>
where
    I: SlabIndex,
{
    /// Create a new empty sparse slab
    pub fn empty() -> Self {
        Self {
            value_stack: vec![],
            slab_index_stack: vec![],
            keys: Slab::empty(),
        }
    }

    /// Remove a value and return it
    pub fn remove(&mut self, index: I) -> Option<V> {
        let key = self.keys.remove(index);

        let inner_index = key.0 as usize;

        let value = self.value_stack.swap_remove(inner_index);
        let _ = self.slab_index_stack.swap_remove(inner_index);

        // update the index of the value / key that was the last one, which has now moved
        let index = self.slab_index_stack[inner_index];
        self.keys[index] = key;

        Some(value)
    }

    /// Insert a new value
    pub fn insert(&mut self, value: V) -> I {
        let key = SparseIndex(self.value_stack.len() as u32);
        let index = self.keys.insert(key);
        self.slab_index_stack.push(index);
        self.value_stack.push(value);
        index
    }

    /// Get a mutable reference to a value
    pub fn get_mut(&mut self, key: I) -> Option<&mut V> {
        let idx = self.keys.get(key).copied()?;
        Some(&mut self.value_stack[idx.0 as usize])
    }

    /// Iterate over all the values.
    /// This is the reason this type exists.
    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.value_stack.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert() {
        let mut sparse = SparseSlab::<u8, &str>::empty();
        let first = sparse.insert("first");

        let value = sparse.iter().next().copied().unwrap();
        assert_eq!(value, "first");
    }

    #[test]
    fn remove() {
        let mut sparse = SparseSlab::<u8, &str>::empty();
        let first = sparse.insert("first");
        let second = sparse.insert("second");
        let third = sparse.insert("third");
        sparse.remove(first);
        sparse.remove(third);

        assert_eq!(sparse.value_stack.len(), 1);
        assert_eq!(sparse.slab_index_stack.len(), 1);

        assert_eq!(sparse.value_stack[0], "second");
        assert_eq!(sparse.slab_index_stack[0], 1);
    }

    #[test]
    fn get_mut() {
        let mut sparse = SparseSlab::<u8, u32>::empty();
        let key = sparse.insert(0);

        assert_eq!(sparse.get_mut(key).copied().unwrap(), 0);
        *sparse.get_mut(key).unwrap() = 1;
        assert_eq!(sparse.get_mut(key).copied().unwrap(), 1);
    }
}
