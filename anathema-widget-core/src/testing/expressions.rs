use anathema_values::{Attributes, Path, ValueExpr};

use crate::generator::{ControlFlow, ElseExpr, Expression, IfExpr, Loop, SingleNode};

pub fn expression(
    ident: impl Into<String>,
    text: impl Into<Option<&'static str>>,
    attributes: impl Into<Attributes>,
    children: impl Into<Vec<Expression>>,
) -> Expression {
    let children = children.into();
    Expression::Node(SingleNode {
        ident: ident.into(),
        text: text.into().map(|s| ValueExpr::String(s.into())),
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
