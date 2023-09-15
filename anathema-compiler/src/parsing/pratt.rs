use std::fmt::{self, Display};

use crate::error::{Error, ErrorKind, Result};
use crate::lexer::{Kind, Lexer, Token, Value};
use crate::operator::Operator;

mod precedence {
    pub(super) const Initial: u8 = 0;
    pub(super) const Assignment: u8 = 1;
    pub(super) const ModAssign: u8 = 2;
    pub(super) const DivAssign: u8 = 3;
    pub(super) const MulAssign: u8 = 4;
    pub(super) const SubAssign: u8 = 5;
    pub(super) const AddAssign: u8 = 6;
    pub(super) const Or: u8 = 7;
    pub(super) const And: u8 = 8;
    pub(super) const Equality: u8 = 9;
    pub(super) const LessGreater: u8 = 10;
    pub(super) const AddSub: u8 = 11;
    pub(super) const DivMulMod: u8 = 12;
    pub(super) const Prefix: u8 = 13;
    pub(super) const Call: u8 = 14;
}

struct PrattLexer<'src, 'consts> {
    inner: &'src mut Lexer<'src, 'consts>,
}

impl<'src, 'consts> PrattLexer<'src, 'consts> {
    fn new(inner: &'src mut Lexer<'src, 'consts>) -> Self {
        Self { inner }
    }

    fn next(&mut self) -> Result<Token> {
        let token = self.inner.next()?;
        self.inner.consume(true, false);
        Ok(token)
    }

    fn is_next_token(&mut self, kind: Kind) -> Result<bool> {
        self.inner.is_next_token(kind)
    }

    fn peek_op(&mut self) -> Option<Operator> {
        self.inner.peek_op()
    }

    fn error(&mut self, error_kind: ErrorKind) -> Error {
        self.inner.error(error_kind)
    }

    fn unexpected_token(&self, msg: &str) -> Error {
        self.inner.error(ErrorKind::UnexpectedToken(msg.into()))
    }
}

struct PrattParser<'src, 'consts> {
    lexer: PrattLexer<'src, 'consts>,
}

#[derive(Debug)]
enum Expr {
    Unary {
        op: Operator,
        expr: Box<Expr>,
    },
    Binary {
        op: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Value(Value),
}

impl Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => write!(f, "{value}"),
            Self::Unary { op, expr } => write!(f, "{op}{expr}"),
            Self::Binary { op, lhs, rhs } => write!(f, "({op} {lhs} {rhs})"),
        }
    }
}

impl<'src, 'consts> PrattParser<'src, 'consts> {
    pub fn new(lexer: &'src mut Lexer<'src, 'consts>) -> Self {
        Self {
            lexer: PrattLexer::new(lexer),
        }
    }

    pub fn expr(&mut self) -> Result<Expr> {
        self.expr_bp(precedence::Initial)
    }

    fn prefix_binding_power(&mut self, op: Operator) -> Result<u8> {
        let prec = match op {
            Operator::Not | Operator::Plus | Operator::Minus => precedence::Prefix,
            _ => return Err(self.lexer.error(ErrorKind::InvalidOperator(op))),
        };

        Ok(prec)
    }

    // Expression with binding power
    fn expr_bp(&mut self, precedence: u8) -> Result<Expr> {
        let mut left = match self.lexer.next()?.0 {
            Kind::Op(Operator::LParen) => {
                let left = self.expr();
                // TODO return err if the next expr is not RParen
                left?
            }
            Kind::Op(op) => {
                let right_bp = self.prefix_binding_power(op)?;
                let expr = Expr::Unary {
                    op,
                    expr: Box::new(self.expr_bp(right_bp)?),
                };
                return Ok(expr);
            }
            Kind::Value(value) => Expr::Value(value),
            _ => {
                return Err(self
                    .lexer
                    .unexpected_token("expected either a value or an operator"))
            }
        };

        loop {
            if let Ok(true) = self.lexer.is_next_token(Kind::Eof) {
                break;
            }

            let Some(op) = self.lexer.peek_op() else {
                break;
            };

            if let Some((l_prec, r_prec)) = infix_binding_power(op) {
                if l_prec < precedence {
                    break;
                }

                self.lexer.next();

                left = Expr::Binary {
                    lhs: Box::new(left),
                    rhs: Box::new(self.expr()?),
                    op,
                };

                continue;
            }

            break;
        }

        Ok(left)
    }
}

fn infix_binding_power(op: Operator) -> Option<(u8, u8)> {
    let prec = match op {
        Operator::Equal => precedence::Assignment,
        Operator::EqualEqual => precedence::Equality,
        Operator::And => precedence::And,
        Operator::Or => precedence::Or,
        Operator::LessThan
        | Operator::LessThanOrEqual
        | Operator::GreaterThan
        | Operator::GreaterThanOrEqual => precedence::LessGreater,
        Operator::PlusEqual => precedence::AddAssign,
        Operator::MinusEqual => precedence::SubAssign,
        Operator::MulEqual => precedence::MulAssign,
        Operator::DivEqual => precedence::DivAssign,
        Operator::ModEqual => precedence::ModAssign,

        Operator::Plus => precedence::AddSub,
        Operator::Minus => precedence::AddSub,
        Operator::Equal => precedence::Assignment,
        Operator::Mul => precedence::DivMulMod,
        Operator::Mod => precedence::DivMulMod,
        Operator::Div => precedence::DivMulMod,
        Operator::LParen | Operator::RParen | Operator::Not => return None,
    };

    Some((prec - 1, prec))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Constants;

    fn prat(input: &str) -> Expr {
        let mut consts = Constants::new();
        let mut lexer = Lexer::new(input, &mut consts);
        let mut prat = PrattParser::new(&mut lexer);
        let expr = prat.expr().unwrap();
        expr
    }

    #[test]
    fn unary_expression() {
        let expr = prat("!true");
        assert_eq!(expr.to_string(), "!true");
    }

    #[test]
    fn binary_expression() {
        let expr = prat("a + b");
        assert_eq!(expr.to_string(), "(+ 0 1)");
    }

    #[test]
    fn binary_mul_prec() {
        let expr = prat("a + b * c");
        assert_eq!(expr.to_string(), "(+ 0 (* 1 2))");
    }
}
