/// Reduced functionality vector, preventing invalid operations
/// where we depend on a stack
#[derive(Debug)]
pub struct Stack<T> {
    inner: Vec<T>,
}

impl<T> Stack<T>
where
    T: Copy,
{
    /// Create an empty stack
    pub const fn empty() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    /// Create a stack with an initial capacity.
    /// This will fill the stack with empty entries
    pub fn with_capacity(cap: usize) -> Self {
        let mut inner = Vec::with_capacity(cap);
        Self { inner }
    }

    /// Push a value onto the stack
    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    /// Pop a value off the stack
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    /// Create an iterator over the values on the stack
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> + '_ {
        self.inner.iter()
    }

    /// Create an iterator over the values on the stack
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut T> + '_ {
        self.inner.iter_mut()
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
    pub fn drain(&mut self) -> impl DoubleEndedIterator<Item = T> + '_ {
        self.inner.drain(..)
    }

    /// Clear the values from the stack
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Returns true if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Number of elements on the stack
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Drain all the values into another stack.
    /// Prefer `Self::drain_copy_into` if `T` is `Copy`.
    /// It might be marginally faster.
    pub fn drain_into(&mut self, local: &mut Stack<T>) {
        local.inner.extend(self.inner.drain(..));
    }
}

impl<T: PartialEq> Stack<T> {
    /// Check if the stack contains a given value
    pub fn contains(&self, value: &T) -> bool {
        self.inner.iter().any(|v| v == value)
    }
}

impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            inner: iter.into_iter().collect(),
        }
    }
}

impl<T: Copy> Default for Stack<T> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<T> From<Stack<T>> for Vec<T> {
    fn from(value: Stack<T>) -> Self {
        value.inner
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
