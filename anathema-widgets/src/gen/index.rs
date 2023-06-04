use super::generator::Direction;

#[derive(Debug)]
pub(super) struct Index {
    inner: usize,
    collection_len: usize,
    pub(super) dir: Direction,
}

impl Index {
    pub(super) fn next(&mut self) -> Option<usize> {
        if self.inner >= self.collection_len {
            return None;
        }

        let idx = match self.dir {
            Direction::Forward => self.inner,
            Direction::Backward => self.collection_len - self.inner - 1,
        };

        self.inner += 1;
        Some(idx)
    }

    pub(super) fn reverse(&mut self) {
        match self.dir {
            Direction::Forward => {
                self.dir = Direction::Backward;
                self.inner = self.collection_len - self.inner + 1;
            }
            Direction::Backward => {
                self.dir = Direction::Forward;
                self.inner = self.collection_len - self.inner + 1;
            }
        };
    }

    pub(super) fn flip(&mut self) {
        self.dir = match self.dir {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }

    pub const fn new(dir: Direction, collection_len: usize) -> Self {
        Self {
            dir,
            collection_len,
            inner: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn next_index() {
        let mut index = Index::new(Direction::Forward, 1);
        assert_eq!(0, index.next().unwrap());
        assert!(index.next().is_none());
    }

    #[test]
    fn next_zero_index_len() {
        let mut index = Index::new(Direction::Forward, 0);
        assert!(index.next().is_none());
    }

    #[test]
    fn reverse() {
        let mut index = Index::new(Direction::Forward, 2);
        index.next(); // 0
        index.next(); // 1
        index.reverse(); // next is 0
        assert_eq!(0, index.next().unwrap());
    }

    #[test]
    fn reverse_empty() {
        let mut index = Index::new(Direction::Forward, 0);
        index.reverse();
        assert!(index.next().is_none());
    }

    #[test]
    fn reverse_one_len() {
        let mut index = Index::new(Direction::Forward, 1);
        index.reverse();
        assert!(index.next().is_none());
    }

    #[test]
    fn flip() {
        let mut index = Index::new(Direction::Forward, 2);
        index.flip();
        assert_eq!(1, index.next().unwrap());
    }

    #[test]
    fn flip_empty() {
        let mut index = Index::new(Direction::Forward, 0);
        index.flip();
        assert!(index.next().is_none());
    }

    #[test]
    fn flip_single() {
        let mut index = Index::new(Direction::Forward, 1);
        index.flip();
        assert_eq!(0, index.next().unwrap());
    }
}
