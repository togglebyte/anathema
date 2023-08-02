use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use anathema_values::{BucketRef, List, PathId, ScopeId, Truthy, ValueRef, ValueV2};

use self::controlflow::ControlFlows;
use self::loops::LoopState;
use crate::expression::{Cond, ControlFlow, EvaluationContext, FromContext};
use crate::generator::Op;
use crate::{Expression, Generator};

pub(crate) mod controlflow;
pub(crate) mod loops;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct NodeId(usize);

fn next() -> NodeId {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    NodeId(NEXT.fetch_add(1, Ordering::Relaxed))
}

/// A single node in the node tree
pub enum Node<Output: FromContext> {
    Single(Output, Nodes<Output>),
    Collection(LoopState<Output>),
    ControlFlow(ControlFlows<Output>),
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

    pub fn next(&mut self, bucket: &BucketRef<'_, Output::Value>) -> Option<&mut Output> {
        let gen = self.inner[self.index..].iter_mut();

        for generator in gen {
            match generator {
                Node::Single(node, _) => {
                    self.index += 1;
                    return Some(node);
                }
                Node::Collection(loop_state) => match loop_state.next(bucket) {
                    last @ Some(_) => return last,
                    None => self.index += 1,
                },
                Node::ControlFlow(flows) => match flows.next(bucket) {
                    last @ Some(_) => return last,
                    None => self.index += 1,
                },
            }
        }

        None
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
