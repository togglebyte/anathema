use std::fmt::{self, Display, Formatter};
use std::iter::Peekable;
use std::str::CharIndices;

use super::error::{Error, Result};
use widgets::Number;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum TokenKind<'src> {
    Colon,
    Comma,
    Comment,
    LDoubleCurly,
    RDoubleCurly,
    Hex(u8, u8, u8),
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
    Whitespace(usize),
}

impl<'src> TokenKind<'src> {
    fn to_token(self, pos: usize) -> Token<'src> {
        Token(self, Meta { pos })
    }
}

impl<'src> Display for TokenKind<'src> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug)]
pub struct Meta {
    pub(crate) pos: usize,
}

#[derive(Debug)]
pub struct Token<'src>(pub(crate) TokenKind<'src>, pub(crate) Meta);

pub struct Lexer<'src> {
    pub(crate) src: &'src str,
    chars: Peekable<CharIndices<'src>>,
}

impl<'src> Lexer<'src> {
    pub fn new(src: &'src str) -> Self {
        Self { chars: src.char_indices().peekable(), src }
    }

    fn next_token(&mut self) -> Option<Result<Token<'src>>> {
        let (index, c) = self.chars.next()?;
        let next = self.chars.peek().map(|(_, c)| *c);

        match (c, next) {
            // -----------------------------------------------------------------------------
            //     - Double tokens -
            // -----------------------------------------------------------------------------
            ('/', Some('/')) => Some(Ok(self.take_comment().to_token(index))),
            ('{', Some('{')) => {
                let _ = self.chars.next();
                Some(Ok(TokenKind::LDoubleCurly.to_token(index)))
            }
            ('}', Some('}')) => {
                let _ = self.chars.next();
                Some(Ok(TokenKind::RDoubleCurly.to_token(index)))
            }

            // -----------------------------------------------------------------------------
            //     - Single tokens -
            // -----------------------------------------------------------------------------
            ('[', _) => Some(Ok(TokenKind::LBracket.to_token(index))),
            (']', _) => Some(Ok(TokenKind::RBracket.to_token(index))),
            ('(', _) => Some(Ok(TokenKind::LParen.to_token(index))),
            (')', _) => Some(Ok(TokenKind::RParen.to_token(index))),
            (':', _) => Some(Ok(TokenKind::Colon.to_token(index))),
            (',', _) => Some(Ok(TokenKind::Comma.to_token(index))),
            ('|', _) => Some(Ok(TokenKind::Pipe.to_token(index))),
            ('.', _) => Some(Ok(TokenKind::Fullstop.to_token(index))),
            ('\n', _) => Some(Ok(TokenKind::Newline.to_token(index))),

            // -----------------------------------------------------------------------------
            //     - Ident -
            // -----------------------------------------------------------------------------
            ('a'..='z' | 'A'..='Z' | '_', _) => Some(Ok(TokenKind::Ident(self.take_ident(index)).to_token(index))),

            // -----------------------------------------------------------------------------
            //     - Number -
            // -----------------------------------------------------------------------------
            ('0'..='9' | '-' | '+', _) => Some(self.take_number(index)),

            // -----------------------------------------------------------------------------
            //     - String -
            // -----------------------------------------------------------------------------
            ('"' | '\'', _) => Some(self.take_string(c, index)),

            // -----------------------------------------------------------------------------
            //     - Indents / Whitespace -
            // -----------------------------------------------------------------------------
            _ if c.is_whitespace() && c != '\n' => Some(Ok(self.take_whitespace().to_token(index))),

            // -----------------------------------------------------------------------------
            //     - Hex values -
            // -----------------------------------------------------------------------------
            ('#', Some('0'..='9' | 'a'..='f' | 'A'..='F')) => Some(self.take_hex_values(index)),

            // -----------------------------------------------------------------------------
            //     - Done -
            // -----------------------------------------------------------------------------
            _ => None,
        }
    }

    fn take_string(&mut self, start_char: char, start_index: usize) -> Result<Token<'src>> {
        loop {
            let n = self.chars.next();
            match n {
                Some((end, nc)) if nc == start_char => {
                    break Ok(TokenKind::String(&self.src[start_index + 1..end]).to_token(start_index))
                }
                Some((_, '\\')) => {
                    // escaping string terminator
                    if let Some((_, next)) = self.chars.peek() {
                        if *next == start_char {
                            self.chars.next();
                        }
                    }
                }
                None => break Err(Error::unterminated_string(start_index..self.src.len(), self.src)),
                _ => {} // consume chars
            }
        }
    }

    fn take_number(&mut self, index: usize) -> Result<Token<'src>> {
        let mut end = index;
        let mut parse_float = &self.src[index..=index] == ".";

        let signed = &self.src[index..=index] == "-" || self.chars.peek().map(|(_, c)| *c == '-').unwrap_or(false);

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
                Ok(num) => Ok(TokenKind::Number(Number::Float(num))),
                Err(_) => Err(Error::invalid_number(index..end + 1, self.src)),
            },
            false => match signed {
                true => match input.parse::<i64>() {
                    Ok(num) => Ok(TokenKind::Number(Number::Signed(num))),
                    Err(_) => Err(Error::invalid_number(index..end + 1, self.src)),
                },
                false => match input.parse::<u64>() {
                    Ok(num) => Ok(TokenKind::Number(Number::Unsigned(num))),
                    Err(_) => Err(Error::invalid_number(index..end + 1, self.src)),
                },
            },
        }?;

        Ok(Token(kind, Meta { pos: index }))
    }

    fn take_ident(&mut self, index: usize) -> &'src str {
        let mut end = index;
        while let Some((e, 'a'..='z' | 'A'..='Z' | '-' | '_')) = self.chars.peek() {
            end = *e;
            self.chars.next();
        }
        &self.src[index..=end]
    }

    fn take_comment(&mut self) -> TokenKind<'src> {
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
        TokenKind::Comment
    }

    fn take_whitespace(&mut self) -> TokenKind<'src> {
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
            _ => TokenKind::Whitespace(count),
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
                TokenKind::Hex(r, g, b)
            }
            LONG => {
                let r = u8::from_str_radix(&hex[0..2], 16).expect("already parsed");
                let g = u8::from_str_radix(&hex[2..4], 16).expect("already parsed");
                let b = u8::from_str_radix(&hex[4..6], 16).expect("already parsed");
                TokenKind::Hex(r, g, b)
            }
            _ => unreachable!(),
        };

        Ok(Token(kind, Meta { pos: index }))
    }
}

