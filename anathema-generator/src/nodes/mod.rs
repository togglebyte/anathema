use std::fmt::{self, Debug};
use std::iter::once;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anathema_values::{Container, List, PathId, ScopeId, StoreRef, Truthy, ValueRef};

use self::controlflow::ControlFlows;
use self::loops::LoopState;
use crate::expression::{EvaluationContext, FromContext};
use crate::Expression;

pub(crate) mod controlflow;
pub(crate) mod loops;

// TODO: One possible solution to the partial rebuild
//       could be message passing.
//
//       enum Change {
//          Add,
//          Remove(NodeId),
//          Update,
//          Swap(NodeId, NodeId)
//       }

// TODO: overflowing a u16 should use the Vec variant
enum NodeIdV2 {
    Small([u16; 15]),
    Large(Vec<usize>),
}

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

pub trait RemoveTrigger: Default {
    fn remove(&mut self);
}

fn next() -> NodeId {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    NodeId(vec![NEXT.fetch_add(1, Ordering::Relaxed)])
}

pub struct Node<Output: FromContext> {
    id: NodeId,
    kind: NodeKind<Output>,
}

impl<Output: FromContext> Node<Output> {
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    #[cfg(test)]
    pub fn single(self) -> Option<(Output, Nodes<Output>)> {
        match self.kind {
            NodeKind::Single(output, children) => Some((output, children)),
            _ => None,
        }
    }
}

impl<T> Debug for Node<T>
where
    T: FromContext + Debug,
    T::Value: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .finish()
    }
}

/// A single node in the node tree
#[derive(Debug)]
pub enum NodeKind<Output: FromContext> {
    Single(Output, Nodes<Output>),
    Collection(LoopState<Output>),
    ControlFlow(ControlFlows<Output>),
}

impl<Output: FromContext> NodeKind<Output> {
    pub(crate) fn to_node(self, id: NodeId) -> Node<Output> {
        Node { id, kind: self }
    }
}

#[derive(Debug)]
pub struct Nodes<Output: FromContext> {
    index: usize,
    inner: Vec<Node<Output>>,
}

impl<Output> Nodes<Output>
where
    Output: FromContext,
{
    pub fn new(nodes: Vec<Node<Output>>) -> Self {
        Self {
            inner: nodes,
            index: 0,
        }
    }

    pub(crate) fn empty() -> Self {
        Self::new(vec![])
    }

    fn push(&mut self, node: Node<Output>) {
        self.inner.push(node);
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Generate more nodes if needed (and there is enough information to produce more)
    pub fn next(
        &mut self,
        bucket: &StoreRef<'_, Output::Value>,
    ) -> Option<Result<(&mut Output, &mut Nodes<Output>), Output::Err>> {
        let nodes = self.inner[self.index..].iter_mut();

        for node in nodes {
            match &mut node.kind {
                NodeKind::Single(output, children) => {
                    self.index += 1;
                    return Some(Ok((output, children)));
                }
                NodeKind::Collection(loop_state) => match loop_state.next(bucket, &node.id) {
                    last @ Some(_) => return last,
                    None => self.index += 1,
                },
                NodeKind::ControlFlow(flows) => match flows.next(bucket, &node.id) {
                    last @ Some(_) => return last,
                    None => self.index += 1,
                },
            }
        }

        None
    }

    pub fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (&mut Output, &mut Self)> + '_> {
        let iter = self
            .inner
            .iter_mut()
            .map(
                |node| -> Box<dyn Iterator<Item = (&mut Output, &mut Self)>> {
                    match &mut node.kind {
                        NodeKind::Single(output, nodes) => Box::new(once((output, nodes))),
                        NodeKind::Collection(state) => state.nodes.iter_mut(),
                        NodeKind::ControlFlow(flows) => flows.nodes.iter_mut(),
                    }
                },
            )
            .flatten();

        Box::new(iter)
    }

    pub fn first_mut(&mut self) -> Option<(&mut Output, &mut Self)> {
        let first_node = self.inner.first_mut()?;
        match &mut first_node.kind {
            NodeKind::Single(output, children) => Some((output, children)),
            NodeKind::Collection(state) => state.nodes.first_mut(),
            NodeKind::ControlFlow(flows) => flows.nodes.first_mut(),
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::Store;

    use super::*;
    use crate::expression::FromContext;
    use crate::testing::{Widget, expression, for_expression};
}
