use std::rc::Rc;

use crate::generator::expressions::Expression;

#[derive(Debug)]
pub struct If {
    pub cond: (), //ScopeValue,
    pub body: Rc<[Expression]>,
}

#[derive(Debug)]
pub struct Else {
    pub cond: Option<()>, //Option<ScopeValue>,
    pub body: Rc<[Expression]>,
}
