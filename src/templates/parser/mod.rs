use std::iter::Peekable;
use std::time::Duration;

use crate::display::Color;

use crate::widgets::{fields, Attribute};
use crate::widgets::{Align, Axis, BorderStyle, Display, Sides, TextAlignment, Wrap};
use crate::widgets::{Easing, Fragment, Number, Path, Value};

use crate::templates::ctx::SubContext;
use crate::templates::nodes::template::TemplateNode;

use self::error::{Error, Result};
use lexer::{Lexer, Meta, Token, TokenKind};

pub mod error;
pub(crate) mod lexer;

pub type Indent = usize;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Text {
    String(String),
    Fragments(Vec<Fragment>),
}

impl Text {
    #[cfg(test)]
    pub fn fragments(self) -> Vec<Fragment> {
        match self {
            Self::String(s) => vec![Fragment::String(s)],
            Self::Fragments(f) => f,
        }
    }

    pub(crate) fn path(&self, data_ctx: &SubContext) -> String {
        let mut buffer = String::new();
        match self {
            Text::String(s) => buffer.push_str(s),
            Text::Fragments(fragments) => fragments.iter().for_each(|frag| match frag {
                Fragment::String(s) => buffer.push_str(s),
                Fragment::Data(ref path) => {
                    let value = data_ctx.by_path(path);
                    let value = value.map(Value::to_string);
                    if let Some(val) = value {
                        buffer.push_str(&val);
                    }
                }
            }),
        }

        buffer
    }
}

impl Default for Text {
    fn default() -> Self {
        Self::String(String::new())
    }
}

pub(crate) struct Parser<'src> {
    src: &'src str,
    lexer: Peekable<Lexer<'src>>,
    base_indent: Option<usize>,
}

impl<'src> Iterator for Parser<'src> {
    type Item = Result<(usize, TemplateNode<'src>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_node()
    }
}

impl<'src> Parser<'src> {
    pub(crate) fn new(lexer: Lexer<'src>) -> Self {
        let mut inst = Self { src: lexer.src, lexer: lexer.peekable(), base_indent: None };

        while let Some(Ok(Token(TokenKind::Newline, _))) = inst.lexer.peek() {
            let _ = inst.lexer.next();
        }

        inst
    }

