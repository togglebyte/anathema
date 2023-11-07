use std::ops::ControlFlow;

use anathema_values::{Change, Context, Deferred, LocalScope, NodeId, Path, State, ValueRef};

use super::Nodes;
use crate::contexts::LayoutCtx;
use crate::error::Result;
use crate::generator::expressions::Collection;
use crate::generator::Expression;
use crate::WidgetContainer;

#[derive(Debug)]
struct Iteration<'e> {
    body: Nodes<'e>,
    node_id: NodeId,
}

impl<'e> Iteration<'e> {
    pub fn new(expressions: &'e [Expression], node_id: NodeId) -> Self {
        Self {
            body: Nodes::new(expressions, node_id.child(0)),
            node_id,
        }
    }
}

// -----------------------------------------------------------------------------
//   - Loop -
// -----------------------------------------------------------------------------
#[derive(Debug)]
pub struct LoopNode<'e> {
    expressions: &'e [Expression],
    iterations: Vec<Iteration<'e>>,
    current_iteration: usize,
    pub(super) binding: Path,
    pub(super) collection: Collection<'e>,
    pub(super) value_index: usize,
    node_id: NodeId,
}

impl<'e> LoopNode<'e> {
    pub(crate) fn new(
        expressions: &'e [Expression],
        binding: Path,
        collection: Collection<'e>,
        node_id: NodeId,
    ) -> Self {
        Self {
            expressions,
            iterations: vec![],
            binding,
            collection,
            value_index: 0,
            current_iteration: 0,
            node_id,
        }
    }

    pub(super) fn next<F>(
        &mut self,
        context: &Context<'_, 'e>,
        layout: &LayoutCtx,
        f: &mut F,
    ) -> Result<ControlFlow<(), ()>>
    where
        F: FnMut(&mut WidgetContainer<'e>, &mut Nodes<'e>, &Context<'_, 'e>) -> Result<()>,
    {
        loop {
            let Some(value) = self.next_value(context) else {
                return Ok(ControlFlow::Continue(()));
            };

            let scope = LocalScope::new(self.binding.clone(), value);
            let context = context.reparent(&scope);

            let iter = match self.iterations.get_mut(self.current_iteration) {
                Some(iter) => iter,
                None => {
                    self.iterations
                        .push(Iteration::new(self.expressions, self.node_id.next()));
                    &mut self.iterations[self.current_iteration]
                }
            };

            while let Some(res) = iter.body.next(&context, layout, f) {
                match res? {
                    ControlFlow::Continue(()) => continue,
                    ControlFlow::Break(()) => break,
                }
            }
            self.current_iteration += 1;
        }
    }

    pub(super) fn reset_cache(&mut self) {
        self.current_iteration = 0;
        self.value_index = 0;
        self.iterations
            .iter_mut()
            .for_each(|i| i.body.reset_cache());
    }

    pub(super) fn count(&self) -> usize {
        self.iterations.iter().map(|i| i.body.count()).sum()
    }

    pub(super) fn next_value(&mut self, context: &Context<'_, 'e>) -> Option<ValueRef<'e>> {
        let val = match self.collection {
            Collection::ValueExpressions(expressions) => {
                let value = expressions.get(self.value_index)?;
                self.value_index += 1;
                value.eval(&mut Deferred::new(context))?
            }
            Collection::Path(ref path) => context.lookup(path)?,
            Collection::State { len, .. } if len == self.value_index => return None,
            Collection::State {
                ref path,
                ref mut next,
                ..
            } => {
                let path = match next.pop() {
                    Some(value_index) => path.compose(value_index),
                    None => {
                        let path = path.compose(self.value_index);
                        self.value_index += 1;
                        path
                    }
                };
                let s = path.to_string();
                ValueRef::Deferred(path)
            }
            Collection::Empty => return None,
        };
        Some(val)
    }

    pub(super) fn remove(&mut self, index: usize) {
        self.collection.remove();
        if index >= self.iterations.len() {
            return;
        }
        self.value_index -= 1;
        self.current_iteration = self.iterations.len() - 1;
        self.iterations.remove(index);
    }

    pub(super) fn push(&mut self) {
        self.value_index = self.iterations.len();
        self.collection.push();
    }

    pub(super) fn insert(&mut self, index: usize) {
        self.collection.insert(index);
        self.iterations.insert(index, Iteration::new(self.expressions, self.node_id.next()));
    }

    pub(super) fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&mut WidgetContainer<'e>, &mut Nodes<'e>)> + '_ {
        self.iterations.iter_mut().flat_map(|i| i.body.iter_mut())
    }

    pub(super) fn update(&mut self, node_id: &[usize], change: Change, context: &Context<'_, '_>) {
        for iter in &mut self.iterations {
            if iter.node_id.contains(node_id) {
                iter.body.update(node_id, change, context);
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use anathema_values::testing::TestState;
    use anathema_values::ValueExpr;

    use super::*;
    use crate::testing::{expression, for_expression};

    #[test]
    fn nested_next_value() {
        let state = TestState::new();
        let context = Context::root(&state);
        let outer = for_expression(
            "outer",
            ValueExpr::Ident("nested_list".into()).into(),
            vec![for_expression(
                "inner",
                ValueExpr::Ident("outer".into()).into(),
                vec![expression("test", ValueExpr::Ident("inner".into()), [], [])],
            )],
        );

        let xx = outer.eval(&context, 0.into()).unwrap();

        let y = xx;
    }
}
