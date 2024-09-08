use anathema_store::storage::strings::{StringId, Strings};

use super::Statement;
use crate::components::ComponentTemplates;
use crate::error::{src_line_no, ParseError, ParseErrorKind, Result};
use crate::expressions::parser::parse_expr;
use crate::expressions::Expression;
use crate::token::{Kind, Operator, Tokens, Value};
// use crate::variables::Visibility;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State {
    EnterScope,
    ExitScope,
    ParseFor,
    ParseIf,
    ParseDeclaration,
    ParseComponent,
    ParseAssociatedFunctions,
    ParseAssociatedFunction,
    ParseComponentSlot,
    ParseIdent,
    ParseAttributes,
    ParseAttribute,
    ParseValue,
    Done,
}

// -----------------------------------------------------------------------------
//     - Parser -
// -----------------------------------------------------------------------------
pub(crate) struct Parser<'src, 'strings, 'components> {
    tokens: Tokens,
    components: &'components mut ComponentTemplates,
    strings: &'strings mut Strings,
    src: &'src str,
    state: State,
    open_scopes: Vec<usize>,
    closed_scopes: Vec<usize>,
    base_indent: usize,
    done: bool,
}

impl<'src, 'strings, 'view> Parser<'src, 'strings, 'view> {
    pub(crate) fn new(
        mut tokens: Tokens,
        strings: &'strings mut Strings,
        src: &'src str,
        components: &'view mut ComponentTemplates,
    ) -> Self {
        tokens.consume_newlines();
        let base_indent = match tokens.peek() {
            Kind::Indent(indent) => indent,
            _ => 0,
        };

        Self {
            tokens,
            strings,
            components,
            src,
            state: State::EnterScope,
            open_scopes: Vec::new(),
            closed_scopes: Vec::new(),
            base_indent,
            done: false,
        }
    }

    fn error(&self, kind: ParseErrorKind) -> ParseError {
        let (line, col) = src_line_no(self.tokens.previous().1, self.src);
        ParseError {
            line,
            col,
            src: self.src.to_string(),
            kind,
        }
    }

