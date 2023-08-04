use std::iter::once;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, Value};

use self::controlflow::ControlFlows;
use self::loops::LoopState;
use crate::expression::{Cond, ControlFlow, EvaluationContext, FromContext};
use crate::generator::Op;
use crate::{Expression, Generator};

pub(crate) mod controlflow;
pub(crate) mod loops;

// TODO: overflowing a u16 should use the Vec variant
enum NodeIdV2 {
    Small([u16; 15]),
    Large(Vec<usize>)
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

/// A single node in the node tree
pub enum NodeKind<Output: FromContext> {
    Single(Output, Nodes<Output>),
    Collection(LoopState<Output>),
    ControlFlow(ControlFlows<Output>),
}

impl<Output: FromContext> NodeKind<Output> {
    pub(crate) fn to_node(self, id: NodeId) -> Node<Output> {
        Node {
            id,
            kind: self,
        }
    }
}

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

    pub fn next(&mut self, bucket: &BucketRef<'_, Output::Value>) -> Option<Result<&mut Output, Output::Err>> {
        let nodes = self.inner[self.index..].iter_mut();

        for node in nodes {
            match &mut node.kind {
                NodeKind::Single(output, _) => {
                    self.index += 1;
                    return Some(Ok(output));
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
        let iter = self.inner
            .iter_mut()
            .map(|node| -> Box<dyn Iterator<Item = (&mut Output, &mut Self)>>  {
                match &mut node.kind {
                    NodeKind::Single(output, nodes) => Box::new(once((output, nodes))),
                    NodeKind::Collection(state) => state.nodes.iter_mut(),
                    NodeKind::ControlFlow(flows) => flows.nodes.iter_mut(),
                }
            })
            .flatten();

        Box::new(iter)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expression::FromContext;

    #[derive(Debug)]
    struct Widget {
        ident: &'static str,
    }

    impl Widget {
        fn layout(&mut self, bucket: &BucketRef<'_, u32>) {}
    }

    impl FromContext for Widget {
        type Ctx = &'static str;
        type Value = u32;

        fn from_context(ctx: &Self::Ctx, bucket: &BucketRef<'_, Self::Value>) -> Option<Self> {
            let w = Self { ident: ctx };
            Some(w)
        }
    }

    #[test]
    fn eval_for_loop() {
        let (expressions, bucket) = crate::testing::expressions();
        let bucket_ref = bucket.read();
        let mut ctx = EvaluationContext::new(&bucket_ref, None);

        let nodes = expressions
            .iter()
            .filter_map(|expr| expr.to_node(&ctx))
            .collect();

        let mut nodes = Nodes::<Widget>::new(nodes);

        assert_eq!("root", nodes.next(&bucket.read()).unwrap().ident);
        assert_eq!("inner loopy child 1", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 2", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 1", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 2", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("loopy child 1", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("loopy child 2", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 1", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 2", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 1", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("inner loopy child 2", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("loopy child 1", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("loopy child 2", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("truthy", nodes.next(&bucket_ref).unwrap().ident);
        assert_eq!("last", nodes.next(&bucket_ref).unwrap().ident);
    }
}
