use std::collections::HashMap;

use super::Context;
use crate::expressions::{Expression, Op};
use crate::primitives::Primitive;

// Evaluate the expression using `vars` as a backing store.
// e.g `a.b.c` would first find `a` in `vars` and resolve the remaining path
// from within that expression.
//
// ```
// a.b[c]
// ```
// would resolve `a` from vars, `b` from `a`, and `c` from vars.
fn eval_path(expr: &Expression, ctx: &Context<'_>) -> Option<Expression> {
    use {Expression as E, Primitive as P};

    match expr {
        // NOTE: if `None` is not returned here then overriding globals in templates will fail
        E::Ident(_) => None, //ctx.fetch(ident),
        E::Str(strlit) => ctx.fetch(strlit),
        E::Index(lhs, rhs) => match eval_path(lhs, ctx)? {
            E::List(list) => match const_eval(rhs.clone(), ctx) {
                E::Primitive(P::Int(num)) => list.get(num as usize).cloned(),
                _ => Some(E::Index(
                    E::List(list.clone()).into(),
                    const_eval(*rhs.clone(), ctx).into(),
                )),
            },
            E::Map(map) => match const_eval(rhs.clone(), ctx) {
                E::Str(key) => map.get(&*key).cloned(),
                _ => Some(E::Index(
                    E::Map(map.clone()).into(),
                    const_eval(*rhs.clone(), ctx).into(),
                )),
            },
            index @ E::Index(..) => Some(E::Index(index.into(), const_eval(*rhs.clone(), ctx).into())),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn const_eval(expr: impl Into<Expression>, ctx: &Context<'_>) -> Expression {
    use {Expression as E, Primitive as P};

    macro_rules! ce {
        ($e:expr) => {
            const_eval($e, ctx).into()
        };
    }

    let expr = expr.into();
    match expr {
        expr @ (E::Primitive(_) | E::Str(_)) => expr,
        E::Not(expr) => E::Not(ce!(*expr)),
        E::Negative(expr) => E::Negative(ce!(*expr)),
        E::Equality(lhs, rhs, eq) => E::Equality(ce!(*lhs), ce!(*rhs), eq),

        E::Ident(_) => eval_path(&expr, ctx).map(|e| ce!(e)).unwrap_or(expr),
        E::Index(..) => eval_path(&expr, ctx).map(|e| ce!(e)).unwrap_or(expr),

        E::List(list) => {
            let list = list.iter().cloned().map(|expr| ce!(expr)).collect();
            E::List(list)
        }
        E::Map(map) => {
            let hm = HashMap::from_iter(map.iter().map(|(k, v)| (k.clone(), ce!(v.clone()))));
            E::Map(hm.into())
        }
        E::Op(lhs, rhs, op) => match (ce!(*lhs), ce!(*rhs)) {
            (E::Primitive(P::Int(lhs)), E::Primitive(P::Int(rhs))) => {
                let val = match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Div => lhs / rhs,
                    Op::Mul => lhs * rhs,
                    Op::Mod => lhs % rhs,
                };
                E::Primitive(P::Int(val))
            }
            (lhs, rhs) => E::Op(lhs.into(), rhs.into(), op),
        },
        E::Call { fun, args } => E::Call {
            fun: fun.clone(),
            args: args.iter().map(|expr| ce!(expr.clone())).collect(),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expressions::{add, div, mul, num, sub};
    use crate::statements::with_context;

    #[test]
    fn addition() {
        with_context(|ctx| {
            let expr = add(num(1), num(2));

            let output = const_eval(expr, &ctx);
            assert_eq!(output, *num(3));
        });
    }

    #[test]
    fn subtract() {
        with_context(|ctx| {
            let expr = sub(num(1), num(2));

            let output = const_eval(expr, &ctx);
            assert_eq!(output, *num(-1));
        });
    }

    #[test]
    fn multiply() {
        with_context(|ctx| {
            let expr = mul(num(2), num(2));

            let output = const_eval(expr, &ctx);
            assert_eq!(output, *num(4));
        });
    }

    #[test]
    fn divide() {
        with_context(|ctx| {
            let expr = div(num(2), num(2));

            let output = const_eval(expr, &ctx);
            assert_eq!(output, *num(1));
        });
    }
}
