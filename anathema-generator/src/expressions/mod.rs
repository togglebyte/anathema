mod controlflow;
mod loops;

use std::rc::Rc;

use anathema_values::State;
use controlflow::Cond;
pub(crate) use loops::Loop;

use self::controlflow::FlowState;
use crate::nodes::{expression_to_node, Node, Nodes};
use crate::{Attributes, Flap, NodeId};

#[derive(Debug)]
pub(crate) struct EvalState {
    parent: Option<NodeId>,
    pub(crate) current_expr: usize,
    next_id: usize,
}

impl EvalState {
    pub fn new() -> Self {
        Self {
            current_expr: 0,
            next_id: 0,
            parent: None,
        }
    }

    pub(crate) fn with_parent(parent: &NodeId) -> Self {
        Self {
            parent: Some(parent.clone()),
            ..Self::new()
        }
    }

    fn next_id(&mut self) -> NodeId {
        self.next_id += 1;
        match &self.parent {
            Some(parent) => parent.child(self.next_id - 1),
            None => (self.next_id - 1).into(),
        }
    }
}

#[derive(Debug)]
pub enum Expression<Ctx: Flap> {
    Node {
        context: Rc<Ctx::Meta>,
        attributes: Attributes,
        children: Rc<Expressions<Ctx>>,
    },
    Loop(Rc<Loop>, Rc<Expressions<Ctx>>),
    ControlFlow(FlowState),
}

#[derive(Debug)]
pub struct Expressions<Ctx: Flap>(Vec<Expression<Ctx>>);

impl<Ctx: Flap> Expressions<Ctx> {
    pub fn new(inner: Vec<Expression<Ctx>>) -> Self {
        Self(inner)
    }

    pub(crate) fn next(&self, eval: &mut EvalState, state: &impl State) -> Option<Result<Node<Ctx>, Ctx::Err>> {
        let expression = &self.0[eval.current_expr];
        let output = expression_to_node(expression, eval.next_id(), state);

        if output.is_none() {
            eval.current_expr += 1;

            if eval.current_expr == self.0.len() {
                return None;
            }

            self.next(eval, state)
        } else {
            output
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    #[test]
    fn eval_node() {
        let expr = expression("text", (), ());
        let mut eval_state = EvalState::new();
        let n = expression_to_node(&expr, eval_state.next_id());
        // assert_eq!(expected, actual);
    }

    #[test]
    fn eval_for() {
        let expr = for_expression(
            "item", 
            [1, 2, 3],
            [expression("text", (), ())]
        );
        let expressions = Expressions::new(vec![expr]);
        let mut eval_state = EvalState::new();
        let n = expressions.next(&mut eval_state).unwrap().unwrap();
        panic!("{n:#?}");
        // assert_eq!(expected, actual);
    }
}
