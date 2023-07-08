use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;
use std::str::CharIndices;

use anathema_widget_core::Number;

use crate::error::{Error, Result};

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Kind<'src> {
    Colon,
    Comma,
    Comment,
    LDoubleCurly,
    RDoubleCurly,
    Hex(u8, u8, u8),
    For,
    In,
    If,
    Else,
    View,
    Ident(&'src str),
    Newline,
    Number(Number),
    Pipe,
    Fullstop,
    LBracket,
    RBracket,
    LParen,
    RParen,
    String(&'src str),
    Indent(usize),
    EOF,
}

impl<'src> Kind<'src> {
    fn to_token(self, pos: usize) -> Token<'src> {
        Token(self, pos)
    }
}

impl<'src> Display for Kind<'src> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct Token<'src>(pub(crate) Kind<'src>, pub(crate) usize);

pub struct Lexer<'src> {
    pub(super) src: &'src str,
    chars: Peekable<CharIndices<'src>>,
    next: Option<Result<Token<'src>>>,
    pub(super) current_pos: usize,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self {
            chars: src.char_indices().peekable(),
            src,
            next: None,
            current_pos: 0,
        }
    }

    pub fn next(&mut self) -> Result<Token<'src>> {
        let token = match self.next.take() {
            Some(val) => val,
            None => self.next_token(),
        };

        if let Ok(token) = token.as_ref() {
            self.current_pos = token.1;
        }

        token
    }

    pub fn peek(&mut self) -> &Result<Token<'src>> {
        match self.next {
            Some(ref val) => val,
            None => {
                self.next = Some(self.next());
                self.peek()
            }
        }
    }

    fn next_token(&mut self) -> Result<Token<'src>> {
        let (index, c) = match self.chars.next() {
            None => return self.eof(),
            Some(c) => c,
        };

        let next = self.chars.peek().map(|(_, c)| *c);

        match (c, next) {
            // -----------------------------------------------------------------------------
            //     - Double tokens -
            // -----------------------------------------------------------------------------
            ('/', Some('/')) => Ok(self.take_comment().to_token(index)),
            ('{', Some('{')) => {
                let _ = self.chars.next();
                Ok(Kind::LDoubleCurly.to_token(index))
            }
            ('}', Some('}')) => {
                let _ = self.chars.next();
                Ok(Kind::RDoubleCurly.to_token(index))
            }

            // -----------------------------------------------------------------------------
            //     - Single tokens -
            // -----------------------------------------------------------------------------
            ('[', _) => Ok(Kind::LBracket.to_token(index)),
            (']', _) => Ok(Kind::RBracket.to_token(index)),
            ('(', _) => Ok(Kind::LParen.to_token(index)),
            (')', _) => Ok(Kind::RParen.to_token(index)),
            (':', _) => Ok(Kind::Colon.to_token(index)),
            (',', _) => Ok(Kind::Comma.to_token(index)),
            ('|', _) => Ok(Kind::Pipe.to_token(index)),
            ('.', _) => Ok(Kind::Fullstop.to_token(index)),
            ('\n', _) => Ok(Kind::Newline.to_token(index)),

            // -----------------------------------------------------------------------------
            //     - Ident -
            // -----------------------------------------------------------------------------
            ('a'..='z' | 'A'..='Z' | '_', _) => Ok(self.take_ident(index).to_token(index)),

            // -----------------------------------------------------------------------------
            //     - Number -
            // -----------------------------------------------------------------------------
            ('0'..='9' | '-' | '+', _) => self.take_number(index),

            // -----------------------------------------------------------------------------
            //     - String -
            // -----------------------------------------------------------------------------
            ('"' | '\'', _) => self.take_string(c, index),

            // -----------------------------------------------------------------------------
            //     - Indents / Whitespace -
            // -----------------------------------------------------------------------------
            _ if c.is_whitespace() && c != '\n' => Ok(self.take_whitespace().to_token(index)),

            // -----------------------------------------------------------------------------
            //     - Hex values -
            // -----------------------------------------------------------------------------
            ('#', Some('0'..='9' | 'a'..='f' | 'A'..='F')) => self.take_hex_values(index),

            // -----------------------------------------------------------------------------
            //     - Done -
            // -----------------------------------------------------------------------------
            _ => self.eof(),
        }
    }

    fn eof(&self) -> Result<Token<'src>> {
        Ok(Token(Kind::EOF, self.src.len()))
    }

    fn take_string(&mut self, start_char: char, start_index: usize) -> Result<Token<'src>> {
        loop {
            let n = self.chars.next();
            match n {
                Some((end, nc)) if nc == start_char => {
                    break Ok(Kind::String(&self.src[start_index + 1..end]).to_token(start_index))
                }
                Some((_, '\\')) => {
                    // escaping string terminator
                    if let Some((_, next)) = self.chars.peek() {
                        if *next == start_char {
                            self.chars.next();
                        }
                    }
                }
                None => {
                    break Err(Error::unterminated_string(
                        start_index..self.src.len(),
                        self.src,
                    ))
                }
                _ => {} // consume chars
            }
        }
    }

    fn take_number(&mut self, index: usize) -> Result<Token<'src>> {
        let mut end = index;
        let mut parse_float = &self.src[index..=index] == ".";

        let signed = &self.src[index..=index] == "-"
            || self.chars.peek().map(|(_, c)| *c == '-').unwrap_or(false);

        while let Some((e, c @ ('0'..='9' | '-' | '.' | '+'))) = self.chars.peek() {
            if *c == '.' {
                parse_float = true;
            }
            end = *e;
            self.chars.next();
        }

        let input = &self.src[index..=end];
        let kind = match parse_float {
            true => match input.parse::<f64>() {
                Ok(num) => Ok(Kind::Number(Number::Float(num))),
                Err(_) => Err(Error::invalid_number(index..end + 1, self.src)),
            },
            false => match signed {
                true => match input.parse::<i64>() {
                    Ok(num) => Ok(Kind::Number(Number::Signed(num))),
                    Err(_) => Err(Error::invalid_number(index..end + 1, self.src)),
                },
                false => match input.parse::<u64>() {
                    Ok(num) => Ok(Kind::Number(Number::Unsigned(num))),
                    Err(_) => Err(Error::invalid_number(index..end + 1, self.src)),
                },
            },
        }?;

        Ok(Token(kind, index))
    }

    fn take_ident(&mut self, index: usize) -> Kind<'src> {
        let mut end = index;
        while let Some((e, 'a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9')) = self.chars.peek() {
            end = *e;
            self.chars.next();
        }
        let s = &self.src[index..=end];
        match s {
            "for" => Kind::For,
            "in" => Kind::In,
            "if" => Kind::If,
            "else" => Kind::Else,
            "view" => Kind::View,
            s => Kind::Ident(s),
        }
    }

    fn take_comment(&mut self) -> Kind<'src> {
        loop {
            match self.chars.peek() {
                Some((_, c)) if *c == '\n' => break,
                Some(_) => {
                    let _ = self.chars.next();
                    continue;
                }
                None => break,
            }
        }
        Kind::Comment
    }

    fn take_whitespace(&mut self) -> Kind<'src> {
        let mut count = 1;

        loop {
            match self.chars.peek() {
                Some((_, next)) if next.is_whitespace() && *next != '\n' => {
                    count += 1;
                    self.chars.next();
                }
                Some(_) | None => break,
            }
        }

        match self.chars.peek() {
            Some((_, '/')) => self.take_comment(),
            _ => Kind::Indent(count),
        }
    }

    fn take_hex_values(&mut self, index: usize) -> Result<Token<'src>> {
        let index = index + 1; // consume #
        const SHORT: usize = 3;
        const LONG: usize = 6;

        let mut end = index;
        while let Some((_, '0'..='9' | 'a'..='f' | 'A'..='F')) = self.chars.peek() {
            let _ = self.chars.next();
            end += 1;
        }

        let hex = &self.src[index..end];
        let len = hex.len();
        if len != 3 && len != 6 {
            return Err(Error::invalid_hex_value(index..end, self.src));
        }

        let kind = match len {
            SHORT => {
                let r = u8::from_str_radix(&hex[0..1], 16).expect("already parsed");
                let r = r << 4 | r;
                let g = u8::from_str_radix(&hex[1..2], 16).expect("already parsed");
                let g = g << 4 | g;
                let b = u8::from_str_radix(&hex[2..3], 16).expect("already parsed");
                let b = b << 4 | b;
                Kind::Hex(r, g, b)
            }
            LONG => {
                let r = u8::from_str_radix(&hex[0..2], 16).expect("already parsed");
                let g = u8::from_str_radix(&hex[2..4], 16).expect("already parsed");
                let b = u8::from_str_radix(&hex[4..6], 16).expect("already parsed");
                Kind::Hex(r, g, b)
            }
            _ => unreachable!(),
        };

        Ok(Token(kind, index))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::ErrorKind;

    fn token_kind(input: &str) -> Kind<'_> {
        let actual = Lexer::new(input).next().unwrap().0;
        actual
    }

    #[test]
    fn comment() {
        let actual = token_kind("// hello world");
        let expected = Kind::Comment;
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_char_token() {
        let inputs = [
            ("[", Kind::LBracket),
            ("]", Kind::RBracket),
            (":", Kind::Colon),
            (",", Kind::Comma),
            ("|", Kind::Pipe),
            ("\n", Kind::Newline),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn double_char_token() {
        let inputs = [
            ("//", Kind::Comment),
            ("{{", Kind::LDoubleCurly),
            ("}}", Kind::RDoubleCurly),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn ident() {
        let inputs = ["valid", "valid", "_valid", "_valid-_"];

        for input in inputs {
            let actual = token_kind(input);
            let expected = Kind::Ident(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn unsigned_ints() {
        let inputs = [("1", 1), ("0001", 1), ("100", 100)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = Kind::Number(Number::Unsigned(number));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn signed_ints() {
        let inputs = [("-1", -1), ("-0001", -1), ("-100", -100)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = Kind::Number(Number::Signed(number));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn floats() {
        let inputs = [
            ("0.1", 0.1f64),
            ("-.1", -0.1),
            ("1.", 1.0),
            ("100.5", 100.5),
        ];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = Kind::Number(Number::Float(number));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn strings() {
        let inputs = [
            ("'single quote string'", "single quote string"),
            (
                "'single quote\n string with newline char'",
                "single quote\n string with newline char",
            ),
            ("\"double quote string\"", "double quote string"),
            ("\"double 'single inside'\"", "double 'single inside'"),
            ("'single \"double inside\"'", "single \"double inside\""),
            (r#""escape \"double\"""#, r#"escape \"double\""#),
        ];

        for (input, s) in inputs {
            let actual = token_kind(input);
            let expected = Kind::String(s);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn consume_whitespace() {
        let input = "   ";
        let actual = token_kind(input);
        let expected = Kind::Indent(3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn hex() {
        let inputs = [
            ("#000", Kind::Hex(0, 0, 0)),
            ("#000000", Kind::Hex(0, 0, 0)),
            ("#FFF", Kind::Hex(255, 255, 255)),
            ("#FFFFFF", Kind::Hex(255, 255, 255)),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn view() {
        let input = "view";
        assert_eq!(Kind::View, token_kind(input));
    }

    #[test]
    fn invalid_hex() {
        let inputs = ["#00", "#0000", "#1234567", "#FFX", "#F-A"];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::InvalidHexValue);
        }
    }

    #[test]
    fn invalid_floats() {
        let inputs = ["0.0.1"];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::InvalidNumber);
        }
    }

    #[test]
    fn invalid_number() {
        let inputs = ["+-2"];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::InvalidNumber);
        }
    }

    #[test]
    fn unterminated_string() {
        let inputs = ["'unterminated string", "\'unterminated string", "'", "\""];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::UnterminatedString);
        }
    }
}
