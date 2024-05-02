//! A [`SmallMap`] should only be used for a small amount of values.
//! It will use stack space to reduce the number of allocations as long as
//! the number of entries in the map are no more than `STACK_SIZE`.
//!
//! [`SmallMap`] should be used together with [`MapStack`] and [`SmallMapBuilder`].
//!
//! The goal of the small map is to store associated values with multiple entities.
//! For that reason the [`MapStack`] is used, as it can be filled and drained between creations of
//! small maps, thus reducing the number of allocations.
//!
//! A [`SmallMap`] is immutable and the only way to create one containing values is via the
//! [`SmallMapBuilder`].
use crate::stack::Stack;

#[derive(Debug, Default)]
enum Entry<K, V> {
    Occupied(K, V),
    #[default]
    Empty,
}

impl<K, V> Entry<K, V> {
    fn try_to(&self) -> Option<(&K, &V)> {
        match self {
            Entry::Occupied(key, val) => Some((key, val)),
            Entry::Empty => None,
        }
    }

    fn to_val(&self) -> &V {
        match self {
            Entry::Occupied(_, val) => val,
            Entry::Empty => unreachable!(),
        }
    }

    fn to_mut_val(&mut self) -> &mut V {
        match self {
            Entry::Occupied(_, val) => val,
            Entry::Empty => unreachable!(),
        }
    }

    fn to_key(&self) -> &K {
        match self {
            Entry::Occupied(key, _) => key,
            Entry::Empty => unreachable!(),
        }
    }
}

/// Map index
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SmallIndex(u8);

impl SmallIndex {
    /// Max
    pub const MAX: Self = SmallIndex(u8::MAX);
    /// One
    pub const ONE: Self = SmallIndex(1);
    /// Zero
    pub const ZERO: Self = SmallIndex(0);
}

impl From<SmallIndex> for u8 {
    fn from(value: SmallIndex) -> Self {
        value.0
    }
}

impl From<u8> for SmallIndex {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

const STACK_SIZE: usize = 4;

#[derive(Debug)]
enum SmallMapStore<K, V> {
    /// An empty map
    Empty,
    /// Stack based map
    Stack {
        /// Values in the map.
        data: [Entry<K, V>; STACK_SIZE],
        /// The number of valid entries in `data`.
        len: u8,
    },
    /// Heap based map
    Heap(Box<[(K, V)]>),
}

/// A small map used to store a small amount of values.
///
/// Note: Keys are not de-duped, therefore it is
/// possible to insert two different values with the same key.
///
/// One of the values won't be accessible.
/// ```
/// # use anathema_store::smallmap::*;
/// # use anathema_store::stack::Stack;
/// let mut stack = Stack::empty();
/// let mut builder = SmallMapBuilder::new(&mut stack);
/// builder.insert(|index| ("hello", "world"));
/// builder.insert(|index| ("a", "b"));
/// let map = builder.finish();
///
/// let index = map.get_index("hello").unwrap();
/// assert_eq!("world", *map.get(index).unwrap());
///
/// let index = map.get_index("a").unwrap();
/// assert_eq!("b", *map.get(index).unwrap());
/// ```
#[derive(Debug)]
pub struct SmallMap<K, V>(SmallMapStore<K, V>);

impl<K, V> SmallMap<K, V> {
    /// Get the index for a key.
    /// The index lookup will always be faster and should be
    /// preferred.
    pub fn get_index<Q>(&self, key: &Q) -> Option<SmallIndex>
    where
        K: PartialEq,
        K: std::borrow::Borrow<Q>,
        Q: ?Sized + PartialEq,
    {
        match &self.0 {
            SmallMapStore::Empty => None,
            SmallMapStore::Stack { len: 0, .. } => None,
            SmallMapStore::Stack { data, len } => data[..*len as usize]
                .iter()
                .enumerate()
                .map(|(i, e)| (i, e.to_key()))
                .find_map(|(i, key_str)| (key_str.borrow() == key).then_some(SmallIndex(i as u8))),
            SmallMapStore::Heap(vec) => vec
                .iter()
                .enumerate()
                .find_map(|(i, (key_str, _))| (key_str.borrow() == key).then_some(SmallIndex(i as u8))),
        }
    }

    /// Get a value out of the map
    pub fn get(&self, index: SmallIndex) -> Option<&V> {
        match &self.0 {
            SmallMapStore::Empty => None,
            SmallMapStore::Stack { data, len } if *len >= index.0 => Some(data[index.0 as usize].to_val()),
            SmallMapStore::Stack { .. } => None,
            SmallMapStore::Heap(vec) => Some(&vec[index.0 as usize].1),
        }
    }

    /// Get a value out of the map
    pub fn get_mut(&mut self, index: SmallIndex) -> Option<&mut V> {
        match &mut self.0 {
            SmallMapStore::Empty => None,
            SmallMapStore::Stack { data, len } if *len >= index.0 => Some(data[index.0 as usize].to_mut_val()),
            SmallMapStore::Stack { .. } => None,
            SmallMapStore::Heap(vec) => Some(&mut vec[index.0 as usize].1),
        }
    }

    /// Iterate over the key-value pairs of the map.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> + '_ {
        let mut stack = None;
        let mut heap = None;

