use std::hash::{BuildHasherDefault, Hasher};

pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<NoHashHasher>>;
pub type HashSet<K> = std::collections::HashSet<K, BuildHasherDefault<NoHashHasher>>;

#[derive(Debug, Default)]
pub struct NoHashHasher(u64);

impl Hasher for NoHashHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        panic!("don't use write, depend on the write_* functions instead")
    }

    fn write_u8(&mut self, i: u8) {
        self.0 = i as u64;
    }

    fn write_u16(&mut self, i: u16) {
        self.0 = i as u64;
    }

    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i as u64;
    }

    fn write_usize(&mut self, i: usize) {
        self.0 = i as u64;
    }
}
