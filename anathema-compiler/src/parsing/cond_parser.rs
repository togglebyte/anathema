use super::parser::Cond;
use crate::error::{ErrorKind, Result};
use crate::lexer::Lexer;
use crate::token::{Kind, Token};
use crate::parsing::value_parser::ValueParser;
use crate::{Constants, ValueId};

#[derive(Debug)]
enum Op {
    And,
    Or,
    LParen,
    RParen,
}

#[derive(Debug)]
enum X {
    Value(ValueId),
    Op(Op),
}

pub(super) struct CondParser<'lexer, 'src, 'consts> {
    lexer: &'lexer mut Lexer<'src, 'consts>,
    output: Vec<X>,
    operators: Vec<Op>,
}

impl<'lexer, 'src, 'consts> CondParser<'lexer, 'src, 'consts> {
    pub(super) fn new(lexer: &'lexer mut Lexer<'src, 'consts>) -> Self {
        Self {
            lexer,
            output: vec![],
            operators: vec![],
        }
    }

    pub(super) fn parse(&mut self) -> Result<Cond> {
        panic!()
        //// ( -> push onto op stack
        //// ) -> pop all the ops from the operator stack until reaching a `(`, place each on the output
        //// && -> push `And` onto op stack
        //// || -> push `Or` onto op stack
        //// Value ->
        ////
        ////
        //// (xx && xx && xx) || (yy || zz) -> Cond::Or(group, group)

        //loop {
        //    // Start a group
        //    if self.lexer.consume_if(Kind::LParen)? {
        //        self.operators.push(Op::LParen);
        //    }

        //    // End a group
        //    if self.lexer.consume_if(Kind::RParen)? {
        //        // TODO: unwrap... ewwww
        //        let op = self.operators.pop().unwrap();
        //        match op {
        //            Op::LParen => continue,
        //            _ => self.output.push(X::Op(op)),
        //        }
        //    }

        //    // And
        //    if self.lexer.consume_if(Kind::And)? {
        //        self.operators.push(Op::And);
        //        self.lexer.consume(true, true);
        //    }

        //    // Or
        //    if self.lexer.consume_if(Kind::Or)? {
        //        self.operators.push(Op::Or);
        //        self.lexer.consume(true, true);
        //    }

        //    // Value
        //    let value = ValueParser::new(self.lexer, self.constants).parse()?;
        //    let value_id = self.constants.store_value(value);
        //    self.output.push(X::Value(value_id));
        //    self.lexer.consume(true, true);

        //    // End parsing on newline or EOF
        //    if let Ok(Token(kind, _)) = self.lexer.peek() {
        //        match kind {
        //            Kind::Newline | Kind::EOF => break,
        //            _ => continue,
        //        }
        //    }
        //}

        //while let Some(op) = self.operators.pop() {
        //    self.output.push(X::Op(op));
        //}

        //loop {
        //    let lhs = self.output.pop().unwrap();
        //    let rhs = self.output.pop().unwrap();
        //    let op = self.output.pop().unwrap();
        //}

        //panic!("{:#?}", self.output)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(src: &str) -> Cond {
        let mut consts = Constants::new();
        let mut lexer = Lexer::new(src, &mut consts);
        let mut cond_parser = CondParser::new(&mut lexer);
        cond_parser.parse().unwrap()
    }

//     #[test]
//     fn parse_and() {
//         let cond = parse("a && b");
//     }
}
