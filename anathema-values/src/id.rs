
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct NodeId(Vec<usize>);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(vec![id])
    }

    pub fn child(&self, next: usize) -> Self {
        let mut v = Vec::with_capacity(self.0.len() + 1);
        v.extend(&self.0);
        v.push(next);
        Self(v)
    }

    pub fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

impl From<Vec<usize>> for NodeId {
    fn from(values: Vec<usize>) -> Self {
        Self(values)
    }
}

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}


