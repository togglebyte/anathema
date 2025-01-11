use std::borrow::Cow;

use anathema_state::Subscriber;
use anathema_templates::expressions::{Equality, LogicalOp, Op};

use crate::expression::{Kind, ValueExpr};
use crate::value::ValueKind;

struct Resolver<'e, 'bp> {
    expr: &'e ValueExpr<'bp>,
    sub: Subscriber,
}

impl<'e, 'bp> Resolver<'e, 'bp> {
    pub fn new(expr: &'e ValueExpr<'bp>, sub: Subscriber) -> Self {
        Self { expr, sub }
    }

    fn resolve(&self) -> Self {
        match self.expr {
            // -----------------------------------------------------------------------------
            //   - Primitives -
            // -----------------------------------------------------------------------------
            expr @ (ValueExpr::Bool(_)
            | ValueExpr::Char(_)
            | ValueExpr::Int(_)
            | ValueExpr::Float(_)
            | ValueExpr::Hex(_)
            | ValueExpr::Str(_)) => Self::new(expr, self.sub),

            // -----------------------------------------------------------------------------
            //   - Operations and conditionals -
            // -----------------------------------------------------------------------------
            ValueExpr::Not(value_expr) => {
                let ValueKind::Bool(val) = Self::new(value_expr, self.sub).finalise() else {
                    return ValueKind::Null;
                };
                ValueKind::Bool(!val)
            }
            ValueExpr::Negative(value_expr) => match Self::new(value_expr, self.sub).finalise() {
                ValueKind::Int(n) => ValueKind::Int(-n),
                ValueKind::Float(n) => ValueKind::Float(-n),
                _ => ValueKind::Null,
            },
            ValueExpr::Equality(lhs, rhs, equality) => {
                let lhs = Self::new(lhs, self.sub).finalise();
                let rhs = Self::new(rhs, self.sub).finalise();
                let b = match equality {
                    Equality::Eq => lhs == rhs,
                    Equality::NotEq => lhs != rhs,
                    Equality::Gt => lhs > rhs,
                    Equality::Gte => lhs >= rhs,
                    Equality::Lt => lhs < rhs,
                    Equality::Lte => lhs <= rhs,
                };
                ValueKind::Bool(b)
            }
            ValueExpr::LogicalOp(lhs, rhs, logical_op) => {
                let ValueKind::Bool(lhs) = Self::new(lhs, self.sub).finalise() else { return ValueKind::Null };
                let ValueKind::Bool(rhs) = Self::new(rhs, self.sub).finalise() else { return ValueKind::Null };
                let b = match logical_op {
                    LogicalOp::And => lhs && rhs,
                    LogicalOp::Or => lhs || rhs,
                };
                ValueKind::Bool(b)
            }
            ValueExpr::Op(lhs, rhs, op) => {
                match (Self::new(lhs, self.sub).finalise(), Self::new(rhs, self.sub).finalise()) {
                    (ValueKind::Int(lhs), ValueKind::Int(rhs)) => ValueKind::Int(int_op(lhs, rhs, *op)),
                    (ValueKind::Int(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs as f64, rhs, *op)),
                    (ValueKind::Float(lhs), ValueKind::Int(rhs)) => ValueKind::Float(float_op(lhs, rhs as f64, *op)),
                    (ValueKind::Float(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs, rhs, *op)),
                    _ => ValueKind::Null,
                }
            }
            ValueExpr::Either(first, second) => match Self::new(first, self.sub).finalise() {
                ValueKind::Null => Self::new(second, self.sub).finalise(),
                first => first,
            },

            // -----------------------------------------------------------------------------
            //   - Maps and lists -
            // -----------------------------------------------------------------------------
            ValueExpr::DynMap(value) => {
                let state = value.as_state().unwrap();
                state.as_any_map();
                panic!()
            }
            ValueExpr::DynList(pending_value) => todo!(),
            ValueExpr::List(_) => todo!(),
            ValueExpr::Map(hash_map) => todo!(),
            ValueExpr::Index(src, index) => {
                let expr = panic!(); //resolve_index(src, index, sub);
                Self::new(&expr, self.sub).finalise()
            }

            // -----------------------------------------------------------------------------
            //   - Call -
            // -----------------------------------------------------------------------------
            ValueExpr::Call => todo!(),

            // -----------------------------------------------------------------------------
            //   - Null -
            // -----------------------------------------------------------------------------
            ValueExpr::Null => ValueKind::Null,
        }
    }

