use crate::{Hoppstr, BUCKET_SIZE};

#[derive(Debug)]
pub(crate) struct Region {
    pub(crate) start: usize,
    pub(crate) len: usize,
}

impl Region {
    pub(crate) fn apply(&self, inner: &mut Vec<u8>, buffer: &mut Vec<u8>, len: usize) -> Hoppstr {
        inner[self.start..][..len].copy_from_slice(&buffer[..len]);
        buffer.clear();
        Hoppstr::new(self.start, len)
    }

    pub(crate) fn bucket_index(&self) -> u8 {
        (self.len / BUCKET_SIZE) as u8
    }
}
