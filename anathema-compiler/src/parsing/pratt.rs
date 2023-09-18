use std::fmt::{self, Display};

use crate::{Constants, StringId};
use crate::error::{Error, ErrorKind, Result};
use crate::lexer::Lexer;
use crate::token::{Kind, Operator, Token, Value, Tokens};

struct PrattParser<'src, 'tokens> {
    tokens: &'tokens mut Tokens,
    consts: &'tokens mut Constants,
    src: &'src str,
}

// Parser -> PrattParser -> Expr -> OuterExpr

impl<'src, 'tokens> PrattParser<'src, 'tokens> {
    pub fn new(tokens: &'tokens mut Tokens, src: &'src str, consts: &'tokens mut Constants) -> Self {
        Self {
            tokens,
            consts,
            src,
        }
    }
}

pub mod prec {
    pub const ASSIGNMENT: u8 = 1;
    pub const CONDITIONAL: u8 = 2;
    pub const SUM: u8 = 3;
    pub const PRODUCT: u8 = 4;
    pub const PREFIX: u8 = 6;
    pub const CALL: u8 = 8;
    pub const SELECTION: u8 = 9;
    pub const SUBCRIPT: u8 = 10;
}

fn get_precedence(op: Operator) -> u8 {
    match op {
        Operator::Equal => prec::ASSIGNMENT,
        Operator::Or | Operator::And => prec::CONDITIONAL,
        Operator::Plus | Operator::Minus => prec::SUM,
        Operator::Mul | Operator::Div => prec::PRODUCT,
        Operator::LParen => prec::CALL,
        Operator::Dot => prec::SELECTION,
        Operator::RBracket => prec::SUBCRIPT,
        _ => 0,
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Unary {
        op: Operator,
        expr: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Operator,
    },
    Bool(bool),
    Num(u64),
    Name(StringId),
    Call { fun: Box<Expr>, args: Vec<Expr> },
    Array {
        lhs: Box<Expr>,
        index: Box<Expr>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Unary { op, expr } => write!(f, "({op}{expr})"),
            Expr::Binary { op, lhs, rhs } => write!(f, "({op} {lhs} {rhs})"),
            Expr::Bool(b) => write!(f, "{b}"),
            Expr::Num(b) => write!(f, "{b}"),
            Expr::Name(b) => write!(f, "{b}"),
            Expr::Array { lhs, index } => write!(f, "{lhs}[{index}]"),
            Expr::Call { fun, args } => {
                let s = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{fun}({s})")
            }
        }
    }
}

pub(crate) fn expr(tokens: &mut Tokens) -> Expr {
    expr_bp(tokens, 0)
}

fn expr_bp(tokens: &mut Tokens, precedence: u8) -> Expr {
    let mut left = match tokens.next().0 {
        Kind::Op(Operator::LParen) => {
            let left = expr(tokens);
            // Need to consume the closing bracket
            assert!(matches!(tokens.next().0, Kind::Op(Operator::RParen)));
            left
        }
        Kind::Op(op) => Expr::Unary {
            op,
            expr: Box::new(expr_bp(tokens, prec::PREFIX)),
        },
        Kind::Value(value) => match value {
            Value::Number(n) => Expr::Num(n),
            Value::Ident(ident) => Expr::Name(ident),
            Value::Bool(b) => Expr::Bool(b),
            // TODO: see panic
            _ => panic!("need to cover the rest of the values"),
        }
        Kind::Eof => panic!("unexpected eof"),
        // TODO: see panic
        _ => panic!("we'll deal with this later"),
    };

    loop {
        // postfix operators are just operators in the "LED" context that consume no right expression

        // This could be EOF, which is fine.
        // It could also be any other token which would be
        // a syntax error, but I don't mind that just now
        let Kind::Op(op) = tokens.peek().0 else { return left; };

        let token_prec = get_precedence(op);

        // If the current token precedence is higher than the current precedence, then we bind to the right,
        // otherwise we bind to the left
        if precedence >= token_prec {
            break;
        }

        tokens.consume();

        // Postfix parsing
        match op {
            Operator::LParen => {
                left = parse_function(tokens, left);
                continue;
            }
            Operator::LBracket => {
                left = Expr::Array {
                    lhs: Box::new(left),
                    index: Box::new(expr(tokens)),
                };
                let Kind::Op(Operator::RBracket) = tokens.next().0 else {
                    panic!("invalid token");
                };
                continue;
            }
            _ => {}
        }

        let right = expr_bp(tokens, token_prec);
        left = Expr::Binary {
            lhs: Box::new(left),
            op,
            rhs: Box::new(right),
        };

        continue;
    }

    left
}

fn parse_function(tokens: &mut Tokens, left: Expr) -> Expr {
    let mut args = vec![];

    loop {
        match tokens.peek().0 {
            Kind::Op(Operator::Comma) => {
                tokens.consume();
                continue;
            }
            Kind::Op(Operator::RParen) => {
                tokens.consume();
                break;
            }
            t => ()
        }
        args.push(expr(tokens));
    }

    Expr::Call { fun: Box::new(left), args }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(input: &str) -> String {
        let mut lexer = Lexer::new(input);
        expr(&mut lexer).to_string()
    }

    #[test]
    fn add_sub() {
        let input = "1 + 2";
        assert_eq!(parse(input), "(+ 1 2)");

        let input = "1 - 2";
        assert_eq!(parse(input), "(- 1 2)");
    }

    #[test]
    fn mul_div() {
        let input = "5 + 1 * 2";
        assert_eq!(parse(input), "(+ 5 (* 1 2))");

        let input = "5 - 1 / 2";
        assert_eq!(parse(input), "(- 5 (/ 1 2))");
    }

    #[test]
    fn brackets() {
        let input = "(5 + 1) * 2";
        assert_eq!(parse(input), "(* (+ 5 1) 2)");
    }

    #[test]
    fn function() {
        let input = "f(1, a + 2 * 3, 3)";
        assert_eq!(parse(input), "f(1, (+ a (* 2 3)), 3)");
    }

    #[test]
    fn function_no_args() {
        let input = "f()";
        assert_eq!(parse(input), "f()");
    }
}
