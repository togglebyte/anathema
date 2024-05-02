use std::mem::swap;
use std::ops::Deref;
use std::rc::Rc;

/// An element stored in a generational slab.
#[derive(Debug, PartialEq)]
pub struct Element<T>(Rc<Option<T>>);

impl<T> Deref for Element<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        (*self.0)
            .as_ref()
            .expect("the inner `Option<T>` is always `Some(T)` by the time an `Element<T>` is created")
    }
}

// -----------------------------------------------------------------------------
//   - Entry -
//   Both vacant and occupied use the same Rc to prevent additional allocations
// -----------------------------------------------------------------------------
enum Entry<I, T> {
    Pending,
    // The `Rc<Option<T>>` should always be `None`
    Vacant(Option<I>, Rc<Option<T>>),
    // The `Rc<Option<T>>` should always be `Some(T)`
    Occupied(Rc<Option<T>>),
}

impl<I, T> Entry<I, T> {
    // Insert an Occupied entry in place of a vacant one.
    fn swap(&mut self, inner_value: T) {
        debug_assert!(matches!(self, Entry::Vacant(..)));

        let mut entry = Entry::Pending;
        swap(&mut entry, self);

        match entry {
            Entry::Vacant(_, mut storage_cell) => {
                Rc::get_mut(&mut storage_cell)
                    .expect("Rc strong count is always one here")
                    .replace(inner_value);

                swap(self, &mut Entry::Occupied(storage_cell));
            }
            _ => unreachable!(),
        }
    }

    // Try to make the entry vacant and return the value
    fn try_evict(&mut self, next_id: &mut Option<I>) -> Option<T> {
        // If the strong count is anything but 1, then return None
        if let Entry::Occupied(value) = self {
            if Rc::strong_count(value) != 1 {
                return None;
            }
        }

        let mut value = Entry::Pending;
        swap(&mut value, self);

        match value {
            Entry::Occupied(mut store) => {
                let value = Rc::get_mut(&mut store)
                    .expect("strong count is always one")
                    .take()
                    .expect("occupied variant never contains a None");
                swap(self, &mut Entry::Vacant(next_id.take(), store));
                Some(value)
            }
            _ => unreachable!(),
        }
    }

    // Create a new occupied entry
    fn allocate_occupied(value: T) -> Self {
        Self::Occupied(Rc::new(Some(value)))
    }
}

// -----------------------------------------------------------------------------
//   - Rc backed slab -
// -----------------------------------------------------------------------------
/// Similar to a basic `Slab`, however each value is reference counted.
/// When removing a value from the slab the `Rc` is retained so as to reduce
/// additional allocations.
pub struct RcSlab<I, T> {
    next_id: Option<I>,
    inner: Vec<Entry<I, T>>,
}

impl<I, T> RcSlab<I, T>
where
    I: Copy,
    I: From<usize>,
    I: Into<usize>,
{
    /// Create an empty slab
    pub const fn empty() -> Self {
        Self {
            next_id: None,
            inner: vec![],
        }
    }

    /// This will clone the underlying Rc.
    /// Unlike the `Slab` the `RcSlab` needs the values to be
    /// manually removed with `try_remove`.
    pub fn get(&mut self, index: I) -> Option<Element<T>> {
        match self.inner.get(index.into())? {
            Entry::Occupied(value) => Some(Element(value.clone())),
            _ => None,
        }
    }

    // If there is a `self.next_id` then `take` the id (making it None)
    // and replace the vacant entry at the given index.
    //
    // Write the vacant entry's `next_id` into self.next_id, and
    // finally replace the vacant entry with the occupied value
    /// Insert a value into the slab
    pub fn insert(&mut self, value: T) -> I {
        match self.next_id.take() {
            Some(index) => {
                let entry = &mut self.inner[index.into()];

                let Entry::Vacant(new_next_id, _) = entry else {
                    unreachable!("you found a bug with Anathema, please file a bug report")
                };

                self.next_id = new_next_id.take();
                entry.swap(value);
                index
            }
            None => {
                self.inner.push(Entry::allocate_occupied(value));
                (self.inner.len() - 1).into()
            }
        }
    }

    /// Take a value out of the slab.
    ///
    /// # Panics
    ///
    /// Will panic if the slot is not occupied.
    pub fn try_remove(&mut self, index: I) -> Option<T> {
        let value = self.inner[index.into()].try_evict(&mut self.next_id);
        if value.is_some() {
            self.next_id = Some(index);
        }
        value
    }

    /// Iterator over the keys and elements
    pub fn iter(&self) -> impl Iterator<Item = (I, &T)> + '_ {
        self.inner.iter().enumerate().filter_map(|(i, e)| match e {
            Entry::Occupied(val) => val.as_ref().as_ref().map(|val| (i.into(), val)),
            Entry::Vacant(_, _) | Entry::Pending => None,
        })
    }
}

