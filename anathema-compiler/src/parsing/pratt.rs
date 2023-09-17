use std::fmt::{self, Display};

use crate::Constants;
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

fn get_precedence(op: char) -> u8 {
    match op {
        '=' => prec::ASSIGNMENT,
        '|' | '&' => prec::CONDITIONAL,
        '+' | '-' => prec::SUM,
        '*' | '/' => prec::PRODUCT,
        '(' => prec::CALL,
        '.' => prec::SELECTION,
        '[' => prec::SUBCRIPT,
        _ => 0,
    }
}

#[derive(Debug)]
pub enum Expr {
    Unary {
        op: char,
        expr: Box<Expr>,
    },
    Binary {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: char,
    },
    Bool(bool),
    Num(u32),
    Name(char),
    Call(char, Vec<Expr>),
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
            Expr::Call(ident, args) => {
                let s = args
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{ident}({s})")
            }
        }
    }
}

// pub fn expr(lexer: &mut Lexer) -> Expr {
//     expr_bp(lexer, 0)
// }

// fn expr_bp(lexer: &mut Lexer, precedence: u8) -> Expr {
//     let mut left = match lexer.next() {
//         Token::Op('(') => {
//             let left = expr(lexer);
//             // Need to consume the closing bracket
//             assert!(matches!(lexer.next(), Token::Op(')')));
//             left
//         }
//         Token::Op(op) => Expr::Unary {
//             op,
//             expr: Box::new(expr_bp(lexer, prec::PREFIX)),
//         },
//         Token::Eof => panic!("unexpected eof"),
//         Token::Number(n) => Expr::Num(n),
//         Token::Ident(i) => Expr::Name(i),
//         Token::Bool(b) => Expr::Bool(b),
//     };

//     loop {
//         // postfix operators are just operators in the "LED" context that consume no right expression

//         // This could be EOF, which is fine.
//         // It could also be any other token which would be
//         // a syntax error, but I don't mind that just now
//         let Token::Op(op) = lexer.peek() else {
//             return left;
//         };

//         let token_prec = get_precedence(op);

//         // If the current token precedence is higher than the current precedence, then we bind to the right,
//         // otherwise we bind to the left
//         if precedence >= token_prec {
//             break;
//         }

//         lexer.next();

//         // Postfix parsing
//         match op {
//             '(' => {
//                 parse_function(lexer, &mut left);
//                 continue;
//             }
//             '[' => {
//                 left = Expr::Array {
//                     lhs: Box::new(left),
//                     index: Box::new(expr(lexer)),
//                 };
//                 let Token::Op(']') = lexer.next() else {
//                     panic!("invalid token");
//                 };
//                 continue;
//             }
//             _ => {}
//         }

//         let right = expr_bp(lexer, token_prec);
//         left = Expr::Binary {
//             lhs: Box::new(left),
//             op,
//             rhs: Box::new(right),
//         };

//         continue;
//     }

//     left
// }

// fn parse_function(lexer: &mut Lexer, left: &mut Expr) {
//     let mut args = vec![];

//     loop {
//         match lexer.peek() {
//             Token::Op(',') => {
//                 lexer.next();
//                 continue;
//             }
//             Token::Op(')') => {
//                 lexer.next();
//                 break;
//             }
//             t => (),
//         }
//         args.push(expr(lexer));
//     }

//     let Expr::Name(n) = left else {
//         panic!("invalid function name: {left:?}")
//     };

//     *left = Expr::Call(*n, args);
// }

// // #[derive(Debug, Copy, Clone, PartialEq)]
// // enum Token {
// //     Ident(char),
// //     Number(u32),
// //     Bool(bool),
// //     Op(char),
// //     Eof,
// // }

// // impl Token {
// //     fn new(c: char) -> Self {
// //         match c {
// //             'a'..='z' => Token::Ident(c),
// //             '0'..='9' => Token::Number(c.to_digit(10).unwrap()),
// //             'T' => Token::Bool(true),
// //             'F' => Token::Bool(false),
// //             _ => Token::Op(c),
// //         }
// //     }
// // }

// // // pub struct Lexer(Vec<Token>);

// // // impl Lexer {
// // //     pub fn new(src: &str) -> Self {
// // //         let tokens = src
// // //             .chars()
// // //             .rev()
// // //             .filter(|c| !c.is_whitespace())
// // //             .map(Token::new)
// // //             .collect();

// // //         Self(tokens)
// // //     }

// // //     fn next(&mut self) -> Token {
// // //         self.0.pop().unwrap_or(Token::Eof)
// // //     }

// // //     fn peek(&mut self) -> Token {
// // //         self.0.last().copied().unwrap_or(Token::Eof)
// // //     }
// // // }

// // // #[cfg(test)]
// // // mod test {
// // //     use super::*;

// // //     fn parse(input: &str) -> String {
// // //         let mut lexer = Lexer::new(input);
// // //         expr(&mut lexer).to_string()
// // //     }

// // //     #[test]
// // //     fn add_sub() {
// // //         let input = "1 + 2";
// // //         assert_eq!(parse(input), "(+ 1 2)");

// // //         let input = "1 - 2";
// // //         assert_eq!(parse(input), "(- 1 2)");
// // //     }

// // //     #[test]
// // //     fn mul_div() {
// // //         let input = "5 + 1 * 2";
// // //         assert_eq!(parse(input), "(+ 5 (* 1 2))");

// // //         let input = "5 - 1 / 2";
// // //         assert_eq!(parse(input), "(- 5 (/ 1 2))");
// // //     }

// // //     #[test]
// // //     fn brackets() {
// // //         let input = "(5 + 1) * 2";
// // //         assert_eq!(parse(input), "(* (+ 5 1) 2)");
// // //     }

// // //     #[test]
// // //     fn function() {
// // //         let input = "f(1, a + 2 * 3, 3)";
// // //         assert_eq!(parse(input), "f(1, (+ a (* 2 3)), 3)");
// // //     }

// // //     #[test]
// // //     fn function_no_args() {
// // //         let input = "f()";
// // //         assert_eq!(parse(input), "f()");
// // //     }
// // // }
