/// Contain a FIFO list of node paths
pub struct PathList {
    offset: usize,
    inner: Vec<u16>,
}

impl PathList {
    /// Push a node path into the list.
    pub fn push(&mut self, path: &[u16]) {
        let len = path.len() as u16;
        self.inner.push(len);
        self.inner.extend(path);
    }
}

/// The owner and controller of the path list.
pub struct PathListCtl {
    list: PathList,
}

impl PathListCtl {
    /// Create a new path list controller (and subsequently a new path list)
    pub fn new() -> Self {
        Self {
            list: PathList {
                offset: 0,
                inner: vec![],
            },
        }
    }

    /// Returns true if there are values in the current list
    pub fn is_empty(&self) -> bool {
        self.list.offset != self.list.inner.len()
    }

    /// Borrow the path list mutably.
    pub fn list(&mut self) -> &mut PathList {
        &mut self.list
    }

    /// Get the next path.
    /// Once all paths have been consumed the path list is cleared.
    /// This means the path list should always be exhausted before it's filled up again
    pub fn consume_next(&mut self) -> Option<&[u16]> {
        let list = &mut self.list;

        if list.offset == list.inner.len() {
            list.inner.clear();
            list.offset = 0;
            return None;
        }

        let len = list.inner[list.offset] as usize;
        list.offset += 1;
        let path = &list.inner[list.offset..][..len];
        list.offset += len;
        Some(path)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn push_clear() {
        let mut plc = PathListCtl::new();
        let list = plc.list();
        list.push(&[1, 2]);
        list.push(&[1, 2, 3]);

        assert_eq!(&[1, 2], plc.consume_next().unwrap());
        assert_eq!(&[1, 2, 3], plc.consume_next().unwrap());
        assert_eq!(None, plc.consume_next());

        assert_eq!(plc.list.offset, 0);
        assert!(plc.list.inner.is_empty());
    }

    #[test]
    fn empty_next() {
        let mut plc = PathListCtl::new();
        assert_eq!(None, plc.consume_next());
    }
}
