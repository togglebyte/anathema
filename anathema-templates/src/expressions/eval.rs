use std::collections::HashMap;
use std::rc::Rc;

use anathema_store::storage::strings::Strings;

use super::parser::Expr;
use super::{Expression, Op};
use crate::error::{ParseErrorKind, Result};
use crate::expressions::Equality;
use crate::token::Operator;

pub fn eval(expr: Expr, strings: &Strings) -> Result<Expression, ParseErrorKind> {
    let output = match expr {
        Expr::Primitive(val) => Expression::Primitive(val),
        Expr::Ident(string_id) => {
            let string = strings.get_unchecked(string_id);
            Expression::Ident(Rc::from(string))
        }
        Expr::Str(string_id) => {
            let string = strings.get_unchecked(string_id);
            Expression::Str(Rc::from(string))
        }
        Expr::Array { lhs, index } => {
            let lhs = eval(*lhs, strings)?;
            let index = eval(*index, strings)?;
            Expression::Index(lhs.into(), index.into())
        }
        Expr::Binary { op, lhs, rhs } => match op {
            Operator::Dot => {
                let lhs = eval(*lhs, strings)?.into();
                let string_id = match *rhs {
                    Expr::Ident(s) => s,
                    _ => {
                        return Err(ParseErrorKind::InvalidToken {
                            expected: "this can only be an ident",
                        })
                    }
                };
                let rhs = strings.get_unchecked(string_id);
                Expression::Index(lhs, Expression::Str(rhs.into()).into())
            }
            Operator::Mul | Operator::Plus | Operator::Minus | Operator::Div | Operator::Mod => {
                let (lhs, rhs) = (eval(*lhs, strings)?.into(), eval(*rhs, strings)?.into());
                let op = match op {
                    Operator::Mul => Op::Mul,
                    Operator::Plus => Op::Add,
                    Operator::Minus => Op::Sub,
                    Operator::Div => Op::Div,
                    Operator::Mod => Op::Mod,
                    _ => unreachable!(),
                };
                Expression::Op(lhs, rhs, op)
            }
            Operator::EqualEqual
            | Operator::NotEqual
            | Operator::GreaterThan
            | Operator::GreaterThanOrEqual
            | Operator::LessThan
            | Operator::LessThanOrEqual
            | Operator::And
            | Operator::Or => {
                let equality = match op {
                    Operator::EqualEqual => Equality::Eq,
                    Operator::NotEqual => Equality::NotEq,
                    Operator::GreaterThan => Equality::Gt,
                    Operator::GreaterThanOrEqual => Equality::Gte,
                    Operator::LessThan => Equality::Lt,
                    Operator::LessThanOrEqual => Equality::Lte,
                    Operator::And => Equality::And,
                    Operator::Or => Equality::Or,
                    _ => unreachable!(),
                };
                Expression::Equality(eval(*lhs, strings)?.into(), eval(*rhs, strings)?.into(), equality)
            }
            _ => return Err(ParseErrorKind::InvalidToken { expected: "" }),
        },
        Expr::Unary { op, expr } => {
            let expr = eval(*expr, strings)?;

            match op {
                Operator::Not => Expression::Not(expr.into()),
                Operator::Minus => Expression::Negative(expr.into()),
                _ => {
                    return Err(ParseErrorKind::InvalidToken {
                        expected: "either ! or -",
                    })
                }
            }
        }
        Expr::List(list) => Expression::List(
            list.into_iter()
                .map(|expr| eval(expr, strings))
                .collect::<Result<Vec<_>, _>>()?
                .into(),
        ),
        Expr::Map(map) => {
            let mut inner = HashMap::default();
            for (key, value) in map.into_iter() {
                let key = match eval(key, strings)? {
                    Expression::Str(s) | Expression::Ident(s) => s,
                    _ => return Err(ParseErrorKind::InvalidKey),
                };
                inner.insert(key, eval(value, strings)?);
            }

            Expression::Map(inner.into())
        }
        Expr::Call { fun, args } => {
            let args = args
                .into_iter()
                .map(|expr| eval(expr, strings))
                .collect::<Result<Vec<_>, _>>()?;

            Expression::Call {
                fun: eval(*fun, strings)?.into(),
                args: args.into_boxed_slice(),
            }
        }
    };

    Ok(output)
}

#[cfg(test)]
mod test {
    use anathema_store::storage::strings::Strings;

    use super::*;
    use crate::expressions::parser::parse_expr;
    use crate::lexer::Lexer;
    use crate::token::Tokens;

    fn eval_src(input: &str) -> Expression {
        let mut strings = Strings::empty();
        let lexer = Lexer::new(input, &mut strings);
        let tokens = lexer.collect::<Result<_, _>>().unwrap();
        let mut tokens = Tokens::new(tokens, input.len());
        parse_expr(&mut tokens, &strings).unwrap()
    }

    #[test]
    fn ident() {
        let expr = eval_src("ident");
        assert_eq!(expr.to_string(), "ident");
    }

    #[test]
    fn index() {
        let expr = eval_src("a[x]");
        assert_eq!(expr.to_string(), "a[x]");
    }

