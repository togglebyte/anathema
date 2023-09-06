
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct NodeId(Vec<usize>);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(vec![id])
    }

    pub fn disposable() -> Self {
        Self(vec![])
    }

    pub fn contains(&self, other: &[usize]) -> bool {
        self.0 == &other[..self.0.len()]
    }

    pub fn next(&mut self) -> NodeId {
        let ret = NodeId(self.0.clone());
        self.0.last_mut().map(|v| *v += 1);
        ret
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

impl PartialEq<[usize]> for NodeId {
    fn eq(&self, other: &[usize]) -> bool {
        self.0 == other
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

