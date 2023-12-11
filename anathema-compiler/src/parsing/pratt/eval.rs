use std::rc::Rc;

use anathema_values::hashmap::HashMap;
use anathema_values::{Num, Owned, ValueExpr};

use super::Expr;
use crate::token::Operator;
use crate::Constants;

pub fn eval(expr: Expr, consts: &Constants) -> ValueExpr {
    match expr {
        Expr::Bool(b) => ValueExpr::from(b),
        Expr::Color(color) => ValueExpr::from(color),
        Expr::Ident(string_id) => {
            let string = consts.lookup_string(string_id);
            ValueExpr::Ident(string.into())
        }
        Expr::Str(string_id) => {
            let string = consts.lookup_string(string_id);
            ValueExpr::String(Rc::from(string))
        }
        Expr::Num(num) => ValueExpr::Owned(Owned::Num(num.into())),
        Expr::Array { lhs, index } => {
            let lhs = eval(*lhs, consts);
            let index = eval(*index, consts);
            ValueExpr::Index(lhs.into(), index.into())
        }
        Expr::Binary { op, lhs, rhs } => match op {
            Operator::Dot => ValueExpr::Dot(eval(*lhs, consts).into(), eval(*rhs, consts).into()),
            Operator::Mul | Operator::Plus | Operator::Minus | Operator::Div | Operator::Mod => {
                let (lhs, rhs) = match (eval(*lhs, consts), eval(*rhs, consts)) {
                    (ValueExpr::Owned(Owned::Num(lhs)), ValueExpr::Owned(Owned::Num(rhs))) => {
                        match op {
                            Operator::Mul => return ValueExpr::Owned(Owned::Num(lhs * rhs)),
                            Operator::Plus => return ValueExpr::Owned(Owned::Num(lhs + rhs)),
                            Operator::Minus => return ValueExpr::Owned(Owned::Num(lhs - rhs)),
                            Operator::Div => return ValueExpr::Owned(Owned::Num(lhs / rhs)),
                            Operator::Mod => return ValueExpr::Owned(Owned::Num(lhs % rhs)),
                            _ => unreachable!(),
                        }
                    }
                    (lhs, rhs) => (lhs.into(), rhs.into()),
                };

                match op {
                    Operator::Mul => ValueExpr::Mul(lhs, rhs),
                    Operator::Plus => ValueExpr::Add(lhs, rhs),
                    Operator::Minus => ValueExpr::Sub(lhs, rhs),
                    Operator::Div => ValueExpr::Div(lhs, rhs),
                    Operator::Mod => ValueExpr::Mod(lhs, rhs),
                    _ => unreachable!(),
                }
            }
            Operator::EqualEqual => {
                ValueExpr::Equality(eval(*lhs, consts).into(), eval(*rhs, consts).into())
            }
            Operator::Or | Operator::And => {
                let lhs = eval(*lhs, consts);
                let rhs = eval(*rhs, consts);
                match op {
                    Operator::Or => ValueExpr::Or(lhs.into(), rhs.into()),
                    Operator::And => ValueExpr::And(lhs.into(), rhs.into()),
                    _ => unreachable!(),
                }
            }
            e => panic!("here is a panic: {e:#?}"),
        },
        Expr::Unary { op, expr } => {
            let expr = eval(*expr, consts);

            match op {
                Operator::Not => match expr {
                    ValueExpr::Owned(Owned::Bool(b)) => ValueExpr::Owned((!b).into()),
                    _ => ValueExpr::Not(expr.into()),
                },
                Operator::Minus => match expr {
                    ValueExpr::Owned(Owned::Num(Num::Unsigned(n))) => {
                        ValueExpr::Owned(Owned::Num(Num::Signed(-(n as i64))))
                    }
                    _ => ValueExpr::Negative(expr.into()),
                },
                _ => panic!("operator: {op:#?}"),
            }
        }
        Expr::List(list) => {
            ValueExpr::List(list.into_iter().map(|expr| eval(expr, consts)).collect())
        }
        Expr::Map(map) => ValueExpr::Map(
            map.into_iter()
                .map(|(key, value)| (eval(key, consts).to_string(), eval(value, consts)))
                .collect::<HashMap<_, _>>()
                .into(),
        ),
        Expr::Call { .. } => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parsing::pratt::expr;
    use crate::token::Tokens;
    use crate::Constants;

    fn eval_str(input: &str) -> ValueExpr {
        let mut consts = Constants::new();
        let lexer = Lexer::new(input, &mut consts);
        let tokens = lexer.collect::<Result<_, _>>().unwrap();
        let mut tokens = Tokens::new(tokens, input.len());

        let expression = expr(&mut tokens);
        eval(expression, &consts)
    }

    #[test]
    fn ident() {
        let expr = eval_str("ident");
        assert_eq!(expr.to_string(), "ident");
    }

    #[test]
    fn index() {
        let expr = eval_str("a[x]");
        assert_eq!(expr.to_string(), "a[x]");
    }

    #[test]
    fn number() {
        let expr = eval_str("123");
        assert_eq!(expr.to_string(), "123");
    }

    #[test]
    fn negative_number() {
        let expr = eval_str("-123");
        assert_eq!(expr.to_string(), "-123");
    }

    #[test]
    fn lookup() {
        let expr = eval_str("a.b.c");
        assert_eq!(expr.to_string(), "a.b.c");
    }

    #[test]
    fn bool() {
        let expr = eval_str("true");
        assert_eq!(expr.to_string(), "true");

        let expr = eval_str("!true");
        assert_eq!(expr.to_string(), "false");

        let expr = eval_str("!false");
        assert_eq!(expr.to_string(), "true");

        let expr = eval_str("!!false");
        assert_eq!(expr.to_string(), "false");

        let expr = eval_str("!hello");
        assert_eq!(expr.to_string(), "!hello");

        let expr = eval_str("!!hello");
        assert_eq!(expr.to_string(), "!!hello");
    }

    #[test]
    fn strings() {
        let expr = eval_str("'single quote'");
        assert_eq!(expr.to_string(), "single quote");

        let expr = eval_str("\"double quote\"");
        assert_eq!(expr.to_string(), "double quote");
    }

    #[test]
    fn addition() {
        let expr = eval_str("-2 + -3");
        assert_eq!(expr.to_string(), "-5");

        let expr = eval_str("2 + -3");
        assert_eq!(expr.to_string(), "-1");

        let expr = eval_str("2 + -1");
        assert_eq!(expr.to_string(), "1");

        let expr = eval_str("-3 + 2");
        assert_eq!(expr.to_string(), "-1");

        let expr = eval_str("-1 + 2");
        assert_eq!(expr.to_string(), "1");

        let expr = eval_str("1 + 2 * 3");
        assert_eq!(expr.to_string(), "7");

        let expr = eval_str("a + b * c");
        assert_eq!(expr.to_string(), "a + b * c");
    }

    #[test]
    fn multiplication() {
        let expr = eval_str("2 * 2");
        assert_eq!(expr.to_string(), "4");

        let expr = eval_str("x * 2 * 2");
        assert_eq!(expr.to_string(), "x * 2 * 2");
    }

    #[test]
    fn subtraction() {
        let expr = eval_str("5 - 4");
        assert_eq!(expr.to_string(), "1");

        let expr = eval_str("-5 - 4");
        assert_eq!(expr.to_string(), "-9");

        let expr = eval_str("-5 - -4");
        assert_eq!(expr.to_string(), "-1");

        let expr = eval_str("a - b");
        assert_eq!(expr.to_string(), "a - b");
    }

    #[test]
    fn division() {
        let expr = eval_str("5 / 4");
        assert_eq!(expr.to_string(), "1");

        let expr = eval_str("a / b");
        assert_eq!(expr.to_string(), "a / b");

        let expr = eval_str("-a / b");
        assert_eq!(expr.to_string(), "-a / b");
    }

    #[test]
    fn modulo() {
        let expr = eval_str("5 % 4");
        assert_eq!(expr.to_string(), "1");

        let expr = eval_str("a % 4");
        assert_eq!(expr.to_string(), "a % 4");
    }
}
