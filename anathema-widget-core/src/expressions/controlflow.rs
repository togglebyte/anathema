use anathema_values::ValueExpr;

use super::Expression;

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: ValueExpr,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct ElseExpr {
    pub cond: Option<ValueExpr>,
    pub expressions: Vec<Expression>,
}
