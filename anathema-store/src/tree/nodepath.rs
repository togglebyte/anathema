pub const fn root_node() -> &'static [u16] {
    &[]
}

pub fn new_node_path(parent: &[u16], index: u16) -> Box<[u16]> {
    let mut path = Vec::with_capacity(parent.len() + 1);
    path.extend(parent);
    path.push(index);
    path.into()
}

pub trait AsNodePath {
    /// Split the path in two halves: the parent and the last index
    fn split_parent(&self) -> Option<(&[u16], usize)>;

    /// Return the parent of path (this is the current path minus the last index)
    fn parent(&self) -> Option<&[u16]>;

    fn reparent(&mut self, new_parent: &[u16]);
}

impl AsNodePath for [u16] {
    fn split_parent(&self) -> Option<(&[u16], usize)> {
        match self {
            [] => None,
            [i] => Some((&[], *i as usize)),
            [parent @ .., i] => Some((parent, *i as usize)),
        }
    }

    fn parent(&self) -> Option<&[u16]> {
        match self {
            [] => None,
            [_] => None,
            [parent @ .., _] => Some(parent),
        }
    }

    fn reparent(&mut self, new_parent: &[u16]) {
        debug_assert!(new_parent.len() <= self.len(), "");
        self[..new_parent.len()].copy_from_slice(new_parent);
    }
}

// impl AsNodePath for NodePath {
//     fn split_parent(&self) -> Option<(&[u16], usize)> {
//         self.split()
//     }

//     fn pop(&self) -> Option<&[u16]> {
//         self.as_slice().pop()
//     }
// }

// /// Node path indicates where in the tree a node is.
// /// The node path can change through a values life time,
// /// unlike the value key it self.
// #[derive(Debug, Clone, PartialEq)]
// pub struct NodePath(Box<[u16]>);

// impl NodePath {
//     pub fn root() -> Self {
//         Self(Box::new([]))
//     }

//     pub fn contains(&self, other: &Self) -> bool {
//         let len = self.0.len().min(other.0.len());
//         self.0[..len] == other.0[..len]
//     }

//     pub fn reparent(&mut self, new_parent: &NodePath) {
//         debug_assert!(new_parent.0.len() <= self.0.len());
//         self.0[..new_parent.0.len()].copy_from_slice(&new_parent.0);
//     }

//     pub fn split(&self) -> Option<(&[u16], usize)> {
//         match self.as_slice() {
//             [] => None,
//             [i] => Some((&[], *i as usize)),
//             [parent @ .., i] => Some((parent, *i as usize)),
//         }
//     }

//     pub fn as_slice(&self) -> &[u16] {
//         &self.0
//     }

//     pub fn as_slice_mut(&mut self) -> &mut [u16] {
//         &mut self.0
//     }
// }

// impl Deref for NodePath {
//     type Target = [u16];

//     fn deref(&self) -> &Self::Target {
//         self.as_slice()
//     }
// }

// impl Default for NodePath {
//     fn default() -> Self {
//         NodePath::root()
//     }
// }

// impl Add<u16> for &NodePath {
//     type Output = NodePath;

//     fn add(self, rhs: u16) -> Self::Output {
//         let mut node_id = Vec::with_capacity(self.0.len() + 1);
//         node_id.extend_from_slice(&self.0);
//         node_id.push(rhs);
//         NodePath(node_id.into_boxed_slice())
//     }
// }

// impl From<(&[u16], usize)> for NodePath {
//     fn from((root, index): (&[u16], usize)) -> Self {
//         let mut path = Vec::with_capacity(root.len() + 1);
//         path.extend_from_slice(root);
//         path.push(index as u16);
//         Self(path.into_boxed_slice())
//     }
// }

// impl From<&[u16]> for NodePath {
//     fn from(root: &[u16]) -> Self {
//         Self(root.to_vec().into_boxed_slice())
//     }
// }

// impl<const N: usize> From<[u16; N]> for NodePath {
//     fn from(value: [u16; N]) -> Self {
//         NodePath(Box::new(value))
//     }
// }
