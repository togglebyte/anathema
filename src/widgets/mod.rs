use std::collections::VecDeque;

pub mod lines;

pub struct ScrollBuffer<T> {
    entries: VecDeque<T>,
    height: usize,
    pos: usize,
    max_buffer: usize,
    dirty: bool,
}

impl<T> ScrollBuffer<T> {
    /// Create a new buffer witgh a given height an a max size.
    pub fn new(height: usize, max_buffer: usize) -> Self {
        let mut entries = VecDeque::new();
        entries.make_contiguous();
        Self { entries, height, pos: 0, max_buffer, dirty: true }
    }

    /// Create a buffer from an existing vector
    pub fn from_vec(entries: Vec<T>, height: usize, max_buffer: usize) -> Self {
        let mut entries: VecDeque<T> = entries.into();
        entries.make_contiguous();
        Self { entries: entries.into(), height, pos: 0, max_buffer, dirty: true }
    }

    fn is_at_end(&self) -> bool {
        self.pos == self.entries.len().saturating_sub(self.height)
    }

    /// Get the lines of the buffer if the buffer is `dirty`,
    /// once the lines have been received the buffer is no longer marked 
    /// as `dirty` and any subsequent calls to `lines` will return an empty
    /// slice, until the buffer changes again.
    /// 
    /// This makes it cheap to call `lines` in a loop.
    pub fn lines(&mut self) -> &[T] {
        if self.dirty {
            let to = (self.pos + self.height).min(self.entries.len());
            let from = self.pos.min(to.saturating_sub(self.height));
            self.dirty = false;
            &self.entries.as_slices().0[from..to]
        } else {
            &[]
        }
    }

    /// Resize the buffer to a new height,
    /// and mark it as dirty
    pub fn resize(&mut self, new_heigth: usize) {
        self.height = new_heigth;
        self.dirty = true;
    }

    /// Scroll down N lines and mark it as dirty
    pub fn scroll_down(&mut self, lines: usize) {
        self.pos = (self.pos + lines).min(self.entries.len().saturating_sub(self.height));
        self.dirty = true;
    }

    /// Scroll up N lines and mark it as dirty
    pub fn scroll_up(&mut self, lines: usize) {
        self.pos = self.pos.saturating_sub(lines);
        self.dirty = true;
    }

    /// Scroll to the end of the buffer and marke it as dirty
    pub fn scroll_to_end(&mut self) {
        self.pos = self.entries.len().saturating_sub(self.height);
        self.dirty = true;
    }

    /// Push a new entry to the buffer and mark it as dirty.
    /// If the buffer is full the oldest entry will be removed.
    pub fn push(&mut self, entry: T) {
        if self.entries.len() == self.max_buffer {
            self.entries.pop_front();
        }

        let is_at_end = self.is_at_end();

        self.entries.push_back(entry);

        match is_at_end {
            true => self.scroll_down(1),
            false => self.scroll_up(0),
        }
    }

    /// If the buffer has changed this returns `true`
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the buffer and mark it as dirty
    pub fn clear(&mut self) {
        self.entries.clear();
        self.dirty = true;
    }
}
