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
fn eval_path(expr: Expression, ctx: &Context<'_>) -> Option<Expression> {
    use {Expression as E, Primitive as P};

    match expr {
        // Don't return None here, as this is possibly the ident of a state value,
        // or an attribute
        E::Ident(ref ident) => Some(ctx.fetch(ident).map(Expression::Variable).unwrap_or(expr)),
        E::Str(ref strlit) => Some(ctx.fetch(strlit).map(Expression::Variable).unwrap_or(expr)),
        E::Index(lhs, rhs) => {
            let lhs = const_eval(lhs, ctx)?;
            let rhs = const_eval(rhs, ctx)?;

            match (&lhs, &rhs) {
                (E::List(list), E::Primitive(P::Int(index))) => list.get(*index as usize).cloned(),
                (E::Map(map), E::Str(key)) => map.get(key).cloned(),
                _ => Some(E::Index(lhs.into(), rhs.into())),
            }
        }
        _ => Some(expr),
    }
}

// Returning `None` here means we evaluated a const expression but the expression didn't exist,
// e.g indexing outside of a list of primitives or refering to state value / attributes.
pub(crate) fn const_eval(expr: impl Into<Expression>, ctx: &Context<'_>) -> Option<Expression> {
    use {Expression as E, Primitive as P};

    macro_rules! ce {
        ($e:expr) => {
            const_eval($e, ctx)?.into()
        };
    }

    let expr = expr.into();

    let expr = match expr {
        expr @ (E::Primitive(_) | E::Str(_)) => expr,
        E::Either(first, second) => match const_eval(first, ctx) {
            Some(expr @ (E::Primitive(_) | E::Str(_))) => expr,
            Some(expr) => E::Either(expr.into(), ce!(second)),
            None => return None,
        },
        E::Not(expr) => E::Not(ce!(expr)),
        E::Negative(expr) => E::Negative(ce!(expr)),
        E::Equality(lhs, rhs, eq) => E::Equality(ce!(lhs), ce!(rhs), eq),
        E::LogicalOp(lhs, rhs, op) => E::LogicalOp(ce!(lhs), ce!(rhs), op),
        E::Range(from, to) => E::Range(ce!(from), ce!(to)),

        E::Ident(_) | E::Index(..) => eval_path(expr, ctx)?,
        E::Variable(_) => unreachable!("const eval is not recursive so this can never happen"),

        E::List(list) => {
            let list = list.into_iter().filter_map(|expr| ce!(expr)).collect();
            E::List(list)
        }

        E::TextSegments(segments) => {
            let segments = segments.into_iter().filter_map(|expr| ce!(expr)).collect();
            E::TextSegments(segments)
        }
        E::Map(map) => {
            let hm = HashMap::from_iter(map.into_iter().flat_map(|(k, v)| Some((k, ce!(v)))));
            E::Map(hm)
        }
        E::Op(lhs, rhs, op) => match (ce!(lhs), ce!(rhs)) {
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
            fun,
            args: args.into_iter().filter_map(|expr| ce!(expr)).collect(),
        },
    };

    Some(expr)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::expressions::{add, div, either, ident, index, list, mul, num, range, strlit, sub};
    use crate::statements::with_context;

    #[test]
    fn addition() {
        with_context(|ctx| {
            let expr = add(num(1), num(2));

            let output = const_eval(expr, &ctx).unwrap();
            assert_eq!(output, *num(3));
        });
    }

    #[test]
    fn subtract() {
        with_context(|ctx| {
            let expr = sub(num(1), num(2));

            let output = const_eval(expr, &ctx).unwrap();
            assert_eq!(output, *num(-1));
        });
    }

    #[test]
    fn multiply() {
        with_context(|ctx| {
            let expr = mul(num(2), num(2));

            let output = const_eval(expr, &ctx).unwrap();
            assert_eq!(output, *num(4));
        });
    }

    #[test]
    fn divide() {
        with_context(|ctx| {
            let expr = div(num(2), num(2));

            let output = const_eval(expr, &ctx).unwrap();
            assert_eq!(output, *num(1));
        });
    }

    #[test]
    fn const_index_resolve() {
        with_context(|ctx| {
            let expr = index(list([1, 2, 3]), num(2));
            let output = const_eval(expr, &ctx).unwrap();
            assert_eq!(output, *num(3));
        });
    }

    #[test]
    fn const_index_lookup_of_state() {
        with_context(|ctx| {
            let expr = index(index(ident("state"), ident("list")), num(2));
            let output = const_eval(expr.clone(), &ctx).unwrap();
            assert_eq!(output, *expr);
        });
    }

    #[test]
    fn const_either() {
        // TODO: this is yet to be implemented in const folding
        with_context(|ctx| {
            let expr = either(strlit("tea time"), ident("thing"));
            let output = const_eval(expr, &ctx).unwrap();
            let expected = strlit("tea time");
            assert_eq!(output, *expected);
        });

        with_context(|ctx| {
            let expr = either(ident("teatime"), ident("thing"));
            let output = const_eval(expr.clone(), &ctx).unwrap();
            assert_eq!(output, *expr);
        });
    }

    #[test]
    fn const_range() {
        with_context(|ctx| {
            let expr = range(num(1), num(2));

            let output = const_eval(expr, &ctx).unwrap();
            assert_eq!(output, *range(num(1), num(2)));
        });
    }
}
