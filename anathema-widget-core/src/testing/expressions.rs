use anathema_values::{Attributes, Path, ValueExpr};

use crate::expressions::{
    ControlFlow, ElseExpr, Expression, IfExpr, LoopExpr, SingleNodeExpr, ViewExpr,
};

pub fn expression(
    ident: impl Into<String>,
    text: impl Into<Option<ValueExpr>>,
    attributes: impl IntoIterator<Item = (String, ValueExpr)>,
    children: impl Into<Vec<Expression>>,
) -> Expression {
    let children = children.into();
    Expression::Node(SingleNodeExpr {
        ident: ident.into(),
        text: text.into(),
        attributes: Attributes::from_iter(attributes),
        children,
    })
}

pub fn for_expression(
    binding: impl Into<Path>,
    collection: Box<ValueExpr>,
    body: impl Into<Vec<Expression>>,
) -> Expression {
    Expression::Loop(LoopExpr {
        body: body.into(),
        binding: binding.into(),
        collection: collection.into(),
    })
}

pub fn if_expression(
    if_true: (ValueExpr, Vec<Expression>),
    elses: Vec<(Option<ValueExpr>, Vec<Expression>)>,
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

pub fn view_expression(
    id: impl Into<String>,
    state: Option<ValueExpr>,
    body: Vec<Expression>,
) -> Expression {
    Expression::View(ViewExpr {
        id: id.into(),
        state,
        body,
    })
}
