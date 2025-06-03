use anathema_store::slab::Slab;

pub use self::transaction::Transaction;
use crate::region::Region;
use crate::StrIndex;

mod transaction;

const END: usize = 5;

fn bytes_to_str<'a, 'slice>(bytes: &mut &'a [u8], slices: &'a Slab<u16, &'slice str>) -> &'a str {
    match &bytes[..2] {
        [0xFF, 0xFF] => {
            let len = u16::from_ne_bytes([bytes[2], bytes[3]]) as usize;
            let s = unsafe { std::str::from_utf8_unchecked(&bytes[4..len + 4]) };
            *bytes = &bytes[4 + len..];
            s
        }
        index => {
            *bytes = &bytes[2..];
            let index = u16::from_ne_bytes([index[0], index[1]]);
            slices.get(index).unwrap()
        }
    }
}

fn remove_slice<'slice>(bytes: &mut &[u8], slices: &mut Slab<u16, &'slice str>) {
    while !bytes.is_empty() {
        match &bytes[..2] {
            [0xFF, 0xFF] => {
                let len = u16::from_ne_bytes([bytes[2], bytes[3]]) as usize;
                *bytes = &bytes[4 + len..];
            }
            index => {
                *bytes = &bytes[2..];
                let index = u16::from_ne_bytes([index[0], index[1]]);
                slices.remove(index);
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct Storage<'slice> {
    inner: Vec<u8>,
    slices: Slab<u16, &'slice str>,
    free: Slab<u8, Vec<Region>>,
    buffer: Vec<u8>,
}

impl<'slice> Storage<'slice> {
    pub(super) fn empty() -> Self {
        let mut free = Slab::empty();
        for _ in 0..END {
            free.insert(vec![]);
        }

        Self {
            inner: vec![],
            slices: Slab::empty(),
            free,
            buffer: vec![],
        }
    }

    pub fn begin_insert(&mut self) -> Transaction<'_, 'slice> {
        Transaction::new(self)
    }

    pub fn get(&self, hoppstr: StrIndex) -> impl Iterator<Item = &str> + Clone {
        let mut bytes = &self.inner[hoppstr.index as usize..][..hoppstr.len as usize];

        std::iter::from_fn(move || {
            if !bytes.is_empty() {
                let s = bytes_to_str(&mut bytes, &self.slices);
                Some(s)
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, hoppstr: StrIndex) {
        let mut bytes = &self.inner[hoppstr.index as usize..][..hoppstr.len as usize];
        remove_slice(&mut bytes, &mut self.slices);
        let region = hoppstr.to_region();
        self.free
            .get_mut(region.bucket_index())
            .map(|regions| regions.push(region));
    }
}
