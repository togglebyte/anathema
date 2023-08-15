use anathema_values::StoreRef;
use expression::EvaluationContext;

pub use crate::ctx::DataCtx;
pub use crate::expression::{ControlFlowExpr, Expression, FromContext};
pub use crate::nodes::{Nodes, NodeId};
pub use crate::values::{ExpressionValue, ExpressionValues};

mod ctx;
mod expression;
mod nodes;
mod testing;
mod values;

pub fn make_it_so<'a, T: FromContext>(
    expressions: Vec<Expression<T>>,
    store: StoreRef<T::Value>,
) -> Result<Nodes<T>, T::Err> {
    let eval = EvaluationContext::new(&store, None);

    let nodes = expressions
        .into_iter()
        .enumerate()
        .map(|(i, expr)| expr.to_node(&eval, NodeId::new(i)))
        .collect::<Result<_, _>>()?;

    Ok(Nodes::new(nodes))
}
