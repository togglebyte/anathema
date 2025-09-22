use super::Statement;
use crate::components::{AssocEventMapping, ComponentTemplates, TemplateSource};
use crate::error::{Error, ParseError, ParseErrorKind, Result, src_line_no};
use crate::expressions::Expression;
use crate::expressions::parser::parse_expr;
use crate::strings::{StringId, Strings};
use crate::token::{Kind, Operator, Tokens, Value};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State {
    EnterScope,
    ExitScope,
    ParseWith,
    ParseFor,
    ParseIf,
    ParseSwitch,
    ParseCase,
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
    src: &'src TemplateSource,
    state: State,
    open_scopes: Vec<usize>,
    closed_scopes: Vec<usize>,
    base_indent: usize,
    done: bool,
}

impl<'src, 'strings, 'components> Parser<'src, 'strings, 'components> {
    pub(crate) fn new(
        mut tokens: Tokens,
        strings: &'strings mut Strings,
        src: &'src TemplateSource,
        components: &'components mut ComponentTemplates,
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

    fn error(&self, kind: ParseErrorKind) -> Error {
        let (line, col) = src_line_no(self.tokens.previous().1, self.src);
        ParseError {
            line,
            col,
            src: self.src.to_string(),
            kind,
        }
        .to_error(self.src.path())
    }

    fn read_ident(&mut self) -> Result<StringId> {
        match self.tokens.next_no_indent() {
            Kind::Value(Value::Ident(ident)) => Ok(ident),
            _ => Err(self.error(ParseErrorKind::InvalidToken { expected: "identifier" })),
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Statement> {
        // * It is okay to advance the state once and only once
        //   in any parse function using `self.next_state()`.
        //   The exception to this is the parse_switch function,
        //   where we skip to EnterScope.
        // * State should never be set directly in any of the parse functions.
        //   There is one exception of this, and that's when moving from
        //   `ParseAttributes` to `ParseText`.
        loop {
            let output = match self.state {
                State::EnterScope => self.enter_scope()?,
                State::ParseWith => self.parse_with()?,
                State::ParseFor => self.parse_for()?,
                State::ParseIf => self.parse_if()?,
                State::ParseSwitch => self.parse_switch()?,
                State::ParseCase => self.parse_case()?,
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
            State::ExitScope => self.state = State::ParseWith,
            State::ParseWith => self.state = State::ParseFor,
            State::ParseFor => self.state = State::ParseIf,
            State::ParseIf => self.state = State::ParseSwitch,
            State::ParseSwitch => self.state = State::ParseCase,
            State::ParseCase => self.state = State::ParseDeclaration,
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
    fn enter_scope(&mut self) -> Result<Option<Statement>> {
        let indent = self.tokens.read_indent();

        if let Kind::Op(op @ Operator::Semicolon) = self.tokens.peek() {
            return Err(self.error(ParseErrorKind::InvalidOperator(op)));
        }

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

    fn exit_scope(&mut self) -> Result<Option<Statement>> {
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
    fn parse_ident(&mut self) -> Result<Option<Statement>> {
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

    fn parse_with(&mut self) -> Result<Option<Statement>> {
        // with <binding> as <expr>
        if Kind::With != self.tokens.peek_skip_indent() {
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume();

        let binding = self.read_ident()?;

        if Kind::As != self.tokens.peek_skip_indent() {
            return Err(self.error(ParseErrorKind::InvalidToken { expected: "as" }));
        }
        // Consume `As`
        self.tokens.consume();

        let data = match parse_expr(&mut self.tokens, self.strings) {
            Ok(data) => data,
            Err(e) => return Err(self.error(e)),
        };

        self.next_state();
        Ok(Some(Statement::With { data, binding }))
    }

    fn parse_for(&mut self) -> Result<Option<Statement>> {
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

    fn parse_if(&mut self) -> Result<Option<Statement>> {
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
                self.state = State::EnterScope;
                Ok(Some(Statement::If(cond)))
            }
            _ => {
                self.next_state();
                Ok(None)
            }
        }
    }

    fn parse_switch(&mut self) -> Result<Option<Statement>> {
        match self.tokens.peek_skip_indent() {
            Kind::Switch => {
                self.tokens.consume();
                let cond = parse_expr(&mut self.tokens, self.strings).map_err(|e| self.error(e))?;
                self.state = State::EnterScope;
                Ok(Some(Statement::Switch(cond)))
            }
            _ => {
                self.next_state();
                Ok(None)
            }
        }
    }

    fn parse_case(&mut self) -> Result<Option<Statement>> {
        // <ident> : <body>
        match self.tokens.peek_skip_indent() {
            kind @ (Kind::Case | Kind::Default) => {
                self.tokens.consume();

                let stmt = match kind {
                    Kind::Case => {
                        Statement::Case(parse_expr(&mut self.tokens, self.strings).map_err(|e| self.error(e))?)
                    }
                    Kind::Default => Statement::Default,
                    _ => unreachable!(),
                };

                // peek colon
                let Kind::Op(Operator::Colon) = self.tokens.peek() else {
                    return Err(self.error(ParseErrorKind::UnterminatedCase));
                };
                self.tokens.consume();
                self.next_state();
                Ok(Some(stmt))
            }
            _ => {
                self.next_state();
                Ok(None)
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Option<Statement>> {
        // Check if it's a declaration otherwise move on
        let is_global = match self.tokens.peek_skip_indent() {
            Kind::Local => false,
            Kind::Global => true,
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
            let statement = Statement::Declaration {
                binding,
                value,
                is_global,
            };
            return Ok(Some(statement));
        }

        self.next_state();
        Ok(None)
    }

    fn parse_component(&mut self) -> Result<Option<Statement>> {
        if Kind::Component != self.tokens.peek_skip_indent() {
            self.next_state();
            return Ok(None);
        }

        self.tokens.consume();
        self.tokens.consume_indent();

        let ident = self.read_ident()?;
        let component_id = match self.components.get_component_by_string_id(ident) {
            Some(cid) => cid,
            None => return Err(self.error(ParseErrorKind::UnregisteredComponent(self.strings.get_unchecked(ident)))),
        };
        self.tokens.consume_indent();

        self.next_state();
        Ok(Some(Statement::Component(component_id)))
    }

    fn parse_associated_functions(&mut self) -> Result<bool> {
        if Kind::Op(Operator::LParen) == self.tokens.peek_skip_indent() {
            self.tokens.consume();
            self.tokens.consume_all_whitespace();
            self.next_state();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn parse_associated_function(&mut self) -> Result<Option<Statement>> {
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

        Ok(Some(Statement::AssociatedFunction(AssocEventMapping {
            internal,
            external,
        })))
    }

    fn parse_component_slot(&mut self) -> Result<Option<Statement>> {
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
    fn parse_attributes(&mut self) -> Result<bool> {
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
    fn parse_attribute(&mut self) -> Result<Option<Statement>> {
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
    fn parse_value(&mut self) -> Result<Option<Statement>> {
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
            _ => Expression::TextSegments(values),
        };

        self.next_state();
        Ok(Some(Statement::LoadValue(value)))
    }

    // -----------------------------------------------------------------------------
    //     - Stage 6: Done -
    //     Clear empty spaces, ready for next instructions,
    //     or deal with EOF
    // -----------------------------------------------------------------------------
    fn parse_done(&mut self) -> Result<Option<Statement>> {
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
    use crate::error::ErrorKind;
    use crate::expressions::{boolean, ident, map, num, range, strlit, text_segments};
    use crate::lexer::Lexer;
    use crate::statements::test::{
        associated_fun, case, component, else_stmt, eof, for_loop, global, if_else, if_stmt, load_attrib, load_value,
        local, node, scope_end, scope_start, slot, switch, with,
    };

    fn parse(src: TemplateSource) -> Vec<Result<Statement>> {
        let mut strings = Strings::new();
        let x_id = strings.push("x");
        let mut components = ComponentTemplates::new();
        components.insert(x_id, crate::components::TemplateSource::InMemory(String::new()));
        let lexer = Lexer::new(&src, &mut strings);
        let tokens = Tokens::new(lexer.collect::<Result<Vec<_>>>().unwrap(), src.len());
        let parser = Parser::new(tokens, &mut strings, &src, &mut components);
        parser.collect::<Vec<_>>()
    }

    fn parse_ok(src: &'static str) -> Vec<Statement> {
        match parse(src.into()).into_iter().collect::<Result<Vec<_>>>() {
            Ok(stmts) => stmts,
            Err(e) => panic!("{e}"),
        }
    }

    fn parse_err(src: &'static str) -> ParseError {
        match parse(src.into()).into_iter().collect::<Result<Vec<_>>>().unwrap_err() {
            Error {
                kind: ErrorKind::ParseError(err),
                ..
            } => err,
            _ => panic!("invalid error kind"),
        }
    }

    #[test]
    fn parse_single_instruction() {
        let actual = parse_ok("a").remove(0);
        assert_eq!(node(2), actual);
    }

    #[test]
    fn parse_attributes() {
        let src = "a [a: a]";
        let expected = vec![node(2), load_attrib(2, ident("a")), eof()];

        let actual = parse_ok(src);
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_text() {
        let src = "a 'a'      \n\n//some comments \n    ";
        let expected = vec![node(2), load_value(strlit("a")), eof()];

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
            node(2),
            scope_start(),
            node(3),
            scope_start(),
            node(4),
            scope_end(),
            node(3),
            scope_end(),
            node(2),
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
            node(2),
            scope_start(),
            node(3),
            scope_start(),
            node(4),
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

        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), for_loop(1, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), for_loop(3, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_with() {
        let src = "
            with val as data
                x val
        ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), with(2, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), load_value(ident("val")));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), eof());
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
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(2));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), for_loop(1, ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(2));
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
        assert_eq!(statements.remove(0), node(3));
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
        assert_eq!(statements.remove(0), node(3));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), else_stmt());
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(4));
        assert_eq!(statements.remove(0), scope_end());
    }

    #[test]
    fn parse_switch_oneline_case() {
        let src = "
            switch data
                case true: x
                case false: y
    ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), switch(ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), case(boolean(true)));
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), case(boolean(false)));
        assert_eq!(statements.remove(0), node(3));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), eof());
    }

    #[test]
    fn parse_switch_case() {
        let src = "
            switch data
                case true: 
                    x
                    x
                case false: 
                    y
                    y
    ";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), switch(ident("data")));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), case(boolean(true)));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), node(1));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), case(boolean(false)));
        assert_eq!(statements.remove(0), scope_start());
        assert_eq!(statements.remove(0), node(3));
        assert_eq!(statements.remove(0), node(3));
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), scope_end());
        assert_eq!(statements.remove(0), eof());
    }

    #[test]
    fn parse_component() {
        let src = "@x";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), component(0));

        let src = "@x state";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), component(0));
        assert_eq!(statements.remove(0), load_value(ident("state")));
    }

    #[test]
    fn parse_component_slot() {
        let src = "$slot";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), slot(2));
    }

    #[test]
    fn parse_empty_if() {
        let src = "
            if x
            x
        ";

        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), if_stmt(ident("x")));
        assert_eq!(statements.remove(0), node(1));
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
        assert_eq!(statements.remove(0), node(2));
        assert_eq!(
            statements.remove(0),
            load_value(text_segments([strlit("a"), strlit("b")]))
        );
    }

    #[test]
    fn parse_range() {
        let src = "for i in x..y";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), for_loop(2, range(ident("x"), ident("y"))));
    }

    #[test]
    fn parse_local_declaration() {
        let src = "let x = 1";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), local(1, num(1)));
    }

    #[test]
    fn parse_global_declaration() {
        let src = "global x = 1";
        let mut statements = parse_ok(src);
        assert_eq!(statements.remove(0), global(1, num(1)));
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
            local(1, map([("a", num(1)), ("b", map([("a", num(2))]))])),
        );
    }

    #[test]
    fn associated_functions() {
        let src = "@x (inner->outer,another -> out)";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), component(0));
        assert_eq!(statements.remove(0), associated_fun(2, 3));
        assert_eq!(statements.remove(0), associated_fun(4, 5));
    }

    #[test]
    fn associated_functions_multiline() {
        let src = "@x (
            inner->outer,
            another -> out,
        )";
        let mut statements = parse_ok(src);

        assert_eq!(statements.remove(0), component(0));
        assert_eq!(statements.remove(0), associated_fun(2, 3));
        assert_eq!(statements.remove(0), associated_fun(4, 5));
    }
}
