use std::fmt::Write;

use super::{Storage, END};
use crate::region::Region;
use crate::{Hoppstr, BUCKET_SIZE};

pub struct Transaction<'a, 'slice> {
    storage: &'a mut Storage<'slice>,
}

impl<'a, 'slice> Transaction<'a, 'slice> {
    pub(super) fn new(storage: &'a mut Storage<'slice>) -> Self {
        Self { storage }
    }

    pub fn add_slice(&mut self, slice: &'slice str) {
        let key = self.storage.slices.insert(slice);
        assert!(key != u16::MAX);
        self.storage.buffer.extend_from_slice(&key.to_ne_bytes());
    }

    pub fn commit(mut self) -> Hoppstr {
        let len = self.storage.buffer.len();

        // Steps
        // * Find available storage in the free list that can house our bytes
        // * If no space is available then append this (remember to pad)
        // * Only need to care about padding when there is no free region
        let padding = len % BUCKET_SIZE;
        let size = len + padding;

        let key_range = (size / BUCKET_SIZE).min(END)..END;
        for i in key_range {
            let storage = self.get_storage(i);
            let Some(region) = storage.pop() else { continue };
            return region.apply(&mut self.storage.inner, &mut self.storage.buffer, len);
        }

        // perform special case region pop ONLY if the
        // len is larger than BUCKET_SIZE * 4
        if len > BUCKET_SIZE * 4 {
            let variable_storage = self.get_storage(END - 1);
            variable_storage.sort_unstable_by(|a, b| a.len.cmp(&b.len));

            if let Some(idx) = variable_storage.iter().position(|region| region.len >= len) {
                let region = variable_storage.remove(idx);
                return region.apply(&mut self.storage.inner, &mut self.storage.buffer, len);
            }
        }

        // No free region so make a new one
        let start = self.storage.inner.len();
        self.storage
            .inner
            .extend_from_slice(&self.storage.buffer[..len]);

        self.storage
            .inner
            .resize(self.storage.inner.len() + padding, 0);
        Hoppstr::new(start, len)
    }

    fn get_storage(&mut self, index: usize) -> &mut Vec<Region> {
        let storage = self
            .storage
            .free
            .get_mut(index as u8)
            .expect("this is pre-generated");
        storage
    }
}

impl Write for Transaction<'_, '_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        assert!(s.len() <= u16::MAX as usize);
        let len = s.len() as u16;
        self.storage.buffer.extend_from_slice(&[0xFF, 0xFF]);
        self.storage.buffer.extend_from_slice(&len.to_ne_bytes());
        self.storage.buffer.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
