use std::iter::Peekable;
use std::str::CharIndices;

use anathema_store::storage::strings::Strings;

use crate::error::{ParseError, ParseErrorKind, Result};
use crate::token::{Kind, Operator, Token, Value};

impl<'src, 'consts> Iterator for Lexer<'src, 'consts> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(Token(Kind::Eof, _)) => None,
            val => Some(val),
        }
    }
}

pub struct Lexer<'src, 'strings> {
    pub(super) src: &'src str,
    pub(crate) strings: &'strings mut Strings,
    chars: Peekable<CharIndices<'src>>,
}

impl<'src, 'strings> Lexer<'src, 'strings> {
    pub(crate) fn new(src: &'src str, strings: &'strings mut Strings) -> Self {
        Self {
            chars: src.char_indices().peekable(),
            strings,
            src,
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        let (index, c) = match self.chars.next() {
            None => return self.eof(),
            Some(c) => c,
        };

        let next = self.chars.peek().map(|(_, c)| *c);

        match (c, next) {
            // -----------------------------------------------------------------------------
            //     - Double tokens -
            // -----------------------------------------------------------------------------
            ('/', Some('/')) => {
                self.chars.next(); // consume the second slash
                loop {
                    if let Some((_, '\n')) | None = self.chars.peek() {
                        break;
                    }
                    self.chars.next();
                }
                self.next_token()
            }
            ('&', Some('&')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::And).to_token(index))
            }
            ('|', Some('|')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::Or).to_token(index))
            }
            ('=', Some('=')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::EqualEqual).to_token(index))
            }
            ('!', Some('=')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::NotEqual).to_token(index))
            }
            ('>', Some('=')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::GreaterThanOrEqual).to_token(index))
            }
            ('<', Some('=')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::LessThanOrEqual).to_token(index))
            }
            ('-', Some('>')) => {
                let _ = self.chars.next();
                Ok(Kind::Op(Operator::Association).to_token(index))
            }

            // -----------------------------------------------------------------------------
            //     - Single tokens -
            // -----------------------------------------------------------------------------
            ('(', _) => Ok(Kind::Op(Operator::LParen).to_token(index)),
            (')', _) => Ok(Kind::Op(Operator::RParen).to_token(index)),
            ('[', _) => Ok(Kind::Op(Operator::LBracket).to_token(index)),
            (']', _) => Ok(Kind::Op(Operator::RBracket).to_token(index)),
            ('{', _) => Ok(Kind::Op(Operator::LCurly).to_token(index)),
            ('}', _) => Ok(Kind::Op(Operator::RCurly).to_token(index)),
            (':', _) => Ok(Kind::Op(Operator::Colon).to_token(index)),
            (',', _) => Ok(Kind::Op(Operator::Comma).to_token(index)),
            ('.', _) => Ok(Kind::Op(Operator::Dot).to_token(index)),
            ('!', _) => Ok(Kind::Op(Operator::Not).to_token(index)),
            ('+', _) => Ok(Kind::Op(Operator::Plus).to_token(index)),
            ('-', _) => Ok(Kind::Op(Operator::Minus).to_token(index)),
            ('*', _) => Ok(Kind::Op(Operator::Mul).to_token(index)),
            ('/', _) => Ok(Kind::Op(Operator::Div).to_token(index)),
            ('%', _) => Ok(Kind::Op(Operator::Mod).to_token(index)),
            ('>', _) => Ok(Kind::Op(Operator::GreaterThan).to_token(index)),
            ('<', _) => Ok(Kind::Op(Operator::LessThan).to_token(index)),
            ('=', _) => Ok(Kind::Equal.to_token(index)),
            ('\n', _) => Ok(Kind::Newline.to_token(index)),
            ('@', _) => Ok(Kind::Component.to_token(index)),
            ('$', _) => Ok(Kind::ComponentSlot.to_token(index)),

            // -----------------------------------------------------------------------------
            //     - Ident -
            // -----------------------------------------------------------------------------
            ('a'..='z' | 'A'..='Z' | '_', _) => Ok(self.take_ident_or_keyword(index).to_token(index)),

            // -----------------------------------------------------------------------------
            //     - Number -
            // -----------------------------------------------------------------------------
            ('0'..='9', _) => self.take_number(index),

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

    fn eof(&self) -> Result<Token> {
        Ok(Token(Kind::Eof, self.src.len()))
    }

    fn take_string(&mut self, start_char: char, start_index: usize) -> Result<Token> {
        loop {
            let n = self.chars.next();
            match n {
                Some((end, nc)) if nc == start_char => {
                    let string = self.strings.push(self.src[start_index + 1..end].to_string());
                    break Ok(Kind::Value(Value::String(string)).to_token(start_index));
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
                    break Err(ParseError::new(
                        start_index..self.src.len(),
                        self.src,
                        ParseErrorKind::UnterminatedString,
                    )
                    .into())
                }
                _ => {} // consume chars
            }
        }
    }

    fn take_number(&mut self, index: usize) -> Result<Token> {
        let mut end = index;
        let mut parse_float = &self.src[index..=index] == ".";

        let _signed = &self.src[index..=index] == "-" || self.chars.peek().map(|(_, c)| *c == '-').unwrap_or(false);

        while let Some((e, c @ ('0'..='9' | '.'))) = self.chars.peek() {
            if *c == '.' {
                parse_float = true;
            }
            end = *e;
            self.chars.next();
        }

        let input = &self.src[index..=end];

        let kind = match parse_float {
            true => match input.parse::<f64>() {
                Ok(num) => Ok(Kind::Value(num.into())),
                Err(_) => Err(ParseError::new(index..end + 1, self.src, ParseErrorKind::InvalidNumber)),
            },
            false => match input.parse::<i64>() {
                Ok(num) => Ok(Kind::Value(num.into())),
                Err(_) => Err(ParseError::new(index..end + 1, self.src, ParseErrorKind::InvalidNumber)),
            },
        }?;

        Ok(Token(kind, index))
    }

    fn take_ident_or_keyword(&mut self, index: usize) -> Kind {
        let mut end = index;
        while let Some((e, 'a'..='z' | 'A'..='Z' | '_' | '|' | '0'..='9')) = self.chars.peek() {
            end = *e;
            self.chars.next();
        }

        let s = &self.src[index..=end];
        match s {
            "for" => Kind::For,
            "in" => Kind::In,
            "if" => Kind::If,
            "else" => Kind::Else,
            "true" => Kind::Value(true.into()),
            "false" => Kind::Value(false.into()),
            "let" => Kind::Decl,
            s => {
                let string_id = self.strings.push(s.to_string());
                Kind::Value(Value::Ident(string_id))
            }
        }
    }

    fn take_whitespace(&mut self) -> Kind {
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

        Kind::Indent(count)
    }

    fn take_hex_values(&mut self, index: usize) -> Result<Token> {
        const SHORT: usize = 3;
        const LONG: usize = 6;

        let index = index + 1; // consume #
        let mut end = index;

        while let Some((_, '0'..='9' | 'a'..='f' | 'A'..='F')) = self.chars.peek() {
            let _ = self.chars.next();
            end += 1;
        }

        let hex = &self.src[index..end];
        let len = hex.len();
        // Make sure that it's either three or six characters,
        // otherwise it's an invalid hex
        if len != 3 && len != 6 {
            return Err(ParseError::new(index..end, self.src, ParseErrorKind::InvalidHexValue).into());
        }

        let kind = match len {
            SHORT => {
                let r = u8::from_str_radix(&hex[0..1], 16).expect("already parsed");
                let r = r << 4 | r;
                let g = u8::from_str_radix(&hex[1..2], 16).expect("already parsed");
                let g = g << 4 | g;
                let b = u8::from_str_radix(&hex[2..3], 16).expect("already parsed");
                let b = b << 4 | b;
                Kind::Value((r, g, b).into())
            }
            LONG => {
                let r = u8::from_str_radix(&hex[0..2], 16).expect("already parsed");
                let g = u8::from_str_radix(&hex[2..4], 16).expect("already parsed");
                let b = u8::from_str_radix(&hex[4..6], 16).expect("already parsed");
                Kind::Value((r, g, b).into())
            }
            _ => unreachable!(),
        };

        Ok(Token(kind, index))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::ParseErrorKind;

    fn token_kind(input: &str) -> Kind {
        let mut strings = Strings::empty();
        let kind = Lexer::new(input, &mut strings).next().unwrap().unwrap().0;
        kind
    }

    fn operator(input: &str) -> Operator {
        let kind = token_kind(input);
        match kind {
            Kind::Op(op) => op,
            _ => panic!("token is not an operator"),
        }
    }

    fn error_kind(input: &str) -> ParseErrorKind {
        let mut strings = Strings::empty();
        let mut lexer = Lexer::new(input, &mut strings);
        match lexer.next().unwrap().unwrap_err() {
            crate::error::Error::ParseError(err) => err.kind,
            crate::error::Error::CircularDependency
            | crate::error::Error::MissingComponent(_)
            | crate::error::Error::EmptyTemplate
            | crate::error::Error::EmptyBody
            | crate::error::Error::Io(_) => panic!("invalid error"),
        }
    }

    #[test]
    fn comment() {
        let mut strings = Strings::empty();
        let input = "// hello world";
        let mut lexer = Lexer::new(input, &mut strings);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn comment_retain_newline() {
        let input = "// hello world\n";
        let actual = token_kind(input);
        let expected = Kind::Newline;
        assert_eq!(expected, actual);
    }

    #[test]
    fn single_char_token() {
        let inputs = [
            ("[", Kind::Op(Operator::LBracket)),
            ("]", Kind::Op(Operator::RBracket)),
            (":", Kind::Op(Operator::Colon)),
            (",", Kind::Op(Operator::Comma)),
            ("\n", Kind::Newline),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn double_char_token() {
        let inputs = [("<=", Operator::LessThanOrEqual), ("&&", Operator::And)];

        for (input, expected) in inputs {
            let actual = operator(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn ident() {
        let inputs = ["valid", "valid", "_valid", "_valid-_"];

        for input in inputs {
            let actual = token_kind(input);
            let expected = Kind::Value(Value::Ident(0.into()));
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn unsigned_ints() {
        let inputs = [("1", 1), ("0001", 1), ("100", 100)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = Kind::Value(number.into());
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn floats() {
        let inputs = [("1.0", 1f64), ("0.555", 0.555f64)];

        for (input, number) in inputs {
            let actual = token_kind(input);
            let expected = Kind::Value(number.into());
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
            ("''", ""), // empty string
        ];

        let mut strings = Strings::empty();

        for (input, expected) in inputs {
            let Kind::Value(Value::String(string_id)) = Lexer::new(input, &mut strings).next().unwrap().unwrap().0
            else {
                panic!("invalid token")
            };
            let actual = strings.get_unchecked(string_id);
            assert_eq!(actual, expected);
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
    fn color() {
        let inputs = [
            ("#000", Kind::Value((0, 0, 0).into())),
            ("#000000", Kind::Value((0, 0, 0).into())),
            ("#FFF", Kind::Value((255, 255, 255).into())),
            ("#FFFFFF", Kind::Value((255, 255, 255).into())),
        ];

        for (input, expected) in inputs {
            let actual = token_kind(input);
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn component() {
        let input = "@component";
        assert_eq!(Kind::Component, token_kind(input));
    }

    #[test]
    fn component_slot() {
        let input = "$slot";
        assert_eq!(Kind::ComponentSlot, token_kind(input));
    }

    #[test]
    fn invalid_hex() {
        let inputs = ["#00", "#0000", "#1234567", "#FFX", "#F-A"];

        for input in inputs {
            let actual = error_kind(input);
            assert_eq!(actual, ParseErrorKind::InvalidHexValue);
        }
    }

    #[test]
    fn unterminated_string() {
        let inputs = ["'unterminated string", "\'unterminated string", "'", "\""];

        for input in inputs {
            let actual = error_kind(input);
            assert_eq!(actual, ParseErrorKind::UnterminatedString);
        }
    }

    #[test]
    fn lex_bool() {
        let b = token_kind("false");
        assert_eq!(b, Kind::Value(false.into()));

        let b = token_kind("true");
        assert_eq!(b, Kind::Value(true.into()));
    }

    #[test]
    fn declaration() {
        let decl = token_kind("let");
        assert_eq!(decl, Kind::Decl);
    }

    #[test]
    fn association() {
        let decl = token_kind("->");
        assert_eq!(decl, Kind::Op(Operator::Association));
    }
}
