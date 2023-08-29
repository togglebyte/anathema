use std::sync::Arc;

use anathema_values::{Path, ScopeValue};
// use anathema_widget_core::{Number, Value};

use super::attribute_parser::AttributeParser;
use crate::error::{src_line_no, Error, ErrorKind, Result};
use crate::lexer::{Kind, Lexer, Token};
use crate::{Constants, StringId, ValueId};

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

// -----------------------------------------------------------------------------
//     - Lexer extensions -
// -----------------------------------------------------------------------------
impl<'src> Lexer<'src> {
    // -----------------------------------------------------------------------------
    //     - Errors -
    // -----------------------------------------------------------------------------
    pub(super) fn error(&self, kind: ErrorKind) -> Error {
        let (line, col) = src_line_no(self.current_pos, self.src);
        Error {
            line,
            col,
            src: self.src.to_string(),
            kind,
        }
    }

    // -----------------------------------------------------------------------------
    //     - Token checks -
    // -----------------------------------------------------------------------------
    fn is_whitespace(&mut self) -> bool {
        matches!(self.peek(), Ok(Token(Kind::Indent(_), _)))
    }

    fn is_newline(&mut self) -> bool {
        matches!(self.peek(), Ok(Token(Kind::Newline, _)))
    }

    fn is_comment(&mut self) -> bool {
        matches!(self.peek(), Ok(Token(Kind::Comment, _)))
    }

    fn is_next_token(&mut self, kind: Kind) -> Result<bool> {
        match self.peek() {
            Ok(Token(other, _)) => Ok(kind.eq(other)),
            Err(e) => Err(e.clone()),
        }
    }