        match &self.0 {
            SmallMapStore::Empty => {}
            SmallMapStore::Stack { data, len } => stack = Some(data[..*len as usize].iter().filter_map(Entry::try_to)),
            SmallMapStore::Heap(data) => heap = Some(data.iter().map(|(k, v)| (k, v))),
        }

        std::iter::from_fn(move || match &self.0 {
            SmallMapStore::Empty => None,
            SmallMapStore::Stack { .. } => stack.as_mut()?.next(),
            SmallMapStore::Heap(_) => heap.as_mut()?.next(),
        })
    }
}

/// Reusable memory for storing values for a [`SmallMap`] during the creation step.
pub type MapStack<K, V> = Stack<(K, V)>;

/// Create an instance of a [`SmallMap`] via the builder.
pub struct SmallMapBuilder<'stack, K, V> {
    key_values: &'stack mut MapStack<K, V>,
}

impl<'stack, K, V> SmallMapBuilder<'stack, K, V> {
    /// Create a new builder
    pub fn new(stack: &'stack mut Stack<(K, V)>) -> Self {
        Self { key_values: stack }
    }

    pub fn insert<F>(&mut self, f: F)
    where
        F: FnOnce(SmallIndex) -> (K, V),
    {
        let t = InsertTransaction::new(self);
        let (k, v) = f(t.index());
        t.commit(k, v);
    }

    pub fn transaction(&mut self) -> InsertTransaction<'_, 'stack, K, V> {
        InsertTransaction::new(self)
    }

    /// Consume the builder and produce a `SmallMap`.
    /// The values inside the map can be mutable, but the map itself is not.
    pub fn finish(self) -> SmallMap<K, V> {
        let len = self.key_values.len();
        let mut key_values = self.key_values.drain().rev();
        let store = match len {
            0 => SmallMapStore::Empty,
            len @ 1..=4 => SmallMapStore::Stack {
                data: std::array::from_fn(|_| {
                    key_values
                        .next()
                        .map(|(k, v)| Entry::Occupied(k, v))
                        .unwrap_or(Entry::Empty)
                }),
                len: len as u8,
            },
            5.. => SmallMapStore::Heap(key_values.collect::<Vec<_>>().into_boxed_slice()),
        };
        SmallMap(store)
    }
}

impl<K, V> Drop for SmallMapBuilder<'_, K, V> {
    fn drop(&mut self) {
        self.key_values.clear();
    }
}

pub struct InsertTransaction<'builder, 'stack, K, V> {
    index: SmallIndex,
    builder: &'builder mut SmallMapBuilder<'stack, K, V>,
}

impl<'builder, 'stack, K, V> InsertTransaction<'builder, 'stack, K, V> {
    fn new(builder: &'builder mut SmallMapBuilder<'stack, K, V>) -> Self {
        let index = builder.key_values.next_index();
        Self {
            index: SmallIndex(index as u8),
            builder,
        }
    }

    pub fn index(&self) -> SmallIndex {
        self.index
    }

    /// This will not de-duplicate keys
    pub fn commit(self, key: K, value: V) {
        self.builder.key_values.push((key, value));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn insert_twice() {
        let mut stack = Stack::empty();
        let mut builder = SmallMapBuilder::<&str, u32>::new(&mut stack);

        let tran = builder.transaction();
        assert_eq!(tran.index(), SmallIndex::ZERO);
        tran.commit("a", 123);

        let tran = builder.transaction();
        assert_eq!(tran.index(), SmallIndex::ONE);
        tran.commit("b", 2);
    }

    #[test]
    fn no_commit_then_commit() {
        let mut stack = Stack::empty();
        let mut builder = SmallMapBuilder::<&str, u32>::new(&mut stack);

        let tran = builder.transaction();
        assert_eq!(tran.index(), SmallIndex::ZERO);

        let tran = builder.transaction();
        assert_eq!(tran.index(), SmallIndex::ZERO);
        tran.commit("b", 2);
    }

    #[test]
    fn build_arr_map() {
        let mut stack = Stack::empty();
        let mut builder = SmallMapBuilder::<&str, u32>::new(&mut stack);
        builder.transaction().commit("a", 123);
        builder.transaction().commit("b", 999);

        let map = builder.finish();

        let a_index = map.get_index("a").unwrap();
        let b_index = map.get_index("b").unwrap();

        let a = map.get(a_index).unwrap();
        let b = map.get(b_index).unwrap();

        assert_eq!(*a, 123);
        assert_eq!(*b, 999);
        assert!(map.get(SmallIndex(99)).is_none());
    }

    #[test]
    fn build_heap_entry() {
        let mut stack = Stack::empty();
        let mut builder = SmallMapBuilder::<&str, u32>::new(&mut stack);
        builder.transaction().commit("a", 123);
        builder.transaction().commit("b", 124);
        builder.transaction().commit("c", 125);
        builder.transaction().commit("d", 126);
        builder.transaction().commit("e", 126);

        let map = builder.finish();

        let d_index = map.get_index("d").unwrap();

        let d = map.get(d_index).unwrap();

        assert_eq!(*d, 126);
    }
}