    fn read_ident(&mut self) -> Result<StringId, ParseError> {
        match self.tokens.next_no_indent() {
            Kind::Value(Value::Ident(ident)) => Ok(ident),
            _ => Err(self.error(ParseErrorKind::InvalidToken { expected: "identifier" })),
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement> {
        // * It is okay to advance the state once and only once
        //   in any parse function using `self.next_state()`.
        //   The exception to this is he parse view function
        // * State should never be set directly in any of the parse functions.
        //   There is one exception of this, and that's when moving from
        //   `ParseAttributes` to `ParseText`.
        loop {
            let output = match self.state {
                State::EnterScope => self.enter_scope()?,
                State::ParseFor => self.parse_for()?,
                State::ParseIf => self.parse_if()?,
                State::ParseDeclaration => self.parse_declaration()?,
                State::ParseComponent => self.parse_component()?,
                State::ParseAssociatedFunctions => {
                    // This is used to skip state,
                    // rather than calling "next state" multiple times
                    // inside the `parse_associated_functions` function
                    if !self.parse_associated_functions()? {
                        self.state = State::ParseComponentSlot;
                    }
                    None
                }
                State::ParseAssociatedFunction => self.parse_associated_function()?,
                State::ParseComponentSlot => self.parse_component_slot()?,
                State::ExitScope => self.exit_scope()?,
                State::ParseIdent => self.parse_ident()?,
                State::ParseAttributes => {
                    if !self.parse_attributes()? {
                        self.state = State::ParseValue
                    }
                    None
                }
                State::ParseAttribute => self.parse_attribute()?,
                State::ParseValue => self.parse_value()?,
                State::Done => self.parse_done()?,
            };

            match output {
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
            State::ParseIf => self.state = State::ParseDeclaration,
            State::ParseDeclaration => self.state = State::ParseIdent,
            State::ParseIdent => self.state = State::ParseComponent,
            State::ParseComponent => self.state = State::ParseAssociatedFunctions,
            State::ParseAssociatedFunctions => self.state = State::ParseAssociatedFunction,
            State::ParseAssociatedFunction => self.state = State::ParseComponentSlot,
            State::ParseComponentSlot => self.state = State::ParseAttributes,
            State::ParseAttributes => self.state = State::ParseAttribute,
            State::ParseAttribute => self.state = State::ParseValue,
            State::ParseValue => self.state = State::Done,
            State::Done => self.state = State::EnterScope,
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 1: Parse enter / exit scopes and assignments -
    // -----------------------------------------------------------------------------
    fn enter_scope(&mut self) -> Result<Option<Statement>, ParseError> {
        let indent = self.tokens.read_indent();

        match self.tokens.peek() {
            Kind::Eof | Kind::Newline => {
                self.next_state();
                return Ok(None);
            }
            _ => {}
        }

        let indent = match indent {
            Some(indent) if indent < self.base_indent => return Err(self.error(ParseErrorKind::InvalidDedent)),
            Some(indent) => Some(indent - self.base_indent),
            None => None,
        };

        let ret = match indent {
            // No indent but open scopes
            None if !self.open_scopes.is_empty() => {
                self.closed_scopes.append(&mut self.open_scopes);
                Ok(None)
            }
            // No indent, no open scopes
            None => Ok(None),
            // Indent
            Some(indent) => match self.open_scopes.last() {
                // Indent is bigger than previous: create another scope
                Some(&last) if indent > last => {
                    self.open_scopes.push(indent);
                    Ok(Some(Statement::ScopeStart))
                }
                // Indent is smaller than previous: close larger scopes
                Some(&last) if indent < last => {
                    if indent > 0 && !self.open_scopes.iter().any(|s| indent.eq(s)) {
                        return Err(self.error(ParseErrorKind::InvalidDedent));
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
                    Ok(Some(Statement::ScopeStart))
                }
                _ => Ok(None),
            },
        };

        self.next_state();
        ret
    }

    fn exit_scope(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.closed_scopes.pop() {
            Some(_) => Ok(Some(Statement::ScopeEnd)),
            None => {
                self.next_state();
                Ok(None)
            }
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 2: Parse ident, For, If, declaration and assignment -
    // -----------------------------------------------------------------------------
    fn parse_ident(&mut self) -> Result<Option<Statement>, ParseError> {
        if Kind::Eof == self.tokens.peek() {
            self.state = State::Done;
            return Ok(None);
        }

        // Since the previous parse state was `ParseFor`, the tokens
        // might've been consumed.
        //
        // If the next token is a newline char, a component or a component slot
        // then move to the next state
        if let Kind::Newline | Kind::Component | Kind::ComponentSlot = self.tokens.peek() {
            self.next_state();
            return Ok(None);
        }

        let ident = self.read_ident()?;

        self.tokens.consume_indent();
        self.next_state();
        Ok(Some(Statement::Node(ident)))
    }

    fn parse_for(&mut self) -> Result<Option<Statement>, ParseError> {
        if Kind::For != self.tokens.peek_skip_indent() {
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume();

        let binding = self.read_ident()?;

        if Kind::In != self.tokens.peek_skip_indent() {
            return Err(self.error(ParseErrorKind::InvalidToken { expected: "in" }));
        }

        // Consume `In`
        self.tokens.consume();

        let data = match parse_expr(&mut self.tokens, self.strings) {
            Ok(data) => data,
            Err(e) => return Err(self.error(e)),
        };
        self.next_state();
        Ok(Some(Statement::For { data, binding }))
    }

    fn parse_if(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.tokens.peek_skip_indent() {
            Kind::Else => {
                self.tokens.consume();
                let cond = match self.parse_if()? {
                    Some(Statement::If(cond)) => Some(cond),
                    _ => None,
                };

                Ok(Some(Statement::Else(cond)))
            }
            Kind::If => {
                self.tokens.consume();
                let cond = parse_expr(&mut self.tokens, self.strings).map_err(|e| self.error(e))?;

                self.next_state();
                Ok(Some(Statement::If(cond)))
            }
            _ => {
                self.next_state();
                Ok(None)
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Option<Statement>, ParseError> {
        // Check if it's a declaration otherwise move on
        match self.tokens.peek_skip_indent() {
            Kind::Decl => (),
            _ => {
                self.next_state();
                return Ok(None);
            }
        };
        self.tokens.consume();

        let binding = self.read_ident()?;

        if let Kind::Equal = self.tokens.peek_skip_indent() {
            self.tokens.consume();
            let value = parse_expr(&mut self.tokens, self.strings).map_err(|e| self.error(e))?;
            self.next_state();
            let statement = Statement::Declaration { binding, value };
            return Ok(Some(statement));
        }

        self.next_state();
        Ok(None)
    }

    fn parse_component(&mut self) -> Result<Option<Statement>, ParseError> {
        if Kind::Component != self.tokens.peek_skip_indent() {
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume();
        self.tokens.consume_indent();

        let ident = self.read_ident()?;
        let ident = self.strings.get_unchecked(ident);
        let component_id = self.components.insert_id(ident.to_owned());
        self.tokens.consume_indent();

        self.next_state();
        Ok(Some(Statement::Component(component_id)))
    }

    fn parse_associated_functions(&mut self) -> Result<bool, ParseError> {
        if Kind::Op(Operator::LParen) == self.tokens.peek_skip_indent() {
            self.tokens.consume();
            self.tokens.consume_all_whitespace();
            self.next_state();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_associated_function(&mut self) -> Result<Option<Statement>, ParseError> {
        // Check for the closing paren
        if Kind::Op(Operator::RParen) == self.tokens.peek_skip_indent() {
            self.tokens.consume();
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume_all_whitespace();
        let internal = self.read_ident()?;
        self.tokens.consume_all_whitespace();

        if Kind::Op(Operator::Association) != self.tokens.peek_skip_indent() {
            return Err(self.error(ParseErrorKind::InvalidToken { expected: "->" }));
        }

        // Consume `->`
        self.tokens.consume();
        self.tokens.consume_all_whitespace();

        let external = self.read_ident()?;

        self.tokens.consume_all_whitespace();

        // Consume comma
        if Kind::Op(Operator::Comma) == self.tokens.peek() {
            self.tokens.consume();
            self.tokens.consume_all_whitespace();
        } else if Kind::Op(Operator::RParen) == self.tokens.peek() {
            self.tokens.consume();
            self.next_state();
        } else {
            return Err(self.error(ParseErrorKind::UnterminatedAssociation));
        }

        Ok(Some(Statement::AssociatedFunction { internal, external }))
    }

    fn parse_component_slot(&mut self) -> Result<Option<Statement>, ParseError> {
        if Kind::ComponentSlot != self.tokens.peek_skip_indent() {
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume();
        self.tokens.consume_indent();

        let ident = self.read_ident()?;
        self.tokens.consume_indent();

        self.next_state();
        Ok(Some(Statement::ComponentSlot(ident)))
    }

    // -----------------------------------------------------------------------------
    //     - Stage 3: Parse attributes -
    // -----------------------------------------------------------------------------
    fn parse_attributes(&mut self) -> Result<bool, ParseError> {
        if Kind::Op(Operator::LBracket) == self.tokens.peek_skip_indent() {
            self.tokens.consume();
            self.tokens.consume_all_whitespace();
            self.next_state();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // -----------------------------------------------------------------------------
    //     - Stage 4: Parse single attribute -
    // -----------------------------------------------------------------------------
    fn parse_attribute(&mut self) -> Result<Option<Statement>, ParseError> {
        // Check for the closing bracket
        if Kind::Op(Operator::RBracket) == self.tokens.peek_skip_indent() {
            self.tokens.consume();
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume_all_whitespace();
        let key = self.read_ident()?;
        self.tokens.consume_all_whitespace();

        if Kind::Op(Operator::Colon) != self.tokens.peek_skip_indent() {
            return Err(self.error(ParseErrorKind::InvalidToken { expected: ":" }));
        }

        // Consume `:`
        self.tokens.consume();
        self.tokens.consume_all_whitespace();

        let value = parse_expr(&mut self.tokens, self.strings).map_err(|e| self.error(e))?;

        self.tokens.consume_all_whitespace();

        // Consume comma
        if Kind::Op(Operator::Comma) == self.tokens.peek() {
            self.tokens.consume();
            self.tokens.consume_all_whitespace();
        } else if Kind::Op(Operator::RBracket) == self.tokens.peek() {
            self.tokens.consume();
            self.next_state();
        } else {
            return Err(self.error(ParseErrorKind::UnterminatedAttributes));
        }

        Ok(Some(Statement::LoadAttribute { key, value }))
    }

    // -----------------------------------------------------------------------------
    //     - Stage 5: Node value -
    // -----------------------------------------------------------------------------
    fn parse_value(&mut self) -> Result<Option<Statement>, ParseError> {
        self.tokens.consume_indent();

        if matches!(self.tokens.peek(), Kind::Newline | Kind::Eof) {
            self.next_state();
            return Ok(None);
        }

        let mut values = vec![];

        loop {
            if matches!(self.tokens.peek(), Kind::Newline | Kind::Eof) {
                break;
            }
            let expression = parse_expr(&mut self.tokens, self.strings).map_err(|e| self.error(e))?;
            values.push(expression);
        }

        let value = match values.len() {
            0 => panic!("invalid state"),
            1 => values.remove(0),
            _ => Expression::List(values.into()),
        };

        self.next_state();
        Ok(Some(Statement::LoadValue(value)))
    }

    // -----------------------------------------------------------------------------
    //     - Stage 6: Done -
    //     Clear empty spaces, ready for next instructions,
    //     or deal with EOF
    // -----------------------------------------------------------------------------
    fn parse_done(&mut self) -> Result<Option<Statement>, ParseError> {
        let token = self.tokens.next();

        let ret = match token {
            Kind::Eof if !self.open_scopes.is_empty() => {
                self.open_scopes.pop();
                return Ok(Some(Statement::ScopeEnd));
            }
            Kind::Eof => return Ok(Some(Statement::Eof)),
            Kind::Newline => {
                self.tokens.consume_newlines();
                Ok(None)
            }
            _ => Err(self.error(ParseErrorKind::InvalidToken { expected: "new line" })),
        };

        self.next_state();
        ret
    }
}

// -----------------------------------------------------------------------------
//     - Iterator -
// -----------------------------------------------------------------------------
impl Iterator for Parser<'_, '_, '_> {
    type Item = Result<Statement>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.parse() {
            Ok(Statement::Eof) => {
                self.done = true;
                Some(Ok(Statement::Eof))
            }
            Err(e) => {
                self.state = State::Done;
                Some(Err(e))
            }
            inst => Some(inst),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Error;
    use crate::expressions::{ident, list, map, num, strlit};
    use crate::lexer::Lexer;
    use crate::statements::test::{
        associated_fun, component, decl, else_stmt, eof, for_loop, if_else, if_stmt, load_attrib, load_value, node,
        scope_end, scope_start, slot,
    };

    fn parse(src: &str) -> Vec<Result<Statement>> {
        let mut strings = Strings::empty();
        let mut components = ComponentTemplates::new();
        let lexer = Lexer::new(src, &mut strings);
        let tokens = Tokens::new(lexer.collect::<Result<Vec<_>>>().unwrap(), src.len());
        let parser = Parser::new(tokens, &mut strings, src, &mut components);

        parser.collect::<Vec<_>>()
    }

    fn parse_ok(src: &str) -> Vec<Statement> {
        parse(src).into_iter().map(Result::unwrap).collect()
    }

    fn parse_err(src: &str) -> ParseError {
        match parse(src).into_iter().collect::<Result<Vec<_>>>().unwrap_err() {
            Error::ParseError(err) => err,
            _ => panic!("invalid error kind"),
        }
    }

    #[test]
    fn parse_single_instruction() {
        let actual = parse_ok("a").remove(0);
        assert_eq!(node(0), actual);
    }

    #[test]
    fn parse_attributes() {
        let src = "a [a: a]";
        let expected = vec![node(0), load_attrib(0, ident("a")), eof()];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_text() {
        let src = "a 'a'      \n\n//some comments \n    ";
        let expected = vec![node(0), load_value(strlit("a")), eof()];

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
            node(0),
            scope_start(),
            node(1),
            scope_start(),
            node(2),
            scope_end(),
            node(1),
            scope_end(),
            node(0),
            eof(),
        ];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);

        let src = "
            a
                b
                    c
            ";
        let expected = vec![
            node(0),
            scope_start(),
            node(1),
            scope_start(),
            node(2),
            scope_end(),
            scope_end(),
            eof(),
        ];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_nested_for_loops() {
        let src = "
        x
            for x in data
                for y in data
                    x
        ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), node(0));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), for_loop(0, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), for_loop(2, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(0));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_scopes_and_for() {
        let src = "
        x
            y
        for x in data
            y
        ";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), node(0));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), for_loop(0, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_if() {
        let src = "
        if data
            x
        ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), if_stmt(ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_else() {
        let src = "
        if data
            x
        else
            y
        ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), if_stmt(*ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), else_stmt());
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(2));
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_if_else_if_else() {
        let src = "
        if data
            x
        else if data
            y
        else
            z
        ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), if_stmt(ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), if_else(ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(2));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), else_stmt());
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(3));
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_component() {
        let src = "@mycomp";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), component(0));

        let src = "@mycomp state";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), component(0));
        assert_eq!(statements.remove(0), load_value(ident("state")));
    }

    #[test]
    fn parse_component_slot() {
        let src = "$slot";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), slot(0));
    }

    #[test]
    fn parse_empty_if() {
        let src = "
            if x
            x
        ";

        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), if_stmt(ident("x")));
        assert_eq!(statements.remove(0), node(0));
    }

    #[test]
    fn indented_comment() {
        let src = "
            if x
                // x
            else
        ";

        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), if_stmt(ident("x")));
        assert_eq!(statements.remove(0), else_stmt());
    }

    #[test]
    fn parse_no_instruction() {
        let src = "";
        let expected = vec![eof()];
        let actual = parse_ok(src);
        assert_eq!(expected, actual);

        let src = "\n// comment         \n";
        let expected = vec![eof()];
        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_text_with_multiple_values() {
        let src = "a 'a' 'b'";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), node(0));
        assert_eq!(statements.remove(0), load_value(list([strlit("a"), strlit("b")])));
    }

    #[test]
    fn parse_declaration() {
        let src = "let x = 1";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), decl(0, num(1)));
    }

    #[test]
    fn parse_invalid_declaration() {
        let src = "let x = let y = 1";
        let err = parse_err(src);
        let expected = ParseError {
            kind: ParseErrorKind::InvalidToken {
                expected: "valid token, found statement",
            },
            line: 1,
            col: 9,
            src: "let x = let y = 1".to_string(),
        };

        assert_eq!(err, expected);
    }

    #[test]
    fn empty_multiline_attributes() {
        let src = "
        x [
          ]";
        let _statements = parse_ok(src);
    }

    #[test]
    fn multi_line_declaration() {
        let src = "
        let x = {
            'a': 1,
            'b': {
                'a': 2,
            },
        }";
        let mut statements = parse_ok(src);
        assert_eq!(
            statements.remove(0),
            decl(0, map([("a", num(1)), ("b", map([("a", num(2))]))])),
        );
    }

    #[test]
    fn associated_functions() {
        let src = "@x (inner->outer,another -> out)";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), component(0));
        assert_eq!(statements.remove(0), associated_fun(1, 2));
        assert_eq!(statements.remove(0), associated_fun(3, 4));
    }

    #[test]
    fn associated_functions_multiline() {
        let src = "@x (
            inner->outer,
            another -> out,
        )";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), component(0));
        assert_eq!(statements.remove(0), associated_fun(1, 2));
        assert_eq!(statements.remove(0), associated_fun(3, 4));
    }
}