    pub(super) fn consume_if(&mut self, kind: Kind) -> Result<bool> {
        if self.is_next_token(kind)? {
            let _ = self.next();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // -----------------------------------------------------------------------------
    //     - Consuming / peeking -
    // -----------------------------------------------------------------------------
    pub(super) fn consume(&mut self, whitespace: bool, newlines: bool) {
        loop {
            if whitespace && self.is_whitespace() {
                let _ = self.next();
            } else if newlines && self.is_newline() {
                let _ = self.next();
            } else if self.is_comment() {
                let _ = self.next();
            } else {
                break;
            }
        }
    }

    pub(super) fn read_ident(&mut self) -> Result<&'src str> {
        match self.next() {
            Ok(Token(Kind::Ident(ident), _)) => Ok(ident),
            Ok(_) => Err(self.error(ErrorKind::InvalidToken {
                expected: "identifier",
            })),
            Err(e) => Err(e),
        }
    }

    fn read_indent(&mut self) -> Option<Result<usize>> {
        match self.peek() {
            Ok(Token(Kind::Indent(indent), _)) => {
                let indent = *indent;
                let _ = self.next();
                Some(Ok(indent))
            }
            Ok(_) => None,
            Err(e) => {
                let ret = Some(Err(e.clone()));
                let _ = self.next();
                ret
            }
        }
    }
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
    lexer: Lexer<'src>,
    state: State,
    constants: &'consts mut Constants,
    open_scopes: Vec<usize>,
    closed_scopes: Vec<usize>,
    base_indent: usize,
    done: bool,
}

impl<'src, 'consts> Parser<'src, 'consts> {
    pub(crate) fn new(mut lexer: Lexer<'src>, ctx: &'consts mut Constants) -> Result<Self> {
        lexer.consume(false, true);
        let base_indent = match lexer.peek() {
            Ok(Token(Kind::Indent(indent), _)) => *indent,
            _ => 0,
        };

        let inst = Self {
            lexer,
            state: State::EnterScope,
            constants: ctx,
            open_scopes: Vec::new(),
            closed_scopes: Vec::new(),
            base_indent,
            done: false,
        };

        Ok(inst)
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
        let indent = self.lexer.read_indent().transpose()?;

        if self.lexer.is_next_token(Kind::EOF)? {
            self.next_state();
            return Ok(None);
        }

        let indent = match indent {
            Some(indent) if indent < self.base_indent => {
                return Err(self.lexer.error(ErrorKind::InvalidUnindent))
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
                        return Err(self.lexer.error(ErrorKind::InvalidUnindent));
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
        if self.lexer.is_next_token(Kind::EOF)? {
            self.state = State::Done;
            return Ok(None);
        }

        // Since the previous parse state was `ParseFor`, the tokens
        // might've been consumed.
        // If the next token is a newline char then move to the next state
        if self.lexer.is_next_token(Kind::Newline)? {
            self.next_state();
            return Ok(None);
        }

        let ident = self.lexer.read_ident()?;

        let index = self.constants.store_string(ident);
        self.lexer.consume(true, false);
        self.next_state();
        Ok(Some(Expression::Node(index)))
    }

    fn parse_for(&mut self) -> Result<Option<Expression>> {
        self.lexer.consume(true, false);
        if self.lexer.consume_if(Kind::For)? {
            self.lexer.consume(true, false);
            let binding = self.lexer.read_ident()?;
            self.lexer.consume(true, false);

            if !matches!(self.lexer.peek(), Ok(Token(Kind::In, _))) {
                return Err(self.lexer.error(ErrorKind::InvalidToken { expected: "in" }));
            }
            // Consume `In`
            let _ = self.lexer.next();
            self.lexer.consume(true, false);

            let binding = self.constants.store_string(binding);

            let data = AttributeParser::new(&mut self.lexer, &mut self.constants).parse("")?;
            let data = self.constants.store_value(data);
            self.lexer.consume(true, false);

            self.next_state();
            Ok(Some(Expression::For { data, binding }))
        } else {
            self.next_state();
            Ok(None)
        }
    }

    fn parse_if(&mut self) -> Result<Option<Expression>> {
        self.lexer.consume(true, false);
        if self.lexer.consume_if(Kind::Else)? {
            self.lexer.consume(true, false);

            let cond = match self.parse_if()? {
                Some(Expression::If(cond)) => Some(cond),
                _ => None,
            };

            Ok(Some(Expression::Else(cond)))
        } else if self.lexer.consume_if(Kind::If)? {
            self.lexer.consume(true, false);

            let cond = AttributeParser::new(&mut self.lexer, &mut self.constants).parse("")?;
            let cond = self.constants.store_value(cond);
            self.lexer.consume(true, false);

            self.next_state();
            Ok(Some(Expression::If(cond)))
        } else {
            self.next_state();
            Ok(None)
        }
    }

    fn parse_view(&mut self) -> Result<Option<Expression>> {
        self.lexer.consume(true, false);
        if self.lexer.consume_if(Kind::View)? {
            self.lexer.consume(true, false);
            let id = AttributeParser::new(&mut self.lexer, &mut self.constants).parse("")?;
            let id = self.constants.store_value(id);
            self.lexer.consume(true, false);
            self.next_state();
            Ok(Some(Expression::View(id)))
        } else {
            self.next_state();
            Ok(None)
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 3: Parse attributes -
    // -----------------------------------------------------------------------------
    fn parse_attributes(&mut self) -> Result<bool> {
        self.lexer.consume(true, false);

        if self.lexer.consume_if(Kind::LBracket)? {
            self.next_state();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 4: Parse single attribute -
    // -----------------------------------------------------------------------------
    fn parse_attribute(&mut self) -> Result<Option<Expression>> {
        self.lexer.consume(true, true);

        // Check for the closing bracket
        if self.lexer.consume_if(Kind::RBracket)? {
            self.next_state();
            return Ok(None);
        }

        let left = self.lexer.read_ident()?;
        self.lexer.consume(true, true);

        if !self.lexer.consume_if(Kind::Colon)? {
            return Err(self.lexer.error(ErrorKind::InvalidToken { expected: ":" }));
        }
        self.lexer.consume(true, true);

        let right = AttributeParser::new(&mut self.lexer, &mut self.constants).parse(left)?;
        self.lexer.consume(true, true);

        // Consume comma
        if self.lexer.consume_if(Kind::Comma)? {
            self.lexer.consume(true, true);
        } else if self.lexer.consume_if(Kind::RBracket)? {
            self.next_state();
        } else {
            return Err(self.lexer.error(ErrorKind::UnterminatedAttributes));
        }

        let key = self.constants.store_string(left);
        let value = self.constants.store_value(right);

        Ok(Some(Expression::LoadAttribute { key, value }))
    }

    // -----------------------------------------------------------------------------
    //     - Stage 5: Parse text -
    // -----------------------------------------------------------------------------
    fn parse_text(&mut self) -> Result<Option<Expression>> {
        self.lexer.consume(true, false);

        // Only valid tokens here are:
        // * [
        // * \n
        // * Text
        // * EOF

        if let Ok(Token(kind, _)) = self.lexer.peek() {
            match kind {
                Kind::Newline | Kind::String(_) | Kind::LBracket | Kind::EOF => {}
                _ => {
                    return Err(self.lexer.error(ErrorKind::InvalidToken {
                        expected: "either a new line, `[` or text",
                    }))
                }
            }
        }

        let ret = match self.lexer.peek() {
            Ok(Token(Kind::String(s), _)) => {
                let text = parse_scope_value(s, self.constants);
                let index = self.constants.store_value(text);
                let _ = self.lexer.next();
                Ok(Some(Expression::LoadText(index)))
            }
            _ => Ok(None),
        };

        self.next_state();
        ret
    }

    // -----------------------------------------------------------------------------
    //     - Stage 6: Done -
    //     Clear empty spaces, ready for next instructions,
    //     or deal with EOF
    // -----------------------------------------------------------------------------
    fn parse_done(&mut self) -> Result<Option<Expression>> {
        self.lexer.consume(true, false);
        let token = self.lexer.next().map(|t| t.0)?;

        let ret = match token {
            Kind::EOF if !self.open_scopes.is_empty() => {
                self.open_scopes.pop();
                return Ok(Some(Expression::ScopeEnd));
            }
            Kind::EOF => return Ok(Some(Expression::EOF)),
            Kind::Newline => {
                self.lexer.consume(false, true);
                Ok(None)
            }
            _ => Err(self.lexer.error(ErrorKind::InvalidToken {
                expected: "new line",
            })),
        };

        self.next_state();
        ret
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
pub(super) fn parse_scope_value(
    text: &str,
    consts: &mut Constants,
) -> ScopeValue {
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
    //                 fragments.push(ExpressionValue::Static(Arc::new(Value::String(
    //                     text_fragment,
    //                 ))));
    //             }
    //             pos = i;
    //         }
    //         ((i, '}'), Some((_, '}'))) => {
    //             let frag = &text[pos + 2..i].trim();
    //             if !frag.is_empty() {
    //                 let mut lexer = Lexer::new(frag);
    //                 if let Ok(Token(Kind::Ident(ident), _)) = lexer.next() {
    //                     if let Ok(path) = parse_path(&mut lexer, ident) {
    //                         let path_id = consts.store_path(path);
    //                         fragments.push(ExpressionValue::Dyn(path_id));
    //                     }
    //                 }
    //             }
    //             pos = i + 2;
    //         }
    //         _ => {}
    //     }
    // }

    // let remainder = &text[pos..];

    // if !remainder.is_empty() {
    //     let text_fragment = remainder.replace("\\\"", "\"");
    //     fragments.push(ExpressionValue::Static(Arc::new(Value::String(text_fragment))));
    // }

    // // There is at least one fragment value so it's 
    // // fine to call `remove` here.
    // if fragments.len() > 1 {
    //     ExpressionValue::List(fragments.into())
    // } else {
    //     fragments.remove(0)
    // }
}

// -----------------------------------------------------------------------------
//     - Parse path -
//  Note: this is not part of the `Parser` as this is used in other
//  places to parse paths
// -----------------------------------------------------------------------------
pub(super) fn parse_path(lexer: &mut Lexer<'_>, ident: &str) -> Result<Path> {
    let mut path = Path::Key(ident.to_owned());

    loop {
        match lexer.peek() {
            Ok(Token(Kind::Fullstop, _)) => drop(lexer.next()?),
            Ok(Token(Kind::Index(_), _)) => {}
            _ => break,
        }

        match lexer.next() {
            Ok(Token(Kind::Ident(ident), _)) => path = path.compose(Path::Key(ident.to_owned())),
            Ok(Token(Kind::Index(index), _)) => path = path.compose(Path::Index(index)),
            _ => return Err(lexer.error(ErrorKind::InvalidPath)),
        }
    }

    Ok(path)
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(src: &str) -> Vec<Result<Expression>> {
        let mut consts = Constants::default();
        let lexer = Lexer::new(src);
        let parser = Parser::new(lexer, &mut consts).unwrap();
        parser.collect()
    }

    fn parse_ok(src: &str) -> Vec<Expression> {
        parse(src).into_iter().map(Result::unwrap).collect()
    }

    fn parse_err(src: &str) -> Vec<Error> {
        parse(src).into_iter().filter_map(Result::err).collect()
    }

    #[test]
    fn parse_single_instruction() {
        let src = "a";
        let expected = Expression::Node(0);
        let actual = parse_ok(src).remove(0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_attributes() {
        let src = "a [a: a]";
        let expected = vec![
            Expression::Node(0),
            Expression::LoadAttribute { key: 0, value: 0 },
            Expression::EOF,
        ];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_text() {
        let src = "a 'a'      \n\n//some comments \n    ";
        let expected = vec![
            Expression::Node(0),
            Expression::LoadText(0),
            Expression::EOF,
        ];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_scopes() {
        let src = "
            a
                b
                    c
                b
            a
            ";
        let expected = vec![
            Expression::Node(0),
            Expression::ScopeStart,
            Expression::Node(1),
            Expression::ScopeStart,
            Expression::Node(2),
            Expression::ScopeEnd,
            Expression::Node(1),
            Expression::ScopeEnd,
            Expression::Node(0),
            Expression::EOF,
        ];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);

        let src = "
            a
                b
                    c
            ";
        let expected = vec![
            Expression::Node(0),
            Expression::ScopeStart,
            Expression::Node(1),
            Expression::ScopeStart,
            Expression::Node(2),
            Expression::ScopeEnd,
            Expression::ScopeEnd,
            Expression::EOF,
        ];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_nested_for_loops() {
        let src = "
        x
            for x in {{ data }}
                for y in {{ data }}
                    x
        ";
        let mut instructions = parse_ok(src);

        assert_eq!(instructions.remove(0), Expression::Node(0));
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(
            instructions.remove(0),
            Expression::For {
                data: 0,
                binding: 0
            }
        );
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(
            instructions.remove(0),
            Expression::For {
                data: 0,
                binding: 1
            }
        );
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(instructions.remove(0), Expression::Node(0));
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    }

    #[test]
    fn parse_scopes_and_for() {
        let src = "
        x
            y
        for x in {{ data }}
            y
        ";
        let mut instructions = parse_ok(src);
        assert_eq!(instructions.remove(0), Expression::Node(0));
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(instructions.remove(0), Expression::Node(1));
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
        assert_eq!(
            instructions.remove(0),
            Expression::For {
                data: 0,
                binding: 0
            }
        );
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(instructions.remove(0), Expression::Node(1));
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    }

    #[test]
    fn parse_if() {
        let src = "
        if {{ data }}
            x
        ";
        let mut instructions = parse_ok(src);

        assert_eq!(instructions.remove(0), Expression::If(0));
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(instructions.remove(0), Expression::Node(0));
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    }

    #[test]
    fn parse_else() {
        let src = "
        if {{ data }}
            x
        else
            y
        ";
        let mut instructions = parse_ok(src);

        assert_eq!(instructions.remove(0), Expression::If(0));
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(instructions.remove(0), Expression::Node(0));
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
        assert_eq!(instructions.remove(0), Expression::Else(None));
        assert_eq!(instructions.remove(0), Expression::ScopeStart);
        assert_eq!(instructions.remove(0), Expression::Node(1));
        assert_eq!(instructions.remove(0), Expression::ScopeEnd);
    }

    #[test]
    fn parse_if_else_if_else() {
        let src = "
        if {{ data }}
            x
        else if {{ data }}
            y
        else
            z
        ";
        let mut expressions = parse_ok(src);

        assert_eq!(expressions.remove(0), Expression::If(0));
        assert_eq!(expressions.remove(0), Expression::ScopeStart);
        assert_eq!(expressions.remove(0), Expression::Node(0));
        assert_eq!(expressions.remove(0), Expression::ScopeEnd);
        assert_eq!(expressions.remove(0), Expression::Else(Some(0)));
        assert_eq!(expressions.remove(0), Expression::ScopeStart);
        assert_eq!(expressions.remove(0), Expression::Node(1));
        assert_eq!(expressions.remove(0), Expression::ScopeEnd);
        assert_eq!(expressions.remove(0), Expression::Else(None));
        assert_eq!(expressions.remove(0), Expression::ScopeStart);
        assert_eq!(expressions.remove(0), Expression::Node(2));
        assert_eq!(expressions.remove(0), Expression::ScopeEnd);
    }

    #[test]
    fn parse_view() {
        let src = "view 'mail'";
        let mut expressions = parse_ok(src);
        assert_eq!(expressions.remove(0), Expression::View(0));
    }

    #[test]
    fn parse_empty_if() {
        let src = "
            if {{ x }}
            x
        ";

        let mut expressions = parse_ok(src);
        assert_eq!(expressions.remove(0), Expression::If(0));
        assert_eq!(expressions.remove(0), Expression::Node(0));
    }

    #[test]
    fn parse_no_instruction() {
        let src = "";
        let expected: Vec<Expression> = vec![Expression::EOF];
        let actual = parse_ok(src);
        assert_eq!(expected, actual);

        let src = "\n// comment         \n";
        let expected: Vec<Expression> = vec![Expression::EOF];
        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_invalid_token_after_text() {
        let src = "a 'a' 'b'";
        let expected = Error {
            kind: ErrorKind::InvalidToken {
                expected: "new line",
            },
            line: 1,
            col: 7,
            src: src.to_string(),
        };
        let actual = parse_err(src).remove(0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_invalid_path() {
        let src = "node [path: {{ a.-b.c }}]";
        let expected = Error {
            kind: ErrorKind::InvalidPath,
            line: 1,
            col: 17,
            src: src.to_string(),
        };
        let actual = parse_err(src).remove(0);
        assert_eq!(expected, actual);
    }
}
