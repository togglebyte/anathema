use anathema_values::{Path, ScopeValue};

use crate::error::{src_line_no, Error, ErrorKind, Result};
use crate::lexer::Lexer;
use crate::token::{Kind, Token, Tokens, Value};
use crate::{Constants, StringId, ValueId};
use super::pratt::{eval, expr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expression {
    LoadText(ValueId),
    LoadAttribute { key: StringId, value: ValueId },
    View(ValueId),
    Node(StringId),
    For { data: ValueId, binding: StringId },
    If(ValueId),
    Else(Option<ValueId>),
    ScopeStart,
    ScopeEnd,
    EOF,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State {
    EnterScope,
    ExitScope,
    ParseFor,
    ParseIf,
    ParseView,
    ParseIdent,
    ParseAttributes,
    ParseAttribute,
    ParseText,
    Done,
}

// -----------------------------------------------------------------------------
//     - Parser -
// -----------------------------------------------------------------------------
pub struct Parser<'src, 'consts> {
    tokens: Tokens,
    consts: &'consts mut Constants,
    src: &'src str,
    state: State,
    open_scopes: Vec<usize>,
    closed_scopes: Vec<usize>,
    base_indent: usize,
    done: bool,
}

impl<'src, 'consts> Parser<'src, 'consts> {
    pub(crate) fn new(mut tokens: Tokens, consts: &'consts mut Constants, src: &'src str) -> Self {
        tokens.consume_newlines();
        let base_indent = match tokens.peek().0 {
            Kind::Indent(indent) => indent,
            _ => 0,
        };

        let inst = Self {
            tokens,
            consts,
            src,
            state: State::EnterScope,
            open_scopes: Vec::new(),
            closed_scopes: Vec::new(),
            base_indent,
            done: false,
        };

        inst
    }

    fn error(&self, kind: ErrorKind) -> Error {
        let (line, col) = src_line_no(self.tokens.previous().1, self.src);
        Error {
            line,
            col,
            src: self.src.to_string(),
            kind,
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Expression> {
        // * It is okay to advance the state once and only once
        //   in any parse function using `self.next_state()`.
        //   The exception to this is for-loops and if statements
        // * State should never be set directly in any of the parse functions.
        //   There is one exception of this, and that's when moving from
        //   `ParseAttributes` to `ParseText`.
        loop {
            let output = match self.state {
                State::EnterScope => self.enter_scope(),
                State::ParseFor => self.parse_for(),
                State::ParseIf => self.parse_if(),
                State::ParseView => self.parse_view(),
                State::ExitScope => self.exit_scope(),
                State::ParseIdent => self.parse_ident(),
                State::ParseAttributes => {
                    if !self.parse_attributes()? {
                        self.state = State::ParseText
                    }
                    Ok(None)
                }
                State::ParseAttribute => self.parse_attribute(),
                State::ParseText => self.parse_text(),
                State::Done => self.parse_done(),
            };

            match output? {
                Some(inst) => break Ok(inst),
                None => continue,
            }
        }
    }

    fn next_state(&mut self) {
        match self.state {
            State::EnterScope => self.state = State::ExitScope,
            State::ExitScope => self.state = State::ParseFor,
            State::ParseFor => self.state = State::ParseIf,
            State::ParseIf => self.state = State::ParseView,
            State::ParseView => self.state = State::ParseIdent,
            State::ParseIdent => self.state = State::ParseAttributes,
            State::ParseAttributes => self.state = State::ParseAttribute,
            State::ParseAttribute => self.state = State::ParseText,
            State::ParseText => self.state = State::Done,
            State::Done => self.state = State::EnterScope,
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 1: Parse enter / exit scopes -
    // -----------------------------------------------------------------------------
    fn enter_scope(&mut self) -> Result<Option<Expression>> {
        let indent = self.tokens.read_indent();

        if Kind::Eof == self.tokens.peek().0 {
            self.next_state();
            return Ok(None);
        }

        let indent = match indent {
            Some(indent) if indent < self.base_indent => {
                return Err(self.error(ErrorKind::InvalidDedent))
            }
            Some(indent) => Some(indent - self.base_indent),
            None => None,
        };

        let ret = match indent {
            // No indent but open scopes
            None if !self.open_scopes.is_empty() => {
                self.closed_scopes.extend(self.open_scopes.drain(..));
                Ok(None)
            }
            // No indent, no open scopes
            None => Ok(None),
            // Indent
            Some(indent) => match self.open_scopes.last() {
                // Indent is bigger than previous: create another scope
                Some(&last) if indent > last => {
                    self.open_scopes.push(indent);
                    Ok(Some(Expression::ScopeStart))
                }
                // Indent is smaller than previous: close larger scopes
                Some(&last) if indent < last => {
                    if indent > 0 && !self.open_scopes.iter().any(|s| indent.eq(s)) {
                        return Err(self.error(ErrorKind::InvalidDedent));
                    }

                    self.open_scopes.retain(|&s| {
                        if indent < s {
                            self.closed_scopes.push(s);
                            false
                        } else {
                            true
                        }
                    });

                    Ok(None)
                }
                // There are no previous indents, and this indent is not zero
                None if indent > 0 && self.open_scopes.is_empty() => {
                    self.open_scopes.push(indent);
                    Ok(Some(Expression::ScopeStart))
                }
                _ => Ok(None),
            },
        };

        self.next_state();
        ret
    }

    fn exit_scope(&mut self) -> Result<Option<Expression>> {
        match self.closed_scopes.pop() {
            Some(_) => Ok(Some(Expression::ScopeEnd)),
            None => {
                self.next_state();
                Ok(None)
            }
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 2: Parse ident, For and If -
    // -----------------------------------------------------------------------------
    fn parse_ident(&mut self) -> Result<Option<Expression>> {
        if Kind::Eof == self.tokens.peek().0 {
            self.state = State::Done;
            return Ok(None);
        }

        // Since the previous parse state was `ParseFor`, the tokens
        // might've been consumed.
        // If the next token is a newline char then move to the next state
        if Kind::Newline == self.tokens.peek().0 {
            self.next_state();
            return Ok(None);
        }

        let string_id = match self.tokens.next().0 {
            Kind::Value(Value::Ident(ident)) => ident,
            _ => {
                return Err(self.error(ErrorKind::InvalidToken {
                    expected: "identifier",
                }))
            }
        };

        self.tokens.consume_indent();
        self.next_state();
        Ok(Some(Expression::Node(string_id)))
    }

    fn parse_for(&mut self) -> Result<Option<Expression>> {
        if Kind::For != self.tokens.peek_skip_indent().0 {
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume();

        let binding = match self.tokens.next_no_indent().0 {
            Kind::Value(Value::Ident(ident)) => ident,
            _ => {
                return Err(self.error(ErrorKind::InvalidToken {
                    expected: "identifier",
                }))
            }
        };

        // self.lexer.consume(true, false);

        if Kind::In != self.tokens.peek_skip_indent().0 {
            return Err(self.error(ErrorKind::InvalidToken { expected: "in" }));
        }

        // Consume `In`
        self.tokens.consume();

        let expr = expr(&mut self.tokens);
        let value_expr = eval(expr, self.consts);

        // let data = ValueParser::new(&mut self.lexer).parse()?;
        let data = self.consts.store_value(value_expr);

        self.next_state();
        Ok(Some(Expression::For { data, binding }))
    }

    fn parse_if(&mut self) -> Result<Option<Expression>> {
        self.tokens.consume_indent();
        panic!()

        // if self.lexer.consume_if(Kind::Else)? {
        //     self.lexer.consume(true, false);

        //     let cond = match self.parse_if()? {
        //         Some(Expression::If(cond)) => Some(cond),
        //         _ => None,
        //     };

        //     Ok(Some(Expression::Else(cond)))
        // } else if self.lexer.consume_if(Kind::If)? {
        //     panic!()
        //     // self.lexer.consume(true, false);

        //     // let cond = CondParser::new(&mut self.lexer).parse()?;
        //     // let cond_id = self.constants.store_cond(cond);
        //     // self.lexer.consume(true, false);

        //     // self.next_state();
        //     // Ok(Some(Expression::If(cond_id)))
        // } else {
        //     self.next_state();
        //     Ok(None)
        // }
    }

    fn parse_view(&mut self) -> Result<Option<Expression>> {
        panic!()
        // self.lexer.consume(true, false);
        // if self.lexer.consume_if(Kind::View)? {
        //     self.lexer.consume(true, false);
        //     let id = ValueParser::new(&mut self.lexer).parse()?;
        //     let id = self.constants.store_value(id);
        //     self.lexer.consume(true, false);
        //     self.next_state();
        //     Ok(Some(Expression::View(id)))
        // } else {
        //     self.next_state();
        //     Ok(None)
        // }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 3: Parse attributes -
    // -----------------------------------------------------------------------------
    fn parse_attributes(&mut self) -> Result<bool> {
        self.tokens.consume_indent();
        panic!()

        // if self.lexer.consume_if(Kind::LBracket)? {
        //     self.next_state();
        //     Ok(true)
        // } else {
        //     Ok(false)
        // }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 4: Parse single attribute -
    // -----------------------------------------------------------------------------
    fn parse_attribute(&mut self) -> Result<Option<Expression>> {
        panic!()
        // self.lexer.consume(true, true);

        // // Check for the closing bracket
        // if self.lexer.consume_if(Kind::RBracket)? {
        //     self.next_state();
        //     return Ok(None);
        // }

        // let key = self.lexer.read_ident()?;
        // self.lexer.consume(true, true);

        // if !self.lexer.consume_if(Kind::Colon)? {
        //     return Err(self.lexer.error(ErrorKind::InvalidToken { expected: ":" }));
        // }
        // self.lexer.consume(true, true);

        // let value = ValueParser::new(&mut self.lexer).parse()?;
        // self.lexer.consume(true, true);

        // // Consume comma
        // if self.lexer.consume_if(Kind::Comma)? {
        //     self.lexer.consume(true, true);
        // } else if self.lexer.consume_if(Kind::RBracket)? {
        //     self.next_state();
        // } else {
        //     return Err(self.lexer.error(ErrorKind::UnterminatedAttributes));
        // }

        // Ok(Some(Expression::LoadAttribute { key, value }))
    }

    // -----------------------------------------------------------------------------
    //     - Stage 5: Parse text -
    // -----------------------------------------------------------------------------
    fn parse_text(&mut self) -> Result<Option<Expression>> {
        self.tokens.consume_indent();
        panic!()

        // // Only valid tokens here are:
        // // * [
        // // * \n
        // // * Text
        // // * EOF

        // if let Ok(Token(kind, _)) = self.lexer.peek() {
        //     match kind {
        //         Kind::Newline | Kind::Value(Value::String(_)) | Kind::LBracket | Kind::Eof => {}
        //         _ => {
        //             return Err(self.lexer.error(ErrorKind::InvalidToken {
        //                 expected: "either a new line, `[` or text",
        //             }))
        //         }
        //     }
        // }

        // let ret = match self.lexer.peek() {
        //     Ok(Token(Kind::Value(Value::String(s)), _)) => {
        //         panic!()
        //         // let text = parse_scope_value(s, self.lexer.consts);
        //         // let index = self.constants.store_value(text);
        //         // let _ = self.lexer.next();
        //         // Ok(Some(Expression::LoadText(index)))
        //     }
        //     _ => Ok(None),
        // };

        // self.next_state();
        // ret
    }

    // -----------------------------------------------------------------------------
    //     - Stage 6: Done -
    //     Clear empty spaces, ready for next instructions,
    //     or deal with EOF
    // -----------------------------------------------------------------------------
    fn parse_done(&mut self) -> Result<Option<Expression>> {
        panic!()
        // self.lexer.consume(true, false);
        // let token = self.lexer.next().map(|t| t.0)?;

        // let ret = match token {
        //     Kind::Eof if !self.open_scopes.is_empty() => {
        //         self.open_scopes.pop();
        //         return Ok(Some(Expression::ScopeEnd));
        //     }
        //     Kind::Eof => return Ok(Some(Expression::EOF)),
        //     Kind::Newline => {
        //         self.lexer.consume(false, true);
        //         Ok(None)
        //     }
        //     _ => Err(self.lexer.error(ErrorKind::InvalidToken {
        //         expected: "new line",
        //     })),
        // };

        // self.next_state();
        // ret
    }
}

// -----------------------------------------------------------------------------
//     - Iterator -
// -----------------------------------------------------------------------------
impl Iterator for Parser<'_, '_> {
    type Item = Result<Expression>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.parse() {
            Ok(Expression::EOF) => {
                self.done = true;
                Some(Ok(Expression::EOF))
            }
            Err(e) => {
                self.state = State::Done;
                Some(Err(e))
            }
            inst => Some(inst),
        }
    }
}

// -----------------------------------------------------------------------------
//     - Parse `ExpressionValue` -
// -----------------------------------------------------------------------------
pub(super) fn parse_scope_value(text: &str, consts: &mut Constants) -> ScopeValue {
    panic!()
    // let mut fragments = vec![];
    // let mut chars = text.char_indices().peekable();
    // let mut pos = 0;

    // while let Some(c) = chars.next() {
    //     let next = chars.peek();
    //     match (c, next) {
    //         ((i, '{'), Some((_, '{'))) => {
    //             let frag = &text[pos..i];
    //             if !frag.is_empty() {
    //                 let text_fragment = frag.replace("\\\"", "\"");
    //                 fragments.push(ScopeValue::Static(text_fragment.into()));
    //             }
    //             pos = i;
    //         }
    //         ((i, '}'), Some((_, '}'))) => {
    //             let frag = &text[pos + 2..i].trim();
    //             if !frag.is_empty() {
    //                 let mut lexer = Lexer::new(frag, consts);
    //                 if let Ok(Token(Kind::Value(Value::Ident(ident)), _)) = lexer.next() {
    //                     if let Ok(path) = parse_path(&mut lexer, ident) {
    //                         fragments.push(ScopeValue::Dyn(path));
    //                     }
    //                 }
    //             }
    //             pos = i + 2;
    //         }
    //         _ => {}
    //     }
    // }

    // let remainder = &text[pos..];

    // if !remainder.is_empty() || fragments.is_empty() {
    //     let text_fragment = remainder.replace("\\\"", "\"");
    //     fragments.push(ScopeValue::Static(text_fragment.into()));
    // }

    // if fragments.len() > 1 {
    //     ScopeValue::List(fragments.into())
    // } else {
    //     fragments.remove(0)
    // }
}

// -----------------------------------------------------------------------------
//     - Parse path -
//  Note: this is not part of the `Parser` as this is used in other
//  places to parse paths
// -----------------------------------------------------------------------------
pub(super) fn parse_path(lexer: &mut Lexer<'_, '_>, ident: &str) -> Result<Path> {
    panic!()
    // let mut path = Path::Key(ident.to_owned());

    // loop {
    //     match lexer.peek() {
    //         Ok(Token(Kind::Fullstop, _)) => drop(lexer.next()?),
    //         Ok(Token(Kind::Value(Value::Index(_)), _)) => {}
    //         _ => break,
    //     }

    //     match lexer.next() {
    //         Ok(Token(Kind::Value(Value::Ident(ident)), _)) => path = path.compose(Path::Key(ident.to_owned())),
    //         Ok(Token(Kind::Value(Value::Index(index)), _)) => path = path.compose(Path::Index(index)),
    //         _ => return Err(lexer.error(ErrorKind::InvalidPath)),
    //     }
    // }

    // Ok(path)
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(src: &str) -> Vec<Result<Expression>> {
        let mut consts = Constants::new();
        let lexer = Lexer::new(src, &mut consts);
        let tokens = Tokens::new(lexer.collect::<Result<Vec<_>>>().unwrap(), src.len());
        let parser = Parser::new(tokens, &mut consts, src);
        parser.collect()
    }

    fn parse_ok(src: &str) -> Vec<Expression> {
        parse(src).into_iter().map(Result::unwrap).collect()
    }

    fn parse_err(src: &str) -> Vec<Error> {
        parse(src).into_iter().filter_map(Result::err).collect()
    }

    // #[test]
    // fn parse_single_instruction() {
    //     let src = "a";
    //     let expected = Expression::Node(0.into());
    //     let actual = parse_ok(src).remove(0);
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_attributes() {
    //     let src = "a [a: a]";
    //     let expected = vec![
    //         Expression::Node(0.into()),
    //         Expression::LoadAttribute {
    //             key: 0.into(),
    //             value: 0.into(),
    //         },
    //         Expression::EOF,
    //     ];

    //     let actual = parse_ok(src);
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_text() {
    //     let src = "a 'a'      \n\n//some comments \n    ";
    //     let expected = vec![
    //         Expression::Node(0.into()),
    //         Expression::LoadText(0.into()),
    //         Expression::EOF,
    //     ];

    //     let actual = parse_ok(src);
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_scopes() {
    //     let src = "
    //         a
    //             b
    //                 c
    //             b
    //         a
    //         ";
    //     let expected = vec![
    //         Expression::Node(0.into()),
    //         Expression::ScopeStart,
    //         Expression::Node(1.into()),
    //         Expression::ScopeStart,
    //         Expression::Node(2.into()),
    //         Expression::ScopeEnd,
    //         Expression::Node(1.into()),
    //         Expression::ScopeEnd,
    //         Expression::Node(0.into()),
    //         Expression::EOF,
    //     ];

    //     let actual = parse_ok(src);
    //     assert_eq!(expected, actual);

    //     let src = "
    //         a
    //             b
    //                 c
    //         ";
    //     let expected = vec![
    //         Expression::Node(0.into()),
    //         Expression::ScopeStart,
    //         Expression::Node(1.into()),
    //         Expression::ScopeStart,
    //         Expression::Node(2.into()),
    //         Expression::ScopeEnd,
    //         Expression::ScopeEnd,
    //         Expression::EOF,
    //     ];

    //     let actual = parse_ok(src);
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_nested_for_loops() {
    //     let src = "
    //     x
    //         for x in {{ data }}
    //             for y in {{ data }}
    //                 x
    //     ";
    //     let mut instructions = parse_ok(src);

    //     assert_eq!(instructions.remove(0), Expression::Node(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(
    //         instructions.remove(0),
    //         Expression::For {
    //             data: 0.into(),
    //             binding: 0.into()
    //         }
    //     );
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(
    //         instructions.remove(0),
    //         Expression::For {
    //             data: 0.into(),
    //             binding: 1.into()
    //         }
    //     );
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(instructions.remove(0), Expression::Node(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    // }

    // #[test]
    // fn parse_scopes_and_for() {
    //     let src = "
    //     x
    //         y
    //     for x in {{ data }}
    //         y
    //     ";
    //     let mut instructions = parse_ok(src);
    //     assert_eq!(instructions.remove(0), Expression::Node(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(instructions.remove(0), Expression::Node(1.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    //     assert_eq!(
    //         instructions.remove(0),
    //         Expression::For {
    //             data: 0.into(),
    //             binding: 0.into()
    //         }
    //     );
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(instructions.remove(0), Expression::Node(1.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    // }

    // #[test]
    // fn parse_if() {
    //     let src = "
    //     if {{ data }}
    //         x
    //     ";
    //     let mut instructions = parse_ok(src);

    //     assert_eq!(instructions.remove(0), Expression::If(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(instructions.remove(0), Expression::Node(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    // }

    // #[test]
    // fn parse_else() {
    //     let src = "
    //     if {{ data }}
    //         x
    //     else
    //         y
    //     ";
    //     let mut instructions = parse_ok(src);

    //     assert_eq!(instructions.remove(0), Expression::If(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(instructions.remove(0), Expression::Node(0.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    //     assert_eq!(instructions.remove(0), Expression::Else(None));
    //     assert_eq!(instructions.remove(0), Expression::ScopeStart);
    //     assert_eq!(instructions.remove(0), Expression::Node(1.into()));
    //     assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    // }

    // #[test]
    // fn parse_if_else_if_else() {
    //     let src = "
    //     if {{ data }}
    //         x
    //     else if {{ data }}
    //         y
    //     else
    //         z
    //     ";
    //     let mut expressions = parse_ok(src);

    //     assert_eq!(expressions.remove(0), Expression::If(0.into()));
    //     assert_eq!(expressions.remove(0), Expression::ScopeStart);
    //     assert_eq!(expressions.remove(0), Expression::Node(0.into()));
    //     assert_eq!(expressions.remove(0), Expression::ScopeEnd);
    //     assert_eq!(expressions.remove(0), Expression::Else(Some(0.into())));
    //     assert_eq!(expressions.remove(0), Expression::ScopeStart);
    //     assert_eq!(expressions.remove(0), Expression::Node(1.into()));
    //     assert_eq!(expressions.remove(0), Expression::ScopeEnd);
    //     assert_eq!(expressions.remove(0), Expression::Else(None));
    //     assert_eq!(expressions.remove(0), Expression::ScopeStart);
    //     assert_eq!(expressions.remove(0), Expression::Node(2.into()));
    //     assert_eq!(expressions.remove(0), Expression::ScopeEnd);
    // }

    // #[test]
    // fn parse_view() {
    //     let src = "view 'mail'";
    //     let mut expressions = parse_ok(src);
    //     assert_eq!(expressions.remove(0), Expression::View(0.into()));
    // }

    // #[test]
    // fn parse_empty_if() {
    //     let src = "
    //         if {{ x }}
    //         x
    //     ";

    //     let mut expressions = parse_ok(src);
    //     assert_eq!(expressions.remove(0), Expression::If(0.into()));
    //     assert_eq!(expressions.remove(0), Expression::Node(0.into()));
    // }

    // #[test]
    // fn parse_no_instruction() {
    //     let src = "";
    //     let expected: Vec<Expression> = vec![Expression::EOF];
    //     let actual = parse_ok(src);
    //     assert_eq!(expected, actual);

    //     let src = "\n// comment         \n";
    //     let expected: Vec<Expression> = vec![Expression::EOF];
    //     let actual = parse_ok(src);
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_invalid_token_after_text() {
    //     let src = "a 'a' 'b'";
    //     let expected = Error {
    //         kind: ErrorKind::InvalidToken {
    //             expected: "new line",
    //         },
    //         line: 1,
    //         col: 7,
    //         src: src.to_string(),
    //     };
    //     let actual = parse_err(src).remove(0);
    //     assert_eq!(expected, actual);
    // }

    // #[test]
    // fn parse_invalid_path() {
    //     let src = "node [path: {{ a.-b.c }}]";
    //     let expected = Error {
    //         kind: ErrorKind::InvalidPath,
    //         line: 1,
    //         col: 18,
    //         src: src.to_string(),
    //     };
    //     let actual = parse_err(src).remove(0);
    //     assert_eq!(expected, actual);
    // }
}
