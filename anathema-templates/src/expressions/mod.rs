use std::collections::HashMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::primitives::Primitive;

pub(crate) mod eval;
pub(crate) mod parser;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Div,
    Mul,
    Mod,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Equality {
    Eq,
    NotEq,
    And,
    Or,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Value types
    Primitive(Primitive),
    Str(Rc<str>),
    List(Rc<[Self]>),
    Map(Rc<HashMap<Rc<str>, Self>>),

    // Unary
    Not(Box<Self>),
    Negative(Box<Self>),

    // Conditionals
    Equality(Box<Self>, Box<Self>, Equality),

    // Lookup
    Ident(Rc<str>),
    Index(Box<Self>, Box<Self>),

    // Operations
    Op(Box<Self>, Box<Self>, Op),

    // Function call
    Call { fun: Box<Self>, args: Box<[Self]> },
}

impl From<Box<Expression>> for Expression {
    fn from(value: Box<Expression>) -> Self {
        *value
    }
}

impl<T: Into<Primitive>> From<T> for Expression {
    fn from(value: T) -> Self {
        Self::Primitive(value.into())
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        Self::Str(value.into())
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(val) => write!(f, "{val}"),
            Self::Str(val) => write!(f, "{val}"),
            Self::Ident(s) => write!(f, "{s}"),
            Self::Index(lhs, idx) => write!(f, "{lhs}[{idx}]"),
            Self::Not(expr) => write!(f, "!{expr}"),
            Self::Negative(expr) => write!(f, "-{expr}"),
            Self::Op(lhs, rhs, op) => {
                let op = match op {
                    Op::Add => '+',
                    Op::Sub => '-',
                    Op::Div => '/',
                    Op::Mul => '*',
                    Op::Mod => '%',
                };
                write!(f, "{lhs} {op} {rhs}")
            }
            Self::List(list) => {
                write!(
                    f,
                    "[{}]",
                    list.iter().map(|val| val.to_string()).collect::<Vec<_>>().join(", ")
                )
            }
            Self::Map(map) => {
                write!(
                    f,
                    "{{{}}}",
                    map.iter()
                        .map(|(key, val)| format!("{key}: {val}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Self::Equality(lhs, rhs, equality) => {
                let equality = match equality {
                    Equality::Eq => "==",
                    Equality::NotEq => "!=",
                    Equality::And => "&&",
                    Equality::Or => "||",
                    Equality::Gt => ">",
                    Equality::Gte => ">=",
                    Equality::Lt => "<",
                    Equality::Lte => "<=",
                };
                write!(f, "{lhs} {equality} {rhs}")
            }
            Self::Call { fun, args } => {
                write!(
                    f,
                    "{fun}({})",
                    args.iter().map(|val| val.to_string()).collect::<Vec<_>>().join(", ")
                )
            }
        }
    }
}

// -----------------------------------------------------------------------------
//   - Paths -
// -----------------------------------------------------------------------------
pub fn ident(p: &str) -> Box<Expression> {
    Expression::Ident(p.into()).into()
}

pub fn index(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Index(lhs, rhs).into()
}

// -----------------------------------------------------------------------------
//   - Maths -
// -----------------------------------------------------------------------------
pub fn mul(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Op(lhs, rhs, Op::Mul).into()
}

pub fn div(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Op(lhs, rhs, Op::Div).into()
}

pub fn modulo(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Op(lhs, rhs, Op::Mod).into()
}

pub fn sub(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Op(lhs, rhs, Op::Sub).into()
}

pub fn add(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Op(lhs, rhs, Op::Add).into()
}

pub fn greater_than(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::Gt).into()
}

pub fn greater_than_equal(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::Gte).into()
}

pub fn less_than(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::Lt).into()
}

pub fn less_than_equal(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::Lte).into()
}

// -----------------------------------------------------------------------------
//   - Values -
// -----------------------------------------------------------------------------
pub fn num(int: i64) -> Box<Expression> {
    Expression::Primitive(int.into()).into()
}

pub fn float(float: f64) -> Box<Expression> {
    Expression::Primitive(float.into()).into()
}

pub fn boolean(b: bool) -> Box<Expression> {
    Expression::Primitive(b.into()).into()
}

pub fn strlit(lit: &str) -> Box<Expression> {
    Expression::Str(lit.into()).into()
}

// -----------------------------------------------------------------------------
//   - List and map -
// -----------------------------------------------------------------------------
pub fn list<E: Into<Expression>>(input: impl IntoIterator<Item = E>) -> Box<Expression> {
    let vec = input.into_iter().map(|val| val.into()).collect::<Vec<_>>();
    Expression::List(vec.into()).into()
}

pub fn map<E: Into<Expression>>(input: impl IntoIterator<Item = (&'static str, E)>) -> Box<Expression> {
    let input = input.into_iter().map(|(k, v)| (k.into(), v.into()));
    let hm: HashMap<Rc<str>, Expression> = HashMap::from_iter(input);
    Expression::Map(hm.into()).into()
}

// -----------------------------------------------------------------------------
//   - Op -
// -----------------------------------------------------------------------------
pub fn neg(expr: Box<Expression>) -> Box<Expression> {
    Expression::Negative(expr).into()
}

// -----------------------------------------------------------------------------
//   - Conditionals -
// -----------------------------------------------------------------------------
pub fn not(expr: Box<Expression>) -> Box<Expression> {
    Expression::Not(expr).into()
}

pub fn eq(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::Eq).into()
}

pub fn and(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::And).into()
}

pub fn or(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Equality(lhs, rhs, Equality::Or).into()
}
