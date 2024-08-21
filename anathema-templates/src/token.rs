use std::fmt::{self, Display, Formatter};

use anathema_store::storage::strings::StringId;

use crate::primitives::Primitive;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operator {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    PlusEqual,
    MinusEqual,
    MulEqual,
    DivEqual,
    ModEqual,
    EqualEqual,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Not,
    And,
    Or,
    Dot,
    Comma,
    Colon,
    Association,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::PlusEqual => write!(f, "+="),
            Self::MinusEqual => write!(f, "-="),
            Self::MulEqual => write!(f, "*="),
            Self::DivEqual => write!(f, "/="),
            Self::ModEqual => write!(f, "%="),
            Self::EqualEqual => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::Not => write!(f, "!"),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Dot => write!(f, "."),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::LCurly => write!(f, "{{"),
            Self::RCurly => write!(f, "}}"),
            Self::Association => write!(f, "->"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Value {
    Primitive(Primitive),
    String(StringId),
    Ident(StringId),
}

impl<T: Into<Primitive>> From<T> for Value {
    fn from(value: T) -> Self {
        Self::Primitive(value.into())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Primitive(val) => write!(f, "{val}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Ident(id) => write!(f, "{id}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Kind {
    Equal,
    For,
    In,
    If,
    Else,
    Component,
    ComponentSlot,
    Newline,
    Indent(usize),

    Value(Value),
    Op(Operator),

    Decl,

    Eof,
}

impl Kind {
    pub(crate) fn to_token(self, pos: usize) -> Token {
        Token(self, pos)
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equal => write!(f, "="),
            Self::For => write!(f, "<for>"),
            Self::In => write!(f, "<in>"),
            Self::If => write!(f, "<if>"),
            Self::Else => write!(f, "<else>"),
            Self::Component => write!(f, "<component>"),
            Self::ComponentSlot => write!(f, "<slot>"),
            Self::Newline => write!(f, "\\n"),
            Self::Indent(s) => write!(f, "<indent {s}>"),
            Self::Value(v) => write!(f, "<value {v}>"),
            Self::Op(o) => write!(f, "<op {o}>"),
            Self::Decl => write!(f, "let"),
            Self::Eof => write!(f, "<Eof>"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Token(pub(crate) Kind, pub(crate) usize);

#[derive(Debug)]
pub(crate) struct Tokens {
    inner: Vec<Token>,
    index: usize,
    eof: usize,
}

impl Tokens {
    pub fn new(inner: Vec<Token>, eof: usize) -> Self {
        Self { inner, index: 0, eof }
    }

    pub fn consume(&mut self) {
        let _ = self.next();
    }

    pub fn next(&mut self) -> Kind {
        match self.inner.get(self.index).copied() {
            Some(token) => {
                self.index += 1;
                token.0
            }
            None => Kind::Eof,
        }
    }

    pub fn next_no_indent(&mut self) -> Kind {
        loop {
            let token = self.next();

            if let Kind::Indent(_) = token {
                continue;
            }

            break token;
        }
    }

    pub fn consume_indent(&mut self) {
        loop {
            if matches!(self.inner.get(self.index), Some(Token(Kind::Indent(_), _))) {
                self.index += 1;
                continue;
            }
            break;
        }
    }

    pub fn consume_newlines(&mut self) {
        loop {
            if matches!(self.inner.get(self.index), Some(Token(Kind::Newline, _))) {
                self.index += 1;
                continue;
            }
            break;
        }
    }

    pub fn consume_all_whitespace(&mut self) {
        loop {
            if matches!(self.inner.get(self.index), Some(Token(Kind::Indent(_), _))) {
                self.index += 1;
                continue;
            }

            if matches!(self.inner.get(self.index), Some(Token(Kind::Newline, _))) {
                self.index += 1;
                continue;
            }

            break;
        }
    }

    pub fn peek(&self) -> Kind {
        self.inner
            .get(self.index)
            .copied()
            .unwrap_or(Token(Kind::Eof, self.eof))
            .0
    }

    pub fn previous(&self) -> Token {
        assert!(self.index != 0);
        self.inner
            .get(self.index - 1)
            .copied()
            .unwrap_or(Token(Kind::Eof, self.eof))
    }

    pub fn peek_skip_indent(&mut self) -> Kind {
        loop {
            let token = self.peek();

            if let Kind::Indent(_) = token {
                self.index += 1;
                continue;
            }

            break token;
        }
    }

    pub fn read_indent(&mut self) -> Option<usize> {
        match self.peek() {
            Kind::Indent(indent) => {
                self.consume();
                Some(indent)
            }
            _ => None,
        }
    }
}
