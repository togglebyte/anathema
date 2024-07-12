use std::ops::{Index, IndexMut, Range};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct SliceIndex(u32);

#[derive(Debug, Default, Copy, Clone)]
/// Region access into a char buffer
struct SessionKey {
    start: u32,
    end: u32,
}

impl SessionKey {
    fn as_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

#[derive(Debug)]
pub struct Session<'a, T> {
    buffer: &'a mut Buffer<T>,
}

impl<'a, T: Copy> Session<'a, T> {
    /// Create and return a new session key from the current session.
    /// This means the current session is now pointing to the end of the buffer.
    #[must_use]
    pub fn next_slice(&mut self) -> SliceIndex {
        self.buffer.next_slice()
    }

    /// Insert a value into the buffer
    pub fn insert(&mut self, pos: usize, value: T) {
        self.buffer.insert(pos, value);
    }

    /// Reference to the last value in the buffer, regardless
    /// of where the session is pointing.
    pub fn last(&self) -> Option<&T> {
        self.buffer.last()
    }

    /// Push a value to the buffer.
    ///
    /// # Panics
    ///
    /// This will panic if there is no slice keys in the buffer.
    /// One can be created with `self.next_slice()`
    pub fn push(&mut self, value: T) {
        self.buffer.push(value)
    }

    /// Pop a value from the buffer.
    /// This is fine as a session is always referring to the end of the
    /// underlying buffer.
    pub fn pop(&mut self) -> Option<T> {
        self.buffer.pop()
    }

    /// Extend the buffer with the contents from the iterator
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        self.buffer.extend(iter);
    }

    /// Remove N elements from the end of the buffer
    pub fn tail_drain(&mut self, size: u32) {
        self.buffer.tail_drain(size);
    }

    /// Get a slice of data from the underlying buffer
    pub fn slice(&self, index: SliceIndex) -> &[T] {
        self.buffer.get(index)
    }

    /// Get a slice of mutable data from the underlying buffer
    pub fn slice_mut(&mut self, index: SliceIndex) -> &mut [T] {
        self.buffer.get_mut(index)
    }

    /// Length of the session buffer, not the total buffer
    pub fn len(&self) -> u32 {
        self.buffer.len()
    }

    /// It's empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// See [`Buffer::truncate`]
    pub fn truncate(&mut self, key: SliceIndex, index: usize) {
        self.buffer.truncate(key, index);
    }
}

impl<'a, T: Copy> Index<usize> for Session<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.buffer.buf.index(index)
    }
}

impl<'a, T: Copy> IndexMut<usize> for Session<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.buffer.buf.index_mut(index)
    }
}

/// A buffer of copy values with a max size of `u32::MAX`.
/// Make sure to interreact with the buffer through a session
/// when writing and reading from the buffer:
/// ```
/// # use anathema_store::buffer::Buffer;
/// let mut buffer = Buffer::empty();
/// let mut session = buffer.new_session();
/// let key = session.next_slice();
///
/// session.extend([1, 2, 3]);
///
/// assert_eq!(buffer.get(key), &[1, 2, 3]);
/// ```
///
/// and use the `SliceIndex` to access the buffer.
#[derive(Debug)]
pub struct Buffer<T> {
    buf: Vec<T>,
    keys: Vec<SessionKey>,
}

impl<T: Copy> Buffer<T> {
    /// Create an empty buffer.
    pub fn empty() -> Self {
        Self {
            buf: Vec::new(),
            keys: Vec::new(),
        }
    }

    fn push(&mut self, value: T) {
        assert!(!self.keys.is_empty(), "tried to push to a buffer without a slice key");
        self.buf.push(value);
        let buf_len = self.buf.len();
        let index = self.keys.len() - 1;
        self.keys[index].end = buf_len as u32;
    }

    fn pop(&mut self) -> Option<T> {
        assert!(!self.keys.is_empty(), "tried to pop from a buffer without a slice key");
        let index = self.keys.len() - 1;
        let output = self.buf.pop();
        if output.is_some() {
            self.keys[index].end -= 1;
        }
        output
    }

    /// Insert a value into the buffer.
    /// This will cause all the keys update after the
    fn insert(&mut self, index: usize, value: T) {
        assert!(index < u32::MAX as usize);

        if self.buf.len() == index {
            self.push(value);
            return;
        }

        self.buf.insert(index, value);

        // Find the slice key where the insert happens.
        // Since the inserts are most likely happening
        // at the end of the buffer, it makes sense to search
        // backwards for the slice index and then subtract that
        // position from the last index.
        let last_key_index = self.keys.len() - 1;
        let Some(key_index) = self
            .keys
            .iter_mut()
            .rev()
            .position(|key| key.as_range().contains(&index))
            .map(|pos| last_key_index - pos)
        else {
            return;
        };

        // Increment the length of the slice key where
        // the insert happened...
        self.keys[key_index].end += 1;

        // ... and offset all the subsequent keys by one
        self.keys[key_index + 1..].iter_mut().for_each(|key| {
            key.start += 1;
            key.end += 1;
        });
    }

    fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        self.buf.extend(iter);
        let buf_len = self.buf.len();

