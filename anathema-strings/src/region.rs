use crate::{BUCKET_SIZE, StrIndex};

#[derive(Debug)]
pub(crate) struct Region {
    pub(crate) start: u32,
    pub(crate) len: u32,
}

impl Region {
    pub(crate) fn apply(&self, inner: &mut [u8], buffer: &mut Vec<u8>, len: usize) -> StrIndex {
        inner[self.start as usize..][..len].copy_from_slice(&buffer[..len]);
        buffer.clear();
        StrIndex::from((self.start, len as u32))
    }

    pub(crate) fn bucket_index(&self) -> u8 {
        (self.len as usize / BUCKET_SIZE) as u8
    }
}
