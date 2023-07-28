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

impl<Output: FromContext> Node<Output> {
    fn has_node(&mut self, bucket: &BucketRef<'_, Output::Value>) -> bool {
        match self {
            Self::Single(..) => true,
            Self::Collection(state) => state
                .generate_next(bucket)
                .map(|node| node.has_node(bucket))
                .unwrap_or(false),
            Self::ControlFlow(flow) => flow
                .generate_next(bucket)
                .map(|node| node.has_node(bucket))
                .unwrap_or(false),
        }
    }

    fn get_node(&mut self) -> Option<&mut Output> {
        match self {
            Self::Single(output, ..) => Some(output),
            Self::Collection(state) => state.last(),
            Self::ControlFlow(state) => state.last(),
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

    // The api should be able to do the following:
    // 1. Produce new values
    // 2. Iterate over produced values
    // 3. Find a Node(value) by some id
    pub fn next(&mut self, bucket: &BucketRef<'_, Output::Value>) -> Option<&mut Output> {
        if self.index == self.inner.len() {
            return None;
        }

        let index = self.index;

        // TODO: know when to advance the index
        let node = &mut self.inner[index];
        let has_node = node.has_node(bucket);

        match (&mut *node, has_node) {
            (Node::Single(..), _) => self.index += 1,
            (Node::Collection(_) | Node::ControlFlow(_), true) => {}
            (Node::Collection(_) | Node::ControlFlow(_), false) => {
                self.index += 1;
                return self.next(bucket);
            }
        }

        self.inner[index].get_node()
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

        while let Some(widget) = nodes.next(&bucket.read()) {
            println!("{widget:?}");
        }
    }
}
