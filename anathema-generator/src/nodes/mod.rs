use std::rc::Rc;

use anathema_values::State;

pub use self::id::NodeId;
use crate::expressions::{EvalState, Expression, Expressions, Loop};
use crate::Flap;

mod id;

#[derive(Debug)]
pub struct Node<Ctx: Flap> {
    node_id: NodeId,
    kind: NodeKind<Ctx>,
}

#[derive(Debug)]
pub enum NodeKind<Ctx: Flap> {
    Single(Ctx, Nodes<Ctx>),
    Loop { body: Nodes<Ctx>, l: Rc<Loop> },
}

#[derive(Debug)]
pub struct Nodes<Ctx: Flap> {
    expressions: Rc<Expressions<Ctx>>,
    inner: Vec<Node<Ctx>>,
    eval_state: EvalState,
    current_node: usize,
}

impl<Ctx: Flap> Nodes<Ctx> {
    pub(crate) fn new(expressions: Rc<Expressions<Ctx>>, eval_state: EvalState) -> Self {
        Self {
            expressions,
            inner: vec![],
            eval_state,
            current_node: 0,
        }
    }

    pub fn next(&mut self, state: &impl State) -> Option<Result<(&mut Ctx, &mut Nodes<Ctx>), Ctx::Err>> {
        if self.inner.len() == self.current_node {
            match self.expressions.next(&mut self.eval_state, state)? {
                Ok(node) => self.inner.push(node),
                Err(e) => return Some(Err(e)),
            }
        }

        let node = match &mut self.inner[self.current_node].kind {
            NodeKind::Single(output, nodes) => {
                self.current_node += 1;
                Some(Ok((output, nodes)))
            }
            _ => {
                panic!()
                // self.next(state)
            }
        };

        node

        // loop {
        //     let node = expressions.next(&mut self.eval_state)?;
        //     match node {
        //         Ok(node) => nodes.push(node),
        //         Err(e) => return Some(Err(e)),
        //     }

        //     let nodes = &mut *nodes;
        //     if let NodeKind::Single(val, nodes) = &mut nodes.last_mut()?.kind {
        //         break Some(Ok((val, nodes)));
        //     } else {
        //         continue;
        //     }
        // }
    }
}

pub(crate) fn expression_to_node<Ctx: Flap>(
    expr: &Expression<Ctx>,
    node_id: NodeId,
    state: &impl State,
) -> Option<Result<Node<Ctx>, Ctx::Err>> {
    match expr {
        Expression::Node {
            children, context, ..
        } => {
            let node = Ctx::do_it(context, state).map(|output| {
                let eval_state = EvalState::with_parent(&node_id);
                Node {
                    node_id,
                    kind: NodeKind::Single(output, Nodes::new(children.clone(), eval_state)),
                }
            });
            Some(node)
        }
        Expression::Loop(state, body) => {
            // None
            let node = Node {
                kind: NodeKind::Loop {
                    body: Nodes::new(body.clone(), EvalState::with_parent(&node_id)),
                    l: state.clone(),
                },
                node_id,
            };
            Some(Ok(node))
        }
        Expression::ControlFlow(state) => None,
    }
}
