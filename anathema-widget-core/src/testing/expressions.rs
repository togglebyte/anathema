use anathema_values::testing::ident;
use anathema_values::{Attributes, Path, ValueExpr};

use crate::expressions::{
    ControlFlow, ElseExpr, Expression, IfExpr, LoopExpr, SingleNodeExpr, ViewExpr,
};

pub fn view(name: &str, body: impl Into<Vec<Expression>>) -> Expression {
    panic!()
    // Expression::View(ViewExpr { ident: *ident(name), state: None })
}

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
        children: children.into(),
    })
}

pub(crate) fn for_expression(
    binding: impl Into<Path>,
    collection: Box<ValueExpr>,
    body: impl Into<Vec<Expression>>,
) -> Expression {
    Expression::Loop(LoopExpr {
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
