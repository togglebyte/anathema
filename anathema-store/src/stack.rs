#[derive(Debug, Default, PartialEq, Clone, Copy)]
enum Entry<T> {
    Occupied(T),
    #[default]
    Empty,
}

impl<T> Entry<T> {
    fn to_value_ref(&self) -> Option<&T> {
        match self {
            Self::Occupied(val) => Some(val),
            Self::Empty => None,
        }
    }

    fn to_value_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Occupied(val) => Some(val),
            Self::Empty => None,
        }
    }

    fn into_value(self) -> Option<T> {
        match self {
            Self::Occupied(val) => Some(val),
            Self::Empty => None,
        }
    }
}

/// Allocate memory but never free it until the entire `Stack` is dropped.
/// Items popped from the stack are marked as `Empty` so the memory is reused.
#[derive(Debug, Default)]
pub struct Stack<T> {
    inner: Vec<Entry<T>>,
    len: usize,
}

impl<T> Stack<T> {
    /// Create an empty stack
    pub const fn empty() -> Self {
        Self {
            inner: Vec::new(),
            len: 0,
        }
    }

    /// Get the next index that will be written to
    pub fn next_index(&self) -> usize {
        self.len
    }

    /// Create a stack with an initial capacity.
    /// This will fill the stack with empty entries
    pub fn with_capacity(cap: usize) -> Self {
        let mut inner = Vec::with_capacity(cap);
        inner.fill_with(|| Entry::Empty);
        Self { inner, len: 0 }
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: T) {
        let mut entry = Entry::Occupied(value);
        if self.len < self.inner.len() {
            std::mem::swap(&mut entry, &mut self.inner[self.len]);
        } else {
            self.inner.push(entry);
        }
        self.len += 1;
    }

    /// Pop a value off the stack
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let mut entry = Entry::Empty;
        self.len -= 1;
        std::mem::swap(&mut entry, &mut self.inner[self.len]);
        let value = entry
            .into_value()
            .expect("the length would be zero if there wasn't a value present");
        Some(value)
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let entry = self.inner.get(index)?;
        match entry {
            Entry::Occupied(val) => Some(val),
            Entry::Empty => None,
        }
    }

    /// Swap out a value in the stack at a given location.
    ///
    /// # Panics
    ///
    /// Panics if the index contains an empty slot
    #[must_use]
    pub fn swap(&mut self, index: usize, new_value: T) -> T {
        let mut entry = Entry::Occupied(new_value);
        std::mem::swap(&mut self.inner[index], &mut entry);
        match entry {
            Entry::Occupied(val) => val,
            Entry::Empty => panic!("tried to take value from an empty entry"),
        }
    }

    /// Create an iterator over the values on the stack
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + '_ {
        self.inner[..self.len].iter().filter_map(Entry::to_value_ref)
    }

    /// Create an iterator over the values on the stack
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + '_ {
        self.inner[..self.len].iter_mut().filter_map(Entry::to_value_mut)
    }

    /// A draining iterator over the values on the stack.
    /// ```
    /// # use anathema_store::stack::Stack;
    /// let mut stack = Stack::empty();
    /// stack.push(1);
    /// stack.push(2);
    ///
    /// assert_eq!(stack.drain().next(), Some(2));
    /// assert!(stack.is_empty());
    /// ```
    pub fn drain(&mut self) -> StackDrain<T, impl DoubleEndedIterator<Item = T> + '_> {
        let len = std::mem::take(&mut self.len);
        let iter = self.inner[..len]
            .iter_mut()
            .rev()
            .filter_map(|e| match std::mem::take(e) {
                Entry::Occupied(value) => Some(value),
                Entry::Empty => unreachable!(),
            });

        StackDrain { inner: iter, len }
    }

    /// Clear the values from the stack
    pub fn clear(&mut self) {
        if self.is_empty() {
            return;
        }
        self.inner[..self.len].fill_with(|| Entry::Empty);
        self.len = 0;
    }

    /// The stack will contains allocated memory even if `is_empty` returns true.k
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn reserve(&mut self, len: usize) {
        if self.len >= len {
            return;
        }

        self.inner.resize_with(len, || Entry::Empty);
    }

    /// Drain all the values into another stack.
    /// Prefer `Self::drain_copy_into` if `T` is `Copy`.
    /// It might be marginally faster.
    pub fn drain_into(&mut self, local: &mut Stack<T>) {
        if self.is_empty() {
            return;
        }
        local.reserve(self.len);
        self.drain().rev().for_each(|ent| local.push(ent));
    }
}

impl<T: PartialEq> Stack<T> {
    /// Check if the stack contains a given value
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|v| v == value)
    }
}

impl<T: Copy> Stack<T> {
    /// Drain the values into another stack.
    /// This function can be marginally faster than `Self::drain_into` but
    /// depends on `T` being `Copy`.
    pub fn drain_copy_into(&mut self, local: &mut Stack<T>) {
        if self.is_empty() {
            return;
        }
        local.reserve(self.len);
        local.len = self.len;
        local.inner[..self.len].copy_from_slice(&self.inner[..self.len]);
        self.clear();
    }
}

impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = iter.into_iter().map(|val| Entry::Occupied(val)).collect::<Vec<_>>();

        Self {
            len: inner.len(),
            inner,
        }
    }
}

/// A draining iterator over the stack.
/// Any values that wasn't consumed will be dropped
/// along with the iterator.
pub struct StackDrain<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    inner: I,
    len: usize,
}

impl<T, I> StackDrain<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T, I> Iterator for StackDrain<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<T, I> DoubleEndedIterator for StackDrain<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<T, I> Drop for StackDrain<T, I>
where
    I: DoubleEndedIterator<Item = T>,
{
    fn drop(&mut self) {
        self.for_each(|_| {});
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn drain() {
        let mut stack = Stack::empty();
        stack.push(1);
        stack.push(2);

        let mut iter = stack.drain();
        assert_eq!(2, iter.next().unwrap());
        drop(iter);

        assert_eq!(stack.inner, vec![Entry::Empty, Entry::Empty]);
    }
}