impl<I, T> RcSlab<I, T>
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
                Entry::Pending => writeln!(&mut s, "{idx}: pending"),
                Entry::Vacant(next, value) => {
                    let count = Rc::strong_count(value);
                    let _ = write!(&mut s, "{idx}: value: {value:?} | count: {count} | ");
                    match next {
                        Some(i) => writeln!(&mut s, "next id: {}", (*i).into()),
                        None => writeln!(&mut s, "no next id"),
                    }
                }
                Entry::Occupied(value) => writeln!(&mut s, "{idx}: {value:?} | count: {}", Rc::strong_count(value)),
            };
        }

        let _ = writeln!(&mut s, "---- next id ----");

        let _ = match self.next_id {
            Some(i) => {
                writeln!(&mut s, "next id: {}", i.into())
            }
            None => writeln!(&mut s, "no next id"),
        };

        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn try_remove_value() {
        let mut slab = RcSlab::<usize, _>::empty();
        let index = slab.insert("I has a value");

        {
            // `_hold_value` holds the underlying value,
            // therefore trying to remove it will return `None`
            let _hold_value = slab.get(index);
            assert!(slab.try_remove(index).is_none());
        }

        assert!(slab.try_remove(index).is_some());
    }

    #[test]
    fn ensure_rc_resuse() {
        let mut slab = RcSlab::<usize, _>::empty();

        // Add and remove the value to ensure there is
        // an unused `Rc<T>` inside the slab
        let index = slab.insert(123);

        // Get a pointer to the (now) vacant `Rc`
        let ptr_a = {
            let Entry::Occupied(rc) = &slab.inner[0] else { panic!() };
            Rc::as_ptr(rc)
        };
        assert!(slab.try_remove(index).is_some());

        // ... then insert a value and make sure the value exists
        slab.insert(456);
        let Entry::Occupied(value) = &slab.inner[0] else { panic!() };
        // and get a pointer to the `Rc`
        let ptr_b = Rc::as_ptr(value);

        // Compare the two pointers and ensure they are the same
        assert_eq!(ptr_a, ptr_b);
    }

    #[test]
    fn push_multi() {
        let mut slab = RcSlab::<usize, usize>::empty();
        let idx1 = slab.insert(1);
        let idx2 = slab.insert(2);
        let idx3 = slab.insert(3);

        assert_eq!(0, idx1);
        assert_eq!(1, idx2);
        assert_eq!(2, idx3);
    }

    #[test]
    fn clones() {
        let mut slab = RcSlab::<usize, usize>::empty();
        // strong count of 1
        let idx1 = slab.insert(1);

        {
            // strong count of 3
            let _val1 = slab.get(idx1);
            let _val2 = slab.get(idx1);
            let Entry::Occupied(val) = &slab.inner[0] else { panic!() };
            assert_eq!(Rc::strong_count(val), 3);
        } // drop all the clones resetting the strong count to 1

        let Entry::Occupied(val) = &slab.inner[0] else { panic!() };
        // Ensure it actually is one
        assert_eq!(Rc::strong_count(val), 1);
    }
}
