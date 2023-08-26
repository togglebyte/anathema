use anathema_values::State;
use expression::EvaluationContext;

pub use crate::ctx::DataCtx;
pub use crate::expression::{ControlFlowExpr, Expression, FromContext};
pub use crate::nodes::{NodeId, Nodes};
pub use crate::values::{ExpressionValue, ExpressionValues};

#[cfg(test)]
mod testing;

mod ctx;
mod expression;
mod nodes;

mod values;

pub fn make_it_so<'a, T: State>(
    expressions: Vec<Expression<T>>,
    state: &mut T
) -> Result<Nodes<T>, T::Err> {
    // let eval = EvaluationContext::new(state, None);

    let nodes = expressions
        .into_iter()
        .enumerate()
        .map(|(i, expr)| expr.to_node(state, NodeId::new(i)))
        .collect::<Result<_, _>>()?;

    Ok(Nodes::new(nodes))
}