    fn parse_node(&mut self) -> Option<Result<(Indent, TemplateNode<'src>)>> {
        // Consume newlines
        // Consume all comment lines
        while self.consume_comment() || self.consume_newline() {}

        // -----------------------------------------------------------------------------
        //     - Ident -
        // -----------------------------------------------------------------------------
        let token = match self.lexer.next()? {
            Ok(token) => token,
            Err(e) => return Some(Err(e)),
        };

        let (ident, indent) = match token.0 {
            TokenKind::Whitespace(mut indent) => {
                indent = match self.base_indent {
                    Some(base) => indent - base,
                    None => {
                        self.base_indent = Some(indent);
                        0
                    }
                };

                match self.lexer.next()? {
                    Ok(Token(TokenKind::Ident(ident), _)) => (ident, indent),
                    Ok(Token(TokenKind::Newline, _)) => return self.parse_node(),
                    Ok(Token(kind, meta)) => {
                        return Some(Err(Error::invalid_token(token.1.pos..meta.pos, self.src, kind, "ident")))
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            TokenKind::Ident(ident) => {
                if self.base_indent.is_none() {
                    self.base_indent = Some(0);
                }
                (ident, 0)
            }
            kind => {
                let end = match self.lexer.next() {
                    Some(Ok(t)) => t.1.pos,
                    _ => self.src.len(),
                };
                return Some(Err(Error::invalid_token(token.1.pos..end, self.src, kind, "ident or indent")));
            }
        };

        self.consume_whitespace();

        // -----------------------------------------------------------------------------
        //     - Attributes -
        // -----------------------------------------------------------------------------
        let attributes = match self.parse_attributes() {
            Ok(attributes) => attributes,
            Err(e) => return Some(Err(e)),
        };

        self.consume_whitespace();

        // -----------------------------------------------------------------------------
        //     - Text -
        //     This will also parse the terminating colon
        // -----------------------------------------------------------------------------
        let text = match self.parse_text() {
            Ok(text) => text,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok((indent, TemplateNode::new(ident, attributes, text))))
    }

    fn consume_comment(&mut self) -> bool {
        match self.lexer.peek() {
            Some(Ok(Token(TokenKind::Comment, _))) => {
                drop(self.lexer.next());
                // if the next token is a newline char, that means the entier
                // line is a comment line and should be consumed
                if let Some(Ok(Token(TokenKind::Newline, _))) = self.lexer.peek() {
                    drop(self.lexer.next());
                }
                true
            }
            _ => false,
        }
    }

    fn consume_newline(&mut self) -> bool {
        if let Some(Ok(Token(TokenKind::Newline, _))) = self.lexer.peek() {
            drop(self.lexer.next());
            true
        } else {
            false
        }
    }

    fn consume_whitespace(&mut self) {
        while let Some(Ok(Token(TokenKind::Whitespace(_) | TokenKind::Comment, _))) = self.lexer.peek() {
            drop(self.lexer.next());
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute<'src>>> {
        let mut attributes = vec![];
        // If the next token is a LBracket then we have attributes,
        // else return an empty vec and move on!

        let _start_pos = if let Some(Ok(Token(TokenKind::LBracket, meta))) = self.lexer.peek() {
            meta.pos
        } else {
            return Ok(attributes);
        };

        self.consume_next(); // consume the left bracket

        loop {
            match self.lexer.peek() {
                Some(Ok(Token(TokenKind::RBracket, _))) => {
                    self.consume_next();
                    break;
                }
                Some(Ok(Token(TokenKind::Comma, _))) => {
                    self.consume_next();
                    continue;
                }
                _ => {}
            }

            let attribs = self.parse_attribute()?;
            attributes.push(attribs);

            self.consume_whitespace();
        }

        Ok(attributes)
    }

    fn parse_attribute(&mut self) -> Result<Attribute<'src>> {
        self.consume_whitespace();

        let left = match self.lexer.next() {
            Some(Ok(Token(TokenKind::Ident(ident), _))) => ident,
            Some(Ok(invalid_token)) => {
                let end = match self.lexer.peek() {
                    Some(Ok(token)) => token.1.pos,
                    _ => self.src.len(),
                };

                return Err(Error::invalid_attribute(
                    invalid_token.1.pos..end,
                    self.src,
                    &self.src[invalid_token.1.pos..end],
                    None,
                ));
            }
            Some(Err(e)) => return Err(e),
            None => return Err(Error::unexpected_end(self.src)),
        };

        self.consume_whitespace();
        self.consume_next(); // Colon

        let right = self.try_parse_attribute_value(left)?;

        self.consume_whitespace();

        let attribute = Attribute { key: left, val: right };

        Ok(attribute)
    }

    fn parse_text(&mut self) -> Result<Option<Text>> {
        // If there is no text, this must be the ending `:`
        // otherwise consume any trailing whitespace and move
        // on to the text
        match self.lexer.peek() {
            Some(Ok(Token(TokenKind::Colon, _))) => self.consume_next(),
            _ => {
                self.consume_whitespace();
                match self.lexer.next() {
                    Some(Ok(Token(TokenKind::Colon, _))) => {}
                    Some(Ok(Token(_, Meta { pos: start }))) => {
                        return Err(Error::unterminated_element(start..start + 1, self.src))
                    }
                    _ => return Err(Error::unexpected_end(self.src)),
                }
            }
        }

        let token = self.lexer.next();
        match token {
            Some(Ok(Token(TokenKind::String(s), Meta { pos: start }))) => {
                self.consume_whitespace();
                match self.lexer.next() {
                    // The next token after the `String` should either be a `\n`, or nothing
                    Some(Ok(Token(TokenKind::Newline, _))) => {} //drop(self.lexer.next()),
                    // Invalid token
                    Some(Ok(Token(invalid_token, meta))) => {
                        return Err(Error::invalid_token(start..meta.pos, self.src, invalid_token, "new line"))
                    }
                    Some(Err(err)) => return Err(err),
                    None => (),
                }
                Ok(Some(parse_to_fragments(s)))
            }
            Some(Ok(Token(TokenKind::Newline, _))) => Ok(None),
            Some(Err(e)) => Err(e),
            _ => Ok(None),
        }
    }

    // Consume the next token and any trailing whitespace
    fn consume_next(&mut self) {
        let _ = self.lexer.next();
        self.consume_whitespace();
    }

    fn try_parse_attribute_value(&mut self, left: &'src str) -> Result<Value> {
        match self.lexer.next() {
            Some(Ok(Token(TokenKind::String(border_style), _))) if left == fields::BORDER_STYLE => {
                Ok(Value::BorderStyle(BorderStyle::Custom(border_style.to_string())))
            }
            Some(Ok(Token(TokenKind::String(val), _))) => match parse_to_fragments(val) {
                Text::String(s) => Ok(Value::String(s)),
                Text::Fragments(fragments) => Ok(Value::Fragments(fragments)),
            },
            Some(Ok(Token(TokenKind::Hex(r, g, b), _))) => Ok(Value::Color(Color::Rgb { r, g, b })),
            Some(Ok(Token(TokenKind::Ident(b @ "true" | b @ "false"), _))) => {
                match b {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ => unsafe { std::hint::unreachable_unchecked() }, // this could not possible be anything else!
                }
            }
            Some(Ok(Token(TokenKind::Ident(fields::ANIMATE), Meta { pos: start }))) => self.try_parse_animate(start),
            Some(Ok(Token(TokenKind::Ident(val), Meta { pos: start }))) => {
                let val = val.trim();
                match left {
                    fields::ALIGNMENT => match val {
                        "top" => Ok(Value::Alignment(Align::Top)),
                        "top-right" => Ok(Value::Alignment(Align::TopRight)),
                        "right" => Ok(Value::Alignment(Align::Right)),
                        "bottom-right" => Ok(Value::Alignment(Align::BottomRight)),
                        "bottom" => Ok(Value::Alignment(Align::Bottom)),
                        "bottom-left" => Ok(Value::Alignment(Align::BottomLeft)),
                        "left" => Ok(Value::Alignment(Align::Left)),
                        "top-left" => Ok(Value::Alignment(Align::TopLeft)),
                        "centre" | "center" => Ok(Value::Alignment(Align::Centre)),
                        _ => Err(Error::invalid_attribute(start..start + val.len(), self.src, left, Some(val))),
                    },
                    fields::AXIS | fields::DIRECTION => match val {
                        "horizontal" | "horz" => Ok(Value::Axis(Axis::Horizontal)),
                        "vertical" | "vert" => Ok(Value::Axis(Axis::Vertical)),
                        _ => Err(Error::invalid_attribute(start..start + val.len(), self.src, left, Some(val))),
                    },
                    fields::BORDER_STYLE => match val {
                        "thick" => Ok(Value::BorderStyle(BorderStyle::Thick)),
                        "thin" => Ok(Value::BorderStyle(BorderStyle::Thin)),
                        chars => Ok(Value::BorderStyle(BorderStyle::Custom(chars.to_string()))),
                    },
                    fields::DISPLAY => match val {
                        "show" => Ok(Value::Display(Display::Show)),
                        "hide" => Ok(Value::Display(Display::Hide)),
                        "exclude" => Ok(Value::Display(Display::Exclude)),
                        _ => Err(Error::invalid_attribute(start..start + val.len(), self.src, left, Some(val))),
                    },
                    fields::SIDES => {
                        let mut sides = self.parse_side(val, start)?;
                        self.consume_whitespace();
                        if self.next_is_pipe(false).is_some() {
                            while let Some(pipe_pos) = self.next_is_pipe(true) {
                                self.consume_whitespace();
                                match self.lexer.next() {
                                    Some(Ok(Token(TokenKind::Ident(ident), Meta { pos: start }))) => {
                                        sides |= self.parse_side(ident, start)?;
                                        self.consume_whitespace();
                                    }
                                    _ => return Err(Error::trailing_pipe(pipe_pos, self.src)),
                                }
                            }
                        }

                        Ok(Value::Sides(sides))
                    }
                    fields::TEXT_ALIGN => match val {
                        "centre" | "center" => Ok(Value::TextAlignment(TextAlignment::Centre)),
                        "left" => Ok(Value::TextAlignment(TextAlignment::Left)),
                        "right" => Ok(Value::TextAlignment(TextAlignment::Right)),
                        _ => Err(Error::invalid_attribute(start..start + val.len(), self.src, left, Some(val))),
                    },
                    fields::WRAP => match val {
                        "no-wrap" => Ok(Value::Wrap(Wrap::NoWrap)),
                        "break" => Ok(Value::Wrap(Wrap::Break)),
                        "word" => Ok(Value::Wrap(Wrap::Word)),
                        _ => Err(Error::invalid_attribute(start..start + val.len(), self.src, left, Some(val))),
                    },
                    _custom_attribute => match self.try_parse_color(val) {
                        Some(color) => Ok(Value::Color(color)),
                        None => Ok(Value::String(val.to_string())),
                    },
                }
            }
            Some(Ok(Token(TokenKind::Number(val), _))) => Ok(Value::Number(val)),
            Some(Ok(Token(TokenKind::LDoubleCurly, Meta { pos: start }))) => {
                self.consume_whitespace();
                let ret = match self.lexer.next() {
                    Some(Ok(Token(TokenKind::Ident(ident), _))) => {
                        let path = self.try_parse_path(ident)?;
                        Ok(Value::DataBinding(path))
                    }
                    _ => Err(Error::invalid_attribute(start..start + 1, self.src, left, None)),
                };
                self.consume_whitespace();
                match self.lexer.next() {
                    Some(Ok(Token(TokenKind::RDoubleCurly, _))) => ret,
                    _ => Err(Error::invalid_attribute(start..start + 1, self.src, left, None)),
                }
            }
            Some(Ok(invalid_token)) => {
                let end = match self.lexer.peek() {
                    Some(Ok(token)) => token.1.pos,
                    _ => self.src.len(),
                };

                Err(Error::invalid_attribute(
                    invalid_token.1.pos..end,
                    self.src,
                    &self.src[invalid_token.1.pos..end],
                    None,
                ))
            }
            Some(Err(e)) => Err(e),
            None => Err(Error::unexpected_end(self.src)),
        }
    }

    fn try_parse_color(&mut self, maybe_color: &str) -> Option<Color> {
        match maybe_color {
            "black" => Some(Color::Black),
            "blue" => Some(Color::Blue),
            "cyan" => Some(Color::Cyan),
            "dark-blue" => Some(Color::DarkBlue),
            "dark-cyan" => Some(Color::DarkCyan),
            "dark-green" => Some(Color::DarkGreen),
            "dark-grey" => Some(Color::DarkGrey),
            "dark-magenta" => Some(Color::DarkMagenta),
            "dark-red" => Some(Color::DarkRed),
            "dark-yellow" => Some(Color::DarkYellow),
            "green" => Some(Color::Green),
            "grey" => Some(Color::Grey),
            "magenta" => Some(Color::Magenta),
            "red" => Some(Color::Red),
            "reset" => Some(Color::Reset),
            "white" => Some(Color::White),
            "yellow" => Some(Color::Yellow),
            _ => None,
        }
    }

    // -----------------------------------------------------------------------------
    //     - Animation -
    //     TODO: The error reporting should be improved
    // -----------------------------------------------------------------------------
    fn try_parse_animate(&mut self, start_pos: usize) -> Result<Value> {
        // Consume open bracket
        if !matches!(self.lexer.next(), Some(Ok(Token(TokenKind::LParen, _)))) {
            return Err(Error::invalid_attribute(start_pos..start_pos + 1, self.src, fields::ANIMATE, None));
        }

        let value = self.try_parse_attribute_value(fields::ANIMATE)?;

        self.consume_whitespace();

        // Consume comma
        if !matches!(self.lexer.next(), Some(Ok(Token(TokenKind::Comma, _)))) {
            return Err(Error::invalid_attribute(start_pos..start_pos + 1, self.src, fields::ANIMATE, None));
        }

        self.consume_whitespace();

        // Seconds
        let ms = match self.lexer.next() {
            Some(Ok(Token(TokenKind::Number(Number::Unsigned(seconds)), _))) => seconds,
            _ => return Err(Error::invalid_attribute(start_pos..start_pos + 1, self.src, fields::ANIMATE, None)),
        };

        self.consume_whitespace();

        // If the next token is a comma then that's the easing function
        let easing = if matches!(self.lexer.peek(), Some(Ok(Token(TokenKind::Comma, _)))) {
            let _ = self.lexer.next(); // consume comma
            self.consume_whitespace();

            match self.lexer.next() {
                Some(Ok(Token(TokenKind::Ident(ident), _))) => match ident {
                    "linear" => Easing::Linear,
                    "ease-in" => Easing::EaseIn,
                    "ease-out" => Easing::EaseOut,
                    "ease-inout" => Easing::EaseInOut,
                    _ => {
                        return Err(Error::invalid_attribute(start_pos..start_pos + 1, self.src, fields::ANIMATE, None))
                    }
                },
                _ => return Err(Error::invalid_attribute(start_pos..start_pos + 1, self.src, fields::ANIMATE, None)),
            }
        } else {
            Easing::Linear
        };

        // Consume closed bracket
        if !matches!(self.lexer.next(), Some(Ok(Token(TokenKind::RParen, _)))) {
            return Err(Error::invalid_attribute(start_pos..start_pos + 1, self.src, fields::ANIMATE, None));
        }

        Ok(Value::Transition(Box::new(value), Duration::from_millis(ms), easing))
    }

    fn try_parse_path(&mut self, ident: &str) -> Result<Path> {
        parse_path(&mut self.lexer, ident)
    }

    // -----------------------------------------------------------------------------
    //     - Convenience functions -
    // -----------------------------------------------------------------------------
    fn next_is_pipe(&mut self, consume: bool) -> Option<usize> {
        match self.lexer.peek() {
            Some(Ok(Token(TokenKind::Pipe, Meta { pos }))) => {
                let pos = *pos;
                if consume {
                    drop(self.lexer.next());
                }
                Some(pos)
            }
            _ => None,
        }
    }

    fn parse_side(&self, input: &'src str, start_pos: usize) -> Result<Sides> {
        let end = start_pos + input.len();
        match input {
            "top" => Ok(Sides::TOP),
            "right" => Ok(Sides::RIGHT),
            "bottom" => Ok(Sides::BOTTOM),
            "left" => Ok(Sides::LEFT),
            "all" => Ok(Sides::ALL),
            _ => Err(Error::invalid_attribute(start_pos..end, self.src, fields::SIDES, Some(input))),
        }
    }
}

// -----------------------------------------------------------------------------
//     - Parse string into fragments -
// -----------------------------------------------------------------------------
fn parse_to_fragments(text: &str) -> Text {
    let mut fragments = vec![];
    let mut chars = text.char_indices().peekable();
    let mut pos = 0;

    while let Some(c) = chars.next() {
        let next = chars.peek();
        match (c, next) {
            ((i, '{'), Some((_, '{'))) => {
                let frag = &text[pos..i];
                if !frag.is_empty() {
                    fragments.push(Fragment::String(frag.replace("\\\"", "\"")));
                }
                pos = i;
            }
            ((i, '}'), Some((_, '}'))) => {
                let frag = &text[pos + 2..i].trim();
                if !frag.is_empty() {
                    let mut lexer = Lexer::new(frag);
                    if let Some(Ok(Token(TokenKind::Ident(ident), _))) = lexer.next() {
                        if let Ok(path) = parse_path(&mut lexer.peekable(), ident) {
                            fragments.push(Fragment::Data(path));
                        }
                    }
                }
                pos = i + 2;
            }
            _ => {}
        }
    }

    let remainder = &text[pos..];

    if !remainder.is_empty() {
        fragments.push(Fragment::String(remainder.replace("\\\"", "\"")));
    }

    if fragments.len() == 1 && fragments[0].is_string() {
        let s = match fragments.remove(0) {
            Fragment::String(s) => s,
            _ => unreachable!(),
        };
        Text::String(s)
    } else {
        Text::Fragments(fragments)
    }
}

// -----------------------------------------------------------------------------
//     - Parse path -
//  Note: this is not part of the `Parser` as this is used in other
//  places to parse paths
// -----------------------------------------------------------------------------
fn parse_path(lexer: &mut Peekable<Lexer>, ident: &str) -> Result<Path> {
    let mut path = Path::new(ident);

    let mut stack = Vec::new();
    loop {
        if let Some(Ok(Token(TokenKind::Fullstop, _))) = lexer.peek() {
            lexer.next();
        } else {
            break;
        }

        match lexer.next() {
            Some(Ok(Token(TokenKind::Ident(ident), _))) => stack.push(Path::new(ident)),
            Some(Ok(Token(TokenKind::Number(num), _))) => stack.push(Path::new(&num.to_string())),
            _ => {}
        }
    }

    while let Some(child) = stack.pop() {
        match stack.last_mut() {
            Some(last) => last.child = Some(Box::new(child)),
            None => path.child = Some(Box::new(child)),
        }
    }

    Ok(path)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::templates::parser::error::ErrorKind;
    use crate::widgets::Attributes;

    fn parse_attributes(template: &str) -> Attributes {
        let lexer = Lexer::new(template);
        let mut parser = Parser::new(lexer);
        let TemplateNode { attributes, .. } = parser.next().unwrap().map(|(_, tn)| tn).unwrap();
        attributes
    }

    #[test]
    fn parse_a_widget() {
        let src = "widget [first: red, second: -2]: \"text\"";
        let lexer = Lexer::new(src);
        let mut parser = Parser::new(lexer);

        let (_ident, output) = parser.next().unwrap().unwrap();

        assert_eq!(output.attributes.get_value("first").unwrap(), Value::Color(Color::Red));
        assert_eq!(output.attributes.get_value("second").unwrap(), Value::Number(Number::Signed(-2)));
        assert!(matches!(output, TemplateNode { ident: "widget", .. }));
    }

    #[test]
    fn parse_height() {
        let attributes = parse_attributes("container [height:1]:");
        let actual = attributes.height().unwrap();
        assert_eq!(1, actual);
    }

    #[test]
    fn parse_width() {
        let attributes = parse_attributes("container [width:1]:");
        let actual = attributes.width().unwrap();
        assert_eq!(1, actual);
    }

    #[test]
    fn sides() {
        let attributes = parse_attributes("widget [sides: left|top]:");
        let actual = attributes.sides();
        assert_eq!(Sides::LEFT | Sides::TOP, actual);
    }

    #[test]
    fn transition_with_easing() {
        let attributes = parse_attributes("position [left: animate(10, 2000, ease-in)]:");

        let transition = attributes.get_value("left").unwrap();

        assert_eq!(
            transition,
            Value::Transition(
                Box::new(Value::Number(Number::Unsigned(10))),
                Duration::from_millis(2000),
                Easing::EaseIn
            )
        );
    }

    #[test]
    fn transition_default_easing() {
        let attributes = parse_attributes("position [left: animate(10, 2000)]:");

        let transition = attributes.get_value("left").unwrap();

        assert_eq!(
            transition,
            Value::Transition(
                Box::new(Value::Number(Number::Unsigned(10))),
                Duration::from_millis(2000),
                Easing::Linear
            )
        );
    }

    #[test]
    fn string_fragments() {
        let text = parse_to_fragments("a{{b}}");
        let fragments = text.fragments();

        assert_eq!(fragments[0], Fragment::String("a".to_string()));
        assert_eq!(fragments[1], Fragment::Data(Path::new("b")));
    }

    #[test]
    fn escaped_string() {
        let text = parse_to_fragments("a\\\"b");
        let fragments = text.fragments();

        assert_eq!(fragments[0], Fragment::String("a\"b".to_string()));
    }

    #[test]
    fn path() {
        let mut lexer = Lexer::new(".a.b.c").peekable();
        let mut path = parse_path(&mut lexer, "root").unwrap();

        assert_eq!(&path.name, "root");

        path = *path.child.unwrap();
        assert_eq!(&path.name, "a");

        path = *path.child.unwrap();
        assert_eq!(&path.name, "b");

        path = *path.child.unwrap();
        assert_eq!(&path.name, "c");
    }

    #[test]
    fn parse_invalid_element() {
        // Note that the first "border" in the source is missing
        // the terminating colon
        let unterminated_element = "border\n    border:";
        let lexer = Lexer::new(unterminated_element);
        let mut parser = Parser::new(lexer);
        let err = parser.next().unwrap().unwrap_err();
        assert!(matches!(err.kind, ErrorKind::UnterminatedElement));
    }

    #[test]
    fn unexpecetd_end() {
        let unterminated_element = "bor";
        let lexer = Lexer::new(unterminated_element);
        let mut parser = Parser::new(lexer);
        let err = parser.next().unwrap().unwrap_err();
        assert!(matches!(err.kind, ErrorKind::UnexpectedEnd));
    }
}
