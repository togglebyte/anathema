use anathema_values::{Attributes, Path, ValueExpr};

use crate::generator::{ControlFlow, ElseExpr, Expression, IfExpr, Loop, SingleNode};

pub fn expression<Txt>(
    ident: impl Into<String>,
    text: Option<Txt>,
    attributes: impl Into<Attributes>,
    children: impl Into<Vec<Expression>>,
) -> Expression
where
    ValueExpr: From<Txt>,
{
    let children = children.into();
    let text: Option<ValueExpr> = text.map(|t| ValueExpr::from(t));
    Expression::Node(SingleNode {
        ident: ident.into(),
        text,
        attributes: attributes.into(),
        children: children.into(),
    })
}

pub(crate) fn for_expression(
    binding: impl Into<Path>,
    collection: Box<ValueExpr>,
    body: impl Into<Vec<Expression>>,
) -> Expression {
    Expression::Loop(Loop {
        body: body.into().into(),
        binding: binding.into(),
        collection: *collection,
    })
}

pub(crate) fn if_expression(
    if_true: (ValueExpr, Vec<Expression>),
    elses: Vec<(Option<ValueExpr>, Vec<Expression>)>,
    //     binding: impl Into<Path>,
    //     collection: Box<ValueExpr>,
    //     body: impl Into<Vec<Expression>>,
) -> Expression {
    Expression::ControlFlow(ControlFlow {
        if_expr: IfExpr {
            cond: if_true.0,
            expressions: if_true.1,
        },
        elses: elses
            .into_iter()
            .map(|(cond, body)| ElseExpr {
                cond,
                expressions: body,
            })
            .collect(),
    })
}