impl<'src> Iterator for Lexer<'src> {
    type Item = Result<Token<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parser::error::ErrorKind;

    fn token_kind(input: &str) -> TokenKind<'_> {
        let actual = Lexer::new(input).next().unwrap().unwrap().0;
        actual
    }

    #[test]
    fn comment() {
        let actual = token_kind("// hello world");
        let expected = TokenKind::Comment;
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_char_token() {
        let inputs = [
            ("[", TokenKind::LBracket),
            ("]", TokenKind::RBracket),
            (":", TokenKind::Colon),
            (",", TokenKind::Comma),
            ("|", TokenKind::Pipe),
            ("\n", TokenKind::Newline),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn double_char_token() {
        let inputs = [("//", TokenKind::Comment), ("{{", TokenKind::LDoubleCurly), ("}}", TokenKind::RDoubleCurly)];

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
            let expected = TokenKind::Ident(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn unsigned_ints() {
        let inputs = [("1", 1), ("0001", 1), ("100", 100)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = TokenKind::Number(Number::Unsigned(number));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn signed_ints() {
        let inputs = [("-1", -1), ("-0001", -1), ("-100", -100)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = TokenKind::Number(Number::Signed(number));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn floats() {
        let inputs = [("0.1", 0.1f64), ("-.1", -0.1), ("1.", 1.0), ("100.5", 100.5)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = TokenKind::Number(Number::Float(number));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn strings() {
        let inputs = [
            ("'single quote string'", "single quote string"),
            ("\"double quote string\"", "double quote string"),
            ("\"double 'single inside'\"", "double 'single inside'"),
            ("'single \"double inside\"'", "single \"double inside\""),
            (r#""escape \"double\"""#, r#"escape \"double\""#),
        ];

        for (input, s) in inputs {
            let actual = token_kind(input);
            let expected = TokenKind::String(s);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn consume_whitespace() {
        let input = "   ";
        let actual = token_kind(input);
        let expected = TokenKind::Whitespace(3);
        assert_eq!(expected, actual);
    }

    #[test]
    fn hex() {
        let inputs = [
            ("#000", TokenKind::Hex(0, 0, 0)),
            ("#000000", TokenKind::Hex(0, 0, 0)),
            ("#FFF", TokenKind::Hex(255, 255, 255)),
            ("#FFFFFF", TokenKind::Hex(255, 255, 255)),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn invalid_hex() {
        let inputs = ["#00", "#0000", "#1234567", "#FFX", "#F-A"];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::InvalidHexValue);
        }
    }

    #[test]
    fn invalid_floats() {
        let inputs = ["0.0.1"];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::InvalidNumber);
        }
    }

    #[test]
    fn invalid_number() {
        let inputs = ["+-2"];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::InvalidNumber);
        }
    }

    #[test]
    fn unterminated_string() {
        let inputs = ["'unterminated string", "\'unterminated string", "'", "\""];

        for input in inputs {
            let actual = Lexer::new(input).next_token().unwrap().unwrap_err().kind;
            assert_eq!(actual, ErrorKind::UnterminatedString);
        }
    }
}