        if !self.keys.is_empty() {
            let index = self.keys.len() - 1;
            self.keys[index].end = buf_len as u32;
        }
    }

    /// Create a new session that can be converted into
    /// a session key, with access to the underlying storage
    /// written to by the session.
    pub fn new_session(&mut self) -> Session<'_, T> {
        Session { buffer: self }
    }

    fn next_slice(&mut self) -> SliceIndex {
        let slice = SliceIndex(self.keys.len() as u32);
        let len = self.buf.len() as u32;
        let key = SessionKey { start: len, end: len };
        self.keys.push(key);
        slice
    }

    pub fn get(&self, index: SliceIndex) -> &[T] {
        let key = self.keys[index.0 as usize];
        &self.buf[key.as_range()]
    }

    pub fn get_mut(&mut self, index: SliceIndex) -> &mut [T] {
        let key = self.keys[index.0 as usize];
        &mut self.buf[key.as_range()]
    }

    /// Drain values from the end of the buffer,
    /// regardless of which key it belongs to.
    ///
    /// This will update the last key in the buffer
    ///
    /// # Panics
    ///
    /// Panics if there are no keys in the buffer
    pub fn tail_drain(&mut self, size: u32) {
        assert!(!self.keys.is_empty());
        let key_index = self.keys.len() - 1;
        self.keys[key_index].end -= size;
        let pos = self.len() - size;
        let _ = self.buf.drain(pos as usize..);
    }

    /// Clear the entire buffer
    pub fn clear(&mut self) {
        self.buf.clear();
        self.keys.clear();
    }

    /// Get a reference to the last value in the buffer
    fn last(&self) -> Option<&T> {
        self.buf.last()
    }

    fn len(&self) -> u32 {
        self.buf.len() as u32
    }

    /// Truncate the storage.
    /// This can only run on the last key, or else it would
    /// damage the indices for the following keys.
    fn truncate(&mut self, key: SliceIndex, index: usize) {
        assert!(
            !self.keys.is_empty(),
            "tried to truncate from a buffer that contains zero slice keys"
        );
        let last_key_index = self.keys.len() - 1;
        assert_eq!(key.0 as usize, last_key_index, "trying to truncate before the last key");

        let slice = &mut self.keys[key.0 as usize];
        let index = index + slice.start as usize;
        self.buf.truncate(index);
        slice.end = self.buf.len() as u32;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn two_sessions() {
        let mut buffer = Buffer::empty();
        let mut s1 = buffer.new_session();
        let k1 = s1.next_slice();
        s1.extend([1, 2, 3]);

        let mut s2 = buffer.new_session();
        let k2 = s2.next_slice();
        s2.extend([10, 20, 30]);

        let b1 = buffer.get(k1);
        let b2 = buffer.get(k2);

        assert_eq!(b1, &[1, 2, 3]);
        assert_eq!(b2, &[10, 20, 30]);
    }

    #[test]
    fn buffer_insert_via_session() {
        let mut buffer = Buffer::<u8>::empty();
        let mut session = buffer.new_session();
        let k1 = session.next_slice();

        session.extend([b'a', b'b']);

        let k2 = session.next_slice();
        session.insert(1, b'x');
        session.push(b'z');

        let output = buffer.get(k1);
        assert_eq!(output, b"axb");

        let output = buffer.get(k2);
        assert_eq!(output, b"z");
    }

    #[test]
    fn session_pop() {
        let mut buffer = Buffer::<u8>::empty();
        let mut session = buffer.new_session();
        let k1 = session.next_slice();
        session.push(0);
        session.pop();

        assert!(buffer.buf.is_empty());
        assert!(buffer.get(k1).is_empty());
    }

    #[test]
    fn clear_buffer() {
        let mut buffer = Buffer::empty();
        let mut session = buffer.new_session();
        let _ = session.next_slice();
        session.push(0);

        assert_eq!(buffer.keys.len(), 1);
        assert_eq!(buffer.buf.len(), 1);

        buffer.clear();

        assert!(buffer.keys.is_empty());
        assert!(buffer.buf.is_empty());
    }

    #[test]
    fn tail_drain() {
        let mut buffer = Buffer::<u8>::empty();
        let mut session = buffer.new_session();
        let _k1 = session.next_slice();
        session.extend(0..10);
        session.tail_drain(3);

        let key = buffer.keys[0];
        assert_eq!(key.start, 0);
        assert_eq!(key.end, 7);
    }

    #[test]
    #[should_panic(expected = "tried to truncate from a buffer that contains zero slice keys")]
    fn truncate_empty_buffer() {
        let mut buffer = Buffer::<u8>::empty();
        buffer.truncate(SliceIndex(0), 123);
    }

    #[test]
    #[should_panic(expected = "trying to truncate before the last key")]
    fn truncate_before_last_key() {
        let mut buffer = Buffer::<u8>::empty();
        let mut session = buffer.new_session();

        let k1 = session.next_slice();
        let _k2 = session.next_slice();
        buffer.truncate(k1, 0);
    }

    #[test]
    fn truncate() {
        let mut buffer = Buffer::<u8>::empty();
        let mut session = buffer.new_session();
        let k1 = session.next_slice();
        session.extend([1, 2, 3]);
        session.truncate(k1, 2);
        assert_eq!(&[1, 2], session.slice(k1));
    }
}
