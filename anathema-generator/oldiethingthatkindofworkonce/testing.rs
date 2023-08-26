use std::sync::Arc;

use anathema_values::{Container, Listen, PathId, Store, Truthy, ValueRef};

use crate::expression::ControlFlowExpr;
use crate::{Expression, ExpressionValue, ExpressionValues, FromContext, NodeId};

// -----------------------------------------------------------------------------
//   - Helper impls -
// -----------------------------------------------------------------------------
impl<T: Clone> Into<ExpressionValues<T>> for Vec<(String, T)> {
    fn into(self) -> ExpressionValues<T> {
        let mut values = ExpressionValues::empty();
        for (k, v) in self {
            values.set(k, ExpressionValue::Static(v.into()));
        }
        values
    }
}

impl<T: Clone, const N: usize> Into<ExpressionValues<T>> for [(String, T); N] {
    fn into(self) -> ExpressionValues<T> {
        let mut values = ExpressionValues::empty();
        for (k, v) in self {
            values.set(k, ExpressionValue::Static(v.into()));
        }
        values
    }
}

impl<T: Clone> Into<ExpressionValues<T>> for () {
    fn into(self) -> ExpressionValues<T> {
        ExpressionValues::empty()
    }
}

impl<T> From<T> for ExpressionValue<T> {
    fn from(value: T) -> Self {
        ExpressionValue::Static(value.into())
    }
}

impl<T, const N: usize> From<[T; N]> for ExpressionValue<T> {
    fn from(values: [T; N]) -> Self {
        ExpressionValue::List(values.map(Into::into).into())
    }
}

pub(crate) struct Expressions<T: FromContext>(Vec<Expression<T>>);

impl<T: FromContext> Into<Expressions<T>> for Expression<T> {
    fn into(self) -> Expressions<T> {
        Expressions(vec![self])
    }
}

impl<T: FromContext, E> Into<Expressions<T>> for Vec<E>
where
    E: Into<Expression<T>>,
{
    fn into(self) -> Expressions<T> {
        let mut output = vec![];
        for expr in self {
            output.push(expr.into())
        }
        Expressions(output)
    }
}

impl<T: FromContext, const N: usize, E> Into<Expressions<T>> for [E; N]
where
    E: Into<Expression<T>>,
{
    fn into(self) -> Expressions<T> {
        let mut output = vec![];
        for expr in self {
            output.push(expr.into())
        }
        Expressions(output)
    }
}

impl<T: FromContext> Into<Expressions<T>> for () {
    fn into(self) -> Expressions<T> {
        Expressions(vec![])
    }
}

impl<T: Truthy> From<T> for ControlFlowExpr<T> {
    fn from(value: T) -> Self {
        ControlFlowExpr::If(ExpressionValue::Static(value.into()))
    }
}

impl<T> Into<ControlFlowExpr<T>> for Option<T> {
    fn into(self) -> ControlFlowExpr<T> {
        ControlFlowExpr::Else(self.map(|val| ExpressionValue::Static(val.into())))
    }
}

// -----------------------------------------------------------------------------
//   - Listener -
// -----------------------------------------------------------------------------
pub(crate) struct Listener;

impl Listen for Listener {
    type Key = NodeId;
    type Value = u32;

    fn subscribe(value: ValueRef<Container<Self::Value>>, key: Self::Key) {}
}

// -----------------------------------------------------------------------------
//   - Test widget -
// -----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct Widget {
    pub ident: &'static str,
}

impl FromContext for Widget {
    type Ctx = &'static str;
    type Err = ();
    type Notifier = crate::testing::Listener;
    type Value = u32;

    fn from_context(ctx: crate::DataCtx<'_, Self>) -> Result<Self, Self::Err> {
        Ok(Self { ident: &*ctx })
    }
}

pub(crate) fn expression(
    context: &'static str,
    attributes: impl Into<ExpressionValues<<Widget as FromContext>::Value>>,
    children: impl Into<Expressions<Widget>>,
) -> Expression<Widget> {
    Expression::Node {
        context,
        attributes: attributes.into(),
        children: children.into().0.into(),
    }
}

pub(crate) fn for_expression(
    binding: impl Into<PathId>,
    collection: impl Into<ExpressionValue<<Widget as FromContext>::Value>>,
    body: impl Into<Expressions<Widget>>,
) -> Expression<Widget> {
    Expression::Loop {
        binding: binding.into(),
        body: body.into().0.into(),
        collection: collection.into(),
    }
}

pub(crate) fn controlflow<E>(
    flows: impl Into<Vec<(ControlFlowExpr<<Widget as FromContext>::Value>, E)>>,
) -> Expression<Widget>
where
    E: Into<Expressions<Widget>>,
{
    let flows = flows
        .into()
        .into_iter()
        .map(|(val, exprs)| (val, exprs.into().0.into()))
        .collect();

    Expression::ControlFlow(flows)
}
