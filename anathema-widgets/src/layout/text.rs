use anathema_store::buffer::{Buffer, Session, SliceIndex};
use anathema_store::tree::ValueId;

#[derive(Debug, Copy, Clone)]
pub enum Entry {
    Str(usize, usize),
    SetStyle(ValueId),
    LineWidth(u32),
    Newline,
}

pub enum IterEntry<'a> {
    Str(&'a str),
    Style(ValueId),
}

pub struct TextBuffer {
    bytes: Buffer<u8>,
    layout: Buffer<Entry>,
}

impl TextBuffer {
    pub fn empty() -> Self {
        Self {
            bytes: Buffer::empty(),
            layout: Buffer::empty(),
        }
    }

    pub fn session(&mut self) -> TextSession<'_> {
        let bytes = Bytes(self.bytes.new_session());
        let layout = self.layout.new_session();

        TextSession { bytes, layout }
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        self.layout.clear();
    }
}

#[derive(Debug)]
pub struct Bytes<'a>(Session<'a, u8>);

impl<'a> Bytes<'a> {
    pub fn word(&self, slice: SliceIndex, offset: usize) -> &str {
        let slice = self.0.slice(slice);
        std::str::from_utf8(&slice[offset..]).expect("should always be valid utf8")
    }

    pub fn word_from(&self, slice: SliceIndex, start: usize, end: usize) -> &str {
        let slice = self.0.slice(slice);
        std::str::from_utf8(&slice[start..end]).expect("should always be valid utf8")
    }

    pub fn extend(&mut self, bytes: impl IntoIterator<Item = u8>) {
        self.0.extend(bytes);
    }

    pub fn pop(&mut self) {
        let _ = self.0.pop();
    }

    pub fn tail_drain(&mut self, size: usize) {
        let _from = self.0.len() - size;
        self.0.tail_drain(size);
    }

    pub fn ends_with_newline(&self) -> bool {
        self.0.last().map(|b| *b == b'\n').unwrap_or(false)
    }

    fn next_slice(&mut self) -> SliceIndex {
        self.0.next_slice()
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TextIndex {
    pub bytes: SliceIndex,
    pub layout: SliceIndex,
}

#[derive(Debug)]
pub struct TextSession<'a> {
    pub bytes: Bytes<'a>,
    pub layout: Session<'a, Entry>,
}

impl<'a> TextSession<'a> {
    pub fn new_key(&mut self) -> TextIndex {
        TextIndex {
            bytes: self.bytes.next_slice(),
            layout: self.layout.next_slice(),
        }
    }
}
