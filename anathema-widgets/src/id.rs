use std::fmt;

// -----------------------------------------------------------------------------
//   - Node id -
// -----------------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(Vec<usize>);

impl NodeId {
    pub fn empty() -> Self {
        Self(vec![])
    }

    pub fn append(&self, next: usize) -> Self {
        let mut next_id = Vec::with_capacity(self.0.len() + 1);
        next_id.extend(self.0.clone());
        next_id.push(next);
        NodeId(next_id)
    }

    pub fn offset(&self, offset: usize) -> Self {
        let mut id = self.clone();
        id.0.last_mut().map(|last| *last += offset);
        id
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().map(ToString::to_string).collect::<Vec<_>>().join("::"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Id {
    Node(u32),
    For(u32),
    ControlFlow(u32),
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node(id) => write!(f, "{id}"),
            Self::For(id) => write!(f, "F{id}"),
            Self::ControlFlow(id) => write!(f, "?{id}"),
        }
    }
}
