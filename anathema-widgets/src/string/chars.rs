// -----------------------------------------------------------------------------
//   - Index -
// -----------------------------------------------------------------------------
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Index {
    pub(crate) slice: u32,
    pub(crate) byte: u32,
}

impl Index {
    pub(crate) const ZERO: Self = Self { slice: 0, byte: 0 };

    pub(crate) const fn new(slice: u32, byte: u32) -> Self {
        Self { slice, byte }
    }

    pub(crate) const fn as_usize(self) -> (usize, usize) {
        (self.slice as usize, self.byte as usize)
    }
}

// -----------------------------------------------------------------------------
//   - Iterator -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub(crate) struct CharIndices<'a, 'b, T> {
    // pub(crate) slices: &'b [(&'a str, T)],
    pub(crate) slices: &'b [(&'a str, T)],
    index: Index,
    indices: std::str::CharIndices<'a>,
    next: Option<(Index, char)>,
}

impl<'a, 'b, T> CharIndices<'a, 'b, T> {
    pub fn new(slices: &'b [(&'a str, T)]) -> Self {
        let indices = slices
            .first()
            .map(|(slice, _)| slice.char_indices())
            .unwrap_or_else(|| "".char_indices());

        Self {
            slices,
            index: Index { slice: 0, byte: 0 },
            indices,
            next: None,
        }
    }

    pub fn peek(&mut self) -> Option<(Index, char)> {
        if let None = self.next {
            self.next = self.next();
        }

        self.next
    }
}

impl<'a, 'b, T> Iterator for CharIndices<'a, 'b, T> {
    type Item = (Index, char);

    fn next(&mut self) -> Option<Self::Item> {
        if let val @ Some(_) = self.next.take() {
            return val;
        };

        match self.indices.next() {
            Some((index, c)) => {
                self.index.byte = index as u32;
                let ret = Some((self.index, c));
                ret
            }
            None if 1 + self.index.slice as usize == self.slices.len() => None,
            None => {
                self.index.slice += 1;
                self.indices = self.slices[self.index.slice as usize].0.char_indices();
                self.index.byte = 0;
                self.next()
            }
        }
    }
}