    fn finalise(self) -> ValueKind<'bp> {
        match self.expr {
            // -----------------------------------------------------------------------------
            //   - Primitives -
            // -----------------------------------------------------------------------------
            ValueExpr::Bool(Kind::Static(b)) => ValueKind::Bool(*b),
            ValueExpr::Bool(Kind::Dyn(pending)) => {
                pending.subscribe(self.sub);
                let state = pending.as_state().unwrap();
                ValueKind::Bool(state.as_bool().unwrap())
            }
            ValueExpr::Char(Kind::Static(c)) => ValueKind::Char(*c),
            ValueExpr::Char(Kind::Dyn(pending)) => {
                pending.subscribe(self.sub);
                let state = pending.as_state().unwrap();
                ValueKind::Char(state.as_char().unwrap())
            }
            ValueExpr::Int(Kind::Static(i)) => ValueKind::Int(*i),
            ValueExpr::Int(Kind::Dyn(pending)) => {
                pending.subscribe(self.sub);
                let state = pending.as_state().unwrap();
                ValueKind::Int(state.as_int().unwrap())
            }
            ValueExpr::Float(Kind::Static(f)) => ValueKind::Float(*f),
            ValueExpr::Float(Kind::Dyn(pending)) => {
                pending.subscribe(self.sub);
                let state = pending.as_state().unwrap();
                ValueKind::Float(state.as_float().unwrap())
            }
            ValueExpr::Hex(Kind::Static(h)) => ValueKind::Hex(*h),
            ValueExpr::Hex(Kind::Dyn(pending)) => {
                pending.subscribe(self.sub);
                let state = pending.as_state().unwrap();
                ValueKind::Hex(state.as_hex().unwrap())
            }
            ValueExpr::Str(Kind::Static(s)) => ValueKind::Str(Cow::Borrowed(s)),
            ValueExpr::Str(Kind::Dyn(val)) => {
                let state = val.as_state().unwrap();
                let s = state.as_str().unwrap();
                ValueKind::Str(Cow::Owned(s.to_owned()))
            }

            // -----------------------------------------------------------------------------
            //   - Operations and conditionals -
            // -----------------------------------------------------------------------------
            ValueExpr::Not(value_expr) => {
                let ValueKind::Bool(val) = Self::new(value_expr, self.sub).finalise() else {
                    return ValueKind::Null;
                };
                ValueKind::Bool(!val)
            }
            ValueExpr::Negative(value_expr) => match Self::new(value_expr, self.sub).finalise() {
                ValueKind::Int(n) => ValueKind::Int(-n),
                ValueKind::Float(n) => ValueKind::Float(-n),
                _ => ValueKind::Null,
            },
            ValueExpr::Equality(lhs, rhs, equality) => {
                let lhs = Self::new(lhs, self.sub).finalise();
                let rhs = Self::new(rhs, self.sub).finalise();
                let b = match equality {
                    Equality::Eq => lhs == rhs,
                    Equality::NotEq => lhs != rhs,
                    Equality::Gt => lhs > rhs,
                    Equality::Gte => lhs >= rhs,
                    Equality::Lt => lhs < rhs,
                    Equality::Lte => lhs <= rhs,
                };
                ValueKind::Bool(b)
            }
            ValueExpr::LogicalOp(lhs, rhs, logical_op) => {
                let ValueKind::Bool(lhs) = Self::new(lhs, self.sub).finalise() else { return ValueKind::Null };
                let ValueKind::Bool(rhs) = Self::new(rhs, self.sub).finalise() else { return ValueKind::Null };
                let b = match logical_op {
                    LogicalOp::And => lhs && rhs,
                    LogicalOp::Or => lhs || rhs,
                };
                ValueKind::Bool(b)
            }
            ValueExpr::Op(lhs, rhs, op) => {
                match (Self::new(lhs, self.sub).finalise(), Self::new(rhs, self.sub).finalise()) {
                    (ValueKind::Int(lhs), ValueKind::Int(rhs)) => ValueKind::Int(int_op(lhs, rhs, *op)),
                    (ValueKind::Int(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs as f64, rhs, *op)),
                    (ValueKind::Float(lhs), ValueKind::Int(rhs)) => ValueKind::Float(float_op(lhs, rhs as f64, *op)),
                    (ValueKind::Float(lhs), ValueKind::Float(rhs)) => ValueKind::Float(float_op(lhs, rhs, *op)),
                    _ => ValueKind::Null,
                }
            }
            ValueExpr::Either(first, second) => match Self::new(first, self.sub).finalise() {
                ValueKind::Null => Self::new(second, self.sub).finalise(),
                first => first,
            },

            // -----------------------------------------------------------------------------
            //   - Maps and lists -
            // -----------------------------------------------------------------------------
            ValueExpr::DynMap(value) => {
                let state = value.as_state().unwrap();
                state.as_any_map();
                panic!()
            }
            ValueExpr::DynList(pending_value) => todo!(),
            ValueExpr::List(_) => todo!(),
            ValueExpr::Map(hash_map) => todo!(),
            ValueExpr::Index(src, index) => {
                let expr = panic!(); //resolve_index(src, index, sub);
                Self::new(&expr, self.sub).finalise()
            }

            // -----------------------------------------------------------------------------
            //   - Call -
            // -----------------------------------------------------------------------------
            ValueExpr::Call => todo!(),

            // -----------------------------------------------------------------------------
            //   - Null -
            // -----------------------------------------------------------------------------
            ValueExpr::Null => ValueKind::Null,
        }
    }
}

fn int_op(lhs: i64, rhs: i64, op: Op) -> i64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    }
}

fn float_op(lhs: f64, rhs: f64, op: Op) -> f64 {
    match op {
        Op::Add => lhs + rhs,
        Op::Sub => lhs - rhs,
        Op::Div => lhs / rhs,
        Op::Mul => lhs * rhs,
        Op::Mod => lhs % rhs,
    }
}
