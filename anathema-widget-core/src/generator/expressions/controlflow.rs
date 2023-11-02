use anathema_values::ValueExpr;

use super::Expression;

#[derive(Debug)]
pub struct IfExpr {
    pub cond: ValueExpr,
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub struct ElseExpr {
    pub cond: Option<ValueExpr>,
    pub expressions: Vec<Expression>,
}
