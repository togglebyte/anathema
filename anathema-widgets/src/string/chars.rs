use crate::string::slice::Slice;

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
pub struct CharIndices<'a, 'b, T> {
    pub(crate) slice: Slice<'a, 'b, T>,
    index: Index,
    // This is the first byte offset and is only used for the 
    // first slice.
    byte_offset: u32,
    indices: Option<std::str::CharIndices<'a>>,
    next: Option<(Index, char)>,
}

impl<'a, 'b, T: Copy> CharIndices<'a, 'b, T> {
    pub fn new(mut slices: Slice<'a, 'b, T>) -> Self {
        let index = slices.start();
        let indices = slices.next().map(|(s, _)| s.char_indices());
        Self {
            slice: slices,
            byte_offset: index.byte,
            index,
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

impl<'a, 'b, T: Copy> Iterator for CharIndices<'a, 'b, T> {
    type Item = (Index, char);

    fn next(&mut self) -> Option<Self::Item> {
        if let val @ Some(_) = self.next.take() {
            return val;
        };

        // Get the current char_indices or fetch
        // the next set and increment the slice index
        let indices = match self.indices.as_mut() {
            Some(indices) => indices,
            None => {
                self.indices = Some(self.slice.next()?.0.char_indices());
                self.index.slice += 1;
                self.index.byte = 0;
                self.byte_offset = 0;
                return self.next();
            }
        };

        match indices.next() {
            Some((index, c)) => {
                self.index.byte = index as u32 + self.byte_offset;
                let ret = Some((self.index, c));
                ret
            }
            None => {
                self.indices = None;
                self.next()
            }
        }
    }
}
