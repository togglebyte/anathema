use std::sync::Arc;

#[derive(Debug)]
pub struct NextNodeId(usize);

impl NextNodeId {
    pub fn new(init: usize) -> Self {
        Self(init)
    }

    pub fn next(&mut self, node_id: &NodeId) -> NodeId {
        let mut ret = node_id.0.to_vec();
        if let Some(v) = ret.last_mut() {
            *v = self.0;
        }
        self.0 += 1;
        NodeId(ret.into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
// #[repr(transparent)]
// TODO: This could possibly be Rc<[usize]> instead,
//       or even Arc<[usize]>, given that both the `WidgetContainer`
//       and the wrapping `Node` has the same id, and these ids are
//       shared with the ids tracking the changes.
// #[derive(PartialOrd, Ord)]
pub struct NodeId(pub Arc<[usize]>);
// pub struct NodeId(pub Vec<usize>);

impl NodeId {
    pub fn new(id: usize) -> Self {
        Self(vec![id].into())
    }

    pub fn contains(&self, other: &[usize]) -> bool {
        *self.0 == other[..self.0.len()]
    }

    pub fn last(&self) -> usize {
        self.0[self.0.len() - 1]
    }

    // pub fn next(&self) -> NodeId {
    //     let mut child = self.0.to_vec();
    //     if let Some(v) = child.last_mut() {
    //         *v += 1;
    //     }
    //     Self(child.into())
    // }

    // pub fn next(&mut self) -> NodeId {
    //     let ret = NodeId(self.0.clone());
    //     if let Some(v) = self.0.last_mut() {
    //         *v += 1;
    //     }
    //     ret
    // }


//     pub fn child(&self, next: usize) -> Self {
//         let mut v = Vec::with_capacity(self.0.len() + 1);
//         v.extend_from_slice(&*self.0);
//         v.push(next);
//         Self(v.into())
//     }

//     pub fn as_slice(&self) -> &[usize] {
//         &self.0
//     }
}


impl NodeId {
    // pub fn new(id: usize) -> Self {
    //     Self(vec![id])
    // }

    // pub fn contains(&self, other: &[usize]) -> bool {
    //     *self.0 == other[..self.0.len()]
    // }

    // pub fn next(&mut self) -> NodeId {
    //     let ret = NodeId(self.0.clone());
    //     if let Some(v) = self.0.last_mut() {
    //         *v += 1;
    //     }
    //     ret
    // }

    pub fn child(&self, next: usize) -> Self {
        let mut v = Vec::with_capacity(self.0.len() + 1);
        v.extend_from_slice(&self.0);
        v.push(next);
        Self(v.into())
    }

    pub fn as_slice(&self) -> &[usize] {
        &self.0
    }
}

impl PartialEq<[usize]> for NodeId {
    fn eq(&self, other: &[usize]) -> bool {
        &*self.0 == other
    }
}

impl From<Vec<usize>> for NodeId {
    fn from(values: Vec<usize>) -> Self {
        Self(values.into())
    }
}

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
