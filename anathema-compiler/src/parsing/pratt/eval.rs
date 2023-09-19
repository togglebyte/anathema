use anathema_values::{Num, Path, ScopeValue, ValueExpr};

use super::Expr;
use crate::token::Operator;
use crate::Constants;

pub fn eval(expr: Expr, consts: &Constants) -> ValueExpr {
    match expr {
        Expr::Name(string_id) => {
            let string = consts.lookup_string(string_id);
            let expr = ValueExpr::Ident(string.into());
            ValueExpr::Lookup(expr.into())
        }
        Expr::Num(num) => ValueExpr::Num(Num::Unsigned(num)),
        Expr::Array { lhs, index } => {
            // let ValueExpr::Path(lhs) = eval(*lhs, consts) else {
            //     panic!("we'll deal with you later");
            // };

            // let path = match eval(*index, consts) {
            //     ValueExpr::Path(path) => lhs.compose(path),
            //     ValueExpr::Num(Num::Unsigned(num)) => lhs.compose(num),
            //     _ => panic!("we'll deal with this later"),
            // };
            // ValueExpr::Path(path)
            let lhs = eval(*lhs, consts);
            let index = eval(*index, consts);
            ValueExpr::Index(lhs.into(), index.into())
        }
        Expr::Binary { op, lhs, rhs } => match op {
            Operator::Dot => {
                ValueExpr::Dot(eval(*lhs, consts).into(), eval(*rhs, consts).into())
            }
            _ => panic!(),
        },
        Expr::Unary { op, expr } => {
            let expr = eval(*expr, consts);

            match op {
                Operator::Not => ValueExpr::Bool(expr.into()),
                Operator::Minus => match expr {
                    ValueExpr::Num(Num::Unsigned(n)) => ValueExpr::Num(Num::Signed(-(n as i64))),
                    _ => expr,
                }
                _ => panic!(),
            }
        }
        _ => panic!(),
    }

    // let y = [1, 2, 3]
    // let a = 5
    // let x = 10
    //
    //
    // text "hello {{ name }}, how are your {{ 10 + counter - 8 }} babies doing today"
    //
    // vstack
    //     text "{{ x }}" // prints 10
    //     for x in y.a.b
    //         text "hello {{ x + a }}"
    //         text "{{ x }}" // prints 1, 2 or 3

    // {{ x + a }} should be stored as a singular expression
    //
    // for x in make_range(1, 10)
    //     text "{{ x }}"

    // 1 + a -> Expression:Add(
    //     ScopeValue::Static(1), ScopeValue::Dyn(a)
    // )
    // a.b.c -> Path::Composite(a, Path::Composite(b, c));
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

        let expr = eval_str("-123");
        assert_eq!(expr.to_string(), "-123");
    }
}