    #[test]
    fn dot() {
        let expr = eval_src("a.x.y");
        assert_eq!(expr.to_string(), "a[x][y]");
    }

    #[test]
    fn number() {
        let expr = eval_src("123");
        assert_eq!(expr.to_string(), "123");
    }

    #[test]
    fn negative_number() {
        let expr = eval_src("-123");
        assert_eq!(expr.to_string(), "-123");
    }

    #[test]
    fn lookup() {
        let expr = eval_src("a.b.c");
        assert_eq!(expr.to_string(), "a[b][c]");
    }

    #[test]
    fn bool() {
        let expr = eval_src("true");
        assert_eq!(expr.to_string(), "true");

        let expr = eval_src("!true");
        assert_eq!(expr.to_string(), "!true");

        let expr = eval_src("!false");
        assert_eq!(expr.to_string(), "!false");

        let expr = eval_src("!!false");
        assert_eq!(expr.to_string(), "!!false");

        let expr = eval_src("!hello");
        assert_eq!(expr.to_string(), "!hello");

        let expr = eval_src("!!hello");
        assert_eq!(expr.to_string(), "!!hello");
    }

    #[test]
    fn strings() {
        let expr = eval_src("'single quote'");
        assert_eq!(expr.to_string(), "single quote");

        let expr = eval_src("\"double quote\"");
        assert_eq!(expr.to_string(), "double quote");
    }

    #[test]
    fn addition() {
        let expr = eval_src("-2 + -3");
        assert_eq!(expr.to_string(), "-2 + -3");

        let expr = eval_src("2 + -3");
        assert_eq!(expr.to_string(), "2 + -3");

        let expr = eval_src("2 + -1");
        assert_eq!(expr.to_string(), "2 + -1");

        let expr = eval_src("-3 + 2");
        assert_eq!(expr.to_string(), "-3 + 2");

        let expr = eval_src("-1 + 2");
        assert_eq!(expr.to_string(), "-1 + 2");

        let expr = eval_src("1 + 2 * 3");
        assert_eq!(expr.to_string(), "1 + 2 * 3");

        let expr = eval_src("a + b * c");
        assert_eq!(expr.to_string(), "a + b * c");
    }

    #[test]
    fn multiplication() {
        let expr = eval_src("2 * 2");
        assert_eq!(expr.to_string(), "2 * 2");

        let expr = eval_src("x * 2 * 2");
        assert_eq!(expr.to_string(), "x * 2 * 2");
    }

    #[test]
    fn subtraction() {
        let expr = eval_src("5 - 4");
        assert_eq!(expr.to_string(), "5 - 4");

        let expr = eval_src("-5 - 4");
        assert_eq!(expr.to_string(), "-5 - 4");

        let expr = eval_src("-5 - -4");
        assert_eq!(expr.to_string(), "-5 - -4");

        let expr = eval_src("a - b");
        assert_eq!(expr.to_string(), "a - b");
    }

    #[test]
    fn division() {
        let expr = eval_src("5 / 4");
        assert_eq!(expr.to_string(), "5 / 4");

        let expr = eval_src("a / b");
        assert_eq!(expr.to_string(), "a / b");

        let expr = eval_src("-a / b");
        assert_eq!(expr.to_string(), "-a / b");
    }

    #[test]
    fn modulo() {
        let expr = eval_src("5 % 4");
        assert_eq!(expr.to_string(), "5 % 4");

        let expr = eval_src("a % 4");
        assert_eq!(expr.to_string(), "a % 4");
    }

    #[test]
    fn function_call() {
        let expr = eval_src("fun(5, 4)");
        assert_eq!(expr.to_string(), "fun(5, 4)");
    }

    #[test]
    fn equality() {
        let expr = eval_src("1 == 1");
        assert_eq!(expr.to_string(), "1 == 1");
    }

    #[test]
    fn not_equal() {
        let expr = eval_src("1 != 1");
        assert_eq!(expr.to_string(), "1 != 1");
    }

    #[test]
    fn greater_than() {
        let expr = eval_src("1 > 1");
        assert_eq!(expr.to_string(), "1 > 1");
    }

    #[test]
    fn greater_than_or_equal_to() {
        let expr = eval_src("1 >= 1");
        assert_eq!(expr.to_string(), "1 >= 1");
    }

    #[test]
    fn less_than() {
        let expr = eval_src("1 < 1");
        assert_eq!(expr.to_string(), "1 < 1");
    }

    #[test]
    fn less_than_or_equal_to() {
        let expr = eval_src("1 <= 1");
        assert_eq!(expr.to_string(), "1 <= 1");
    }

    #[test]
    fn and() {
        let expr = eval_src("true && true");
        assert_eq!(expr.to_string(), "true && true");
    }

    #[test]
    fn or() {
        let expr = eval_src("true || true");
        assert_eq!(expr.to_string(), "true || true");
    }

    #[test]
    fn list() {
        let expr = eval_src("[1, 2, 3]");
        assert_eq!(expr.to_string(), "[1, 2, 3]");
    }

    #[test]
    fn map() {
        let expr = eval_src("[{a: 1}, {b: 89}]");
        assert_eq!(expr.to_string(), "[{a: 1}, {b: 89}]");
    }
}
