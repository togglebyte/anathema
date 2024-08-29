use super::Ticket;

// -----------------------------------------------------------------------------
//   - Entry -
// -----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Clone)]
enum Entry<I, T> {
    Vacant(Option<I>),
    Occupied(T),
    CheckedOut(I),
}

impl<I, T> Entry<I, T> {
    // Insert an Occupied entry in place of a vacant one.
    fn swap(&mut self, value: T) {
        debug_assert!(matches!(self, Entry::Vacant(_)));
        std::mem::swap(self, &mut Entry::Occupied(value));
    }

    // Create a new occupied entry
    fn occupied(value: T) -> Self {
        Self::Occupied(value)
    }

    // Will panic if the entry is vacant.
    // An entry should never be vacant where this call is involved.
    //
    // This means this method should never be used outside of update calls.
    fn as_occupied_mut(&mut self) -> &mut T {
        match self {
            Entry::Occupied(value) => value,
            Entry::Vacant(_) | Entry::CheckedOut(_) => unreachable!("invalid state"),
        }
    }
}

// -----------------------------------------------------------------------------
//   - Slab -
// -----------------------------------------------------------------------------
/// A basic slab
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Slab<I, T> {
    next_id: Option<I>,
    inner: Vec<Entry<I, T>>,
}

impl<I, T> Slab<I, T>
where
    I: Copy,
    I: From<usize>,
    I: Into<usize>,
    I: PartialEq,
{
    /// Create an empty slab
    pub const fn empty() -> Self {
        Self {
            next_id: None,
            inner: vec![],
        }
    }

    // If there is a `self.next_id` then `take` the id (making it None)
    // and replace the vacant entry at the given index.
    //
    // Write the vacant entry's `next_id` into self.next_id, and
    // finally replace the vacant entry with the occupied value
    /// Insert a value into the slab, returning the index
    pub fn insert(&mut self, value: T) -> I {
        match self.next_id.take() {
            Some(index) => {
                let entry = &mut self.inner[index.into()];

                let Entry::Vacant(new_next_id) = entry else {
                    unreachable!("you found a bug with Anathema, please file a bug report")
                };

                self.next_id = new_next_id.take();
                entry.swap(value);
                index
            }
            None => {
                self.inner.push(Entry::occupied(value));
                (self.inner.len() - 1).into()
            }
        }
    }

    /// Insert a value at a given index.
    /// This will force the underlying storage to grow if
    /// the index given is larger than the current capacity.
    ///
    /// This will overwrite any value currently at that index.
    ///
    /// # Panics
    ///
    /// Panics if a value is inserted at a position that is currently checked out
    pub fn insert_at(&mut self, index: I, value: T) {
        let idx = index.into();

        // If the index is outside of the current
        // length then fill the slots in between with
        // vacant entries
        if idx >= self.inner.len() {
            for i in self.inner.len()..idx {
                let entry = Entry::Vacant(self.next_id.take());
                self.next_id = Some(i.into());
                self.inner.push(entry);
            }
            self.inner.push(Entry::Occupied(value));
        // If the index is inside the current length:
        } else {
            let entry = self
                .inner
                .get_mut(idx)
                .expect("there should be entries up to self.len()");

            match entry {
                Entry::CheckedOut(_) => panic!("value is checked out"),
                Entry::Vacant(None) => *entry = Entry::Occupied(value),
                Entry::Occupied(val) => *val = value,
                &mut Entry::Vacant(Some(next_free)) => {
                    // Find the values that points to `index`
                    // and replace that with `next_free`

                    let mut next_id = &mut self.next_id;
                    loop {
                        match next_id {
                            Some(id) if *id == index => {
                                *id = next_free;
                                break;
                            }
                            Some(id) => {
                                let idx: usize = (*id).into();
                                match self.inner.get_mut(idx) {
                                    Some(Entry::Vacant(id)) => {
                                        next_id = id;
                                        continue;
                                    }
                                    Some(Entry::Occupied(_)) => {
                                        unreachable!("entry is occupied, so this should never be the next value")
                                    }
                                    Some(Entry::CheckedOut(_)) => unreachable!("entry checked out"),
                                    None => unreachable!("the index can only point to a vacant value"),
                                }
                            }
                            None => todo!(),
                        }
                    }

                    // Insert new value
                    self.inner[idx] = Entry::Occupied(value);
                }
            }
        }
    }

    /// Get the next id.
    ///
    /// # Warning
    ///
    /// There is no guarantee that this value will be the same
    /// value produced when doing an insert if another insert has happened
    /// since this value was returned.
    pub fn next_id(&self) -> I {
        match self.next_id {
            Some(id) => id,
            None => I::from(self.inner.len()),
        }
    }

    /// Removes a value out of the slab.
    /// This assumes the value exists
    ///
    /// # Panics
    /// Will panic if the slot is not occupied
    pub fn remove(&mut self, index: I) -> T {
        let mut entry = Entry::Vacant(self.next_id.take());
        self.next_id = Some(index);
        std::mem::swap(&mut self.inner[index.into()], &mut entry);

        match entry {
            Entry::Occupied(val) => val,
            Entry::Vacant(_) | Entry::CheckedOut(_) => panic!("removal of vacant entry"),
        }
    }

    /// Removes a value out of the slab.
    ///
    /// # Panics
    ///
    /// Will panic if the slot is not occupied
    pub fn try_remove(&mut self, index: I) -> Option<T> {
        let old = self.inner.get_mut(index.into())?;

        match old {
            Entry::Occupied(_) => {
                let mut entry = Entry::Vacant(self.next_id.take());
                std::mem::swap(old, &mut entry);
                self.next_id = Some(index);
                let Entry::Occupied(val) = entry else { unreachable!() };
                Some(val)
            }
            Entry::Vacant(_) => None,
            Entry::CheckedOut(_) => panic!("value is in use"),
        }
    }

    /// Removes a value out of the slab.
    ///
    /// # Panics
    ///
    /// Will panic if the slot is not occupied
    pub fn remove_if<F>(&mut self, index: I, f: F) -> Option<T>
    where
        F: Fn(&T) -> bool,
    {
        let old = self.inner.get_mut(index.into())?;

        match old {
            Entry::Occupied(val) => {
                if !f(val) {
                    return None;
                }

                let mut entry = Entry::Vacant(self.next_id.take());
                std::mem::swap(old, &mut entry);
                self.next_id = Some(index);
                let Entry::Occupied(val) = entry else { unreachable!() };
                Some(val)
            }
            Entry::Vacant(_) => None,
            Entry::CheckedOut(_) => panic!("value is in use"),
        }
    }

    /// Try to replace an existing value with a new value.
    /// Unlike [`Self::replace`] this function will not panic
    /// if the value does not exist
    pub fn try_replace(&mut self, index: I, mut new_value: T) -> Option<T> {
        match &mut self.inner[index.into()] {
            Entry::Occupied(value) => {
                std::mem::swap(value, &mut new_value);
                Some(new_value)
            }
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        }
    }

    /// Replace an existing value with a new one.
    ///
    /// # Panics
    ///
    /// Will panic if there is no value at the given index.
    pub fn replace(&mut self, index: I, mut new_value: T) -> T {
        let value = self.inner[index.into()].as_occupied_mut();
        std::mem::swap(value, &mut new_value);
        new_value
    }

    /// Get a reference to a value
    pub fn get(&self, index: I) -> Option<&T> {
        match self.inner.get(index.into())? {
            Entry::Occupied(val) => Some(val),
            _ => None,
        }
    }

    /// Get a mutable reference to a value
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        match self.inner.get_mut(index.into())? {
            Entry::Occupied(val) => Some(val),
            _ => None,
        }
    }

    /// Check out a value from the slab.
    /// The value has to be manually returned using `Self::restore`.
    ///
    /// It's up to the developer to remember to do this
    ///
    /// # Panics
    ///
    /// This will panic if a value does not at exist at the given key
    pub fn checkout(&mut self, key: I) -> Ticket<I, T> {
        let mut entry = Entry::CheckedOut(key);
        std::mem::swap(&mut entry, &mut self.inner[key.into()]);

        match entry {
            Entry::Occupied(value) => Ticket { value, key },
            Entry::CheckedOut(_) => panic!("value already checked out"),
            _ => panic!("no entry maching the key"),
        }
    }

    /// Restore a value that is currently checked out.
    ///
    /// # Panics
    ///
    /// This will panic if a value does not at exist at the given key,
    /// or if the value is not currently checked out
    pub fn restore(&mut self, Ticket { value, key }: Ticket<I, T>) {
        let mut entry = Entry::Occupied(value);
        std::mem::swap(&mut entry, &mut self.inner[key.into()]);

        match entry {
            Entry::CheckedOut(_) => (),
            _ => panic!("failed to return checked out value"),
        }
    }

    /// # Panics
    ///
    /// Will panic if the value does not exist
    pub fn get_mut_unchecked(&mut self, index: I) -> &mut T {
        match self.inner.get_mut(index.into()) {
            Some(Entry::Occupied(val)) => val,
            _ => panic!("no slot at index {}", index.into()),
        }
    }

    /// Be aware that this will only ever be as performant as
    /// the underlying vector if all entries are occupied.
    ///
    /// E.g if the only slot occupied is 1,000,000, then this will
    /// iterate over 1,000,000 entries to get there.
    pub fn iter_values(&self) -> impl Iterator<Item = &T> + '_ {
        self.inner.iter().filter_map(|e| match e {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant(_) => None,
            Entry::CheckedOut(_) => None,
        })
    }

    /// Be aware that this will only ever be as performant as
    /// the underlying vector if all entries are occupied.
    ///
    /// E.g if the only slot occupied is 1,000,000, then this will
    /// iterate over 1,000,000 entries to get there.
    pub fn iter_values_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.inner.iter_mut().filter_map(|e| match e {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }

    /// Iterator over the keys and elements
    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> + '_ {
        self.inner.iter().enumerate().filter_map(|(i, e)| match e {
            Entry::Occupied(val) => Some((i.into(), val)),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }

    /// Consume all the values in the slab and resets the next id.
    /// This does not replace occupied entries with vacant ones,
    /// but rather drain the underlying storage.
    pub fn consume(&mut self) -> impl Iterator<Item = T> + '_ {
        self.next_id = None;
        self.inner.drain(..).filter_map(|e| match e {
            Entry::Occupied(val) => Some(val),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }

    /// Mutable iterator over the keys and elements
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (I, &mut T)> + '_ {
        self.inner.iter_mut().enumerate().filter_map(|(i, e)| match e {
            Entry::Occupied(val) => Some((i.into(), val)),
            Entry::Vacant(_) | Entry::CheckedOut(_) => None,
        })
    }
}

impl<I, T> Slab<I, T>
where
    I: Copy,
    I: From<usize>,
    I: Into<usize>,
    T: std::fmt::Debug,
{
    #[doc(hidden)]
    pub fn dump_state(&self) -> String {
        use std::fmt::Write;

        let mut s = String::new();

        for (idx, value) in self.inner.iter().enumerate() {
            let _ = match value {
                Entry::Vacant(next) => {
                    let _ = write!(&mut s, "{idx}: vacant ");
                    match next {
                        Some(i) => writeln!(&mut s, "next id: {}", (*i).into()),
                        None => writeln!(&mut s, "no next id"),
                    }
                }
                Entry::Occupied(value) => writeln!(&mut s, "{idx}: {value:?}"),
                Entry::CheckedOut(_) => writeln!(&mut s, "entry is checked out"),
            };
        }

        let _ = writeln!(&mut s, "---- next id ----");

        let _ = match self.next_id {
            Some(i) => writeln!(&mut s, "next id: {}", i.into()),
            None => writeln!(&mut s, "no next id"),
        };

        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push() {
        let mut slab = Slab::<usize, _>::empty();
        let index = slab.insert(123);
        let val = slab.remove(index);
        assert_eq!(val, 123);
    }

    #[test]
    fn take() {
        let mut slab = Slab::<usize, _>::empty();
        let index_1 = slab.insert(1);
        let _ = slab.remove(index_1);
        let index_2 = slab.insert(2);
        assert_eq!(index_1, index_2)
    }

    #[test]
    fn update() {
        let mut slab = Slab::<usize, _>::empty();
        let index_1 = slab.insert("hello world");
        slab.replace(index_1, "updated");
        let s = slab.remove(index_1);
        assert_eq!(s, "updated");
    }

    #[test]
    fn insert_at_with_no_prior_allocations() {
        let mut slab = Slab::<usize, &str>::empty();
        slab.insert_at(1, "hello");
        assert_eq!(Some(0), slab.next_id);
        assert!(matches!(slab.inner[0], Entry::Vacant(None)));
        assert_eq!(slab.inner[1], Entry::Occupied("hello"));
    }

    #[test]
    fn insert_at_with_prior_allocations() {
        let mut slab = Slab::<usize, &str>::empty();
        slab.insert("a");
        slab.insert("b");
        slab.insert("c");

        // Free order: [1, 2, 0]
        slab.remove(0);
        slab.remove(2);
        slab.remove(1);

        assert_eq!(Some(1), slab.next_id);
        assert_eq!(Entry::Vacant(Some(2)), slab.inner[1]);
        assert_eq!(Entry::Vacant(Some(0)), slab.inner[2]);

        // Free order: [2, 0]
        slab.insert_at(1, "x");

        assert_eq!(Some(2), slab.next_id);
        assert_eq!(Entry::Vacant(Some(0)), slab.inner[2]);
    }
}
