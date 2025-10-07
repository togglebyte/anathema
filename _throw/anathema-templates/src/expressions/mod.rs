use std::collections::HashMap;
use std::fmt::Display;

use anathema_state::Hex;
use anathema_store::slab::Index;

use crate::primitives::Primitive;
use crate::variables::{ScopeId, VarId};

pub(crate) mod eval;
pub(crate) mod parser;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ExpressionId(u32);

impl From<ExpressionId> for Index {
    fn from(value: ExpressionId) -> Self {
        value.0.into()
    }
}

#[derive(Debug)]
pub struct Expressions {
    inner: Vec<(Expression, ScopeId)>,
}

impl Expressions {
    pub fn empty() -> Self {
        Self { inner: vec![] }
    }

    /// Get a reference to an expression
    ///
    /// # Panics
    ///
    /// Panics if the expression id is greater than the length of expressions.
    /// This should never happen as the ids are assigned to blueprints using them.
    ///
    /// Only time this could happen is if the expression id is created outside of the tempalte
    /// generation.
    pub fn get(&self, id: ExpressionId) -> &Expression {
        &self.inner[id.0 as usize].0
    }

    /// Insert an expression in the root scope
    pub fn insert_at_root(&mut self, expression: impl Into<Expression>) -> ExpressionId {
        self.insert(expression.into(), ScopeId::root().clone())
    }

    /// Insert an expression and return the id to the newly inserted expression
    pub fn insert(&mut self, expression: Expression, boundary: ScopeId) -> ExpressionId {
        let id = ExpressionId(self.inner.len() as u32);
        match self
            .inner
            .iter()
            .position(|(e, scope)| expression.eq(e) && boundary.eq(scope))
        {
            Some(id) => ExpressionId(id as u32),
            None => {
                self.inner.push((expression, boundary));
                id
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.inner.clear()
    }
}

impl std::ops::Index<ExpressionId> for Expressions {
    type Output = Expression;

    fn index(&self, index: ExpressionId) -> &Self::Output {
        &self.inner[index.0 as usize].0
    }
}

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
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BoolOp {
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
    Str(String),
    List(Vec<Self>),
    Map(HashMap<String, Self>),

    // Lookup a variable in the var table
    Variable(VarId),

    // This is specifically for the "value" of a node.
    TextSegments(Vec<Self>),

    // Unary
    Not(Box<Self>),
    Negative(Box<Self>),

    // Conditionals
    Equality(Box<Self>, Box<Self>, Equality),
    LogicalOp(Box<Self>, Box<Self>, LogicalOp),

    // Lookup
    Ident(String),
    Index(Box<Self>, Box<Self>),

    // Operations
    Op(Box<Self>, Box<Self>, Op),

    // Either
    Either(Box<Self>, Box<Self>),

    // Range
    Range(Box<Self>, Box<Self>),

    // Function call
    Call { fun: Box<Self>, args: Vec<Self> },
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

impl<T: Into<Expression>> From<HashMap<String, T>> for Expression {
    fn from(map: HashMap<String, T>) -> Self {
        let map = map.into_iter().map(|(key, val)| (key, val.into())).collect();
        Self::Map(map)
    }
}

impl<T: Into<Expression>> From<Vec<T>> for Expression {
    fn from(list: Vec<T>) -> Self {
        let inner = list.into_iter().map(|i| i.into()).collect();
        Self::List(inner)
    }
}

impl<const N: usize, T: Into<Expression>> From<[T; N]> for Expression {
    fn from(list: [T; N]) -> Self {
        let inner = list.map(|i| i.into());
        Self::List(inner.to_vec())
    }
}

impl From<String> for Expression {
    fn from(value: String) -> Self {
        Self::Str(value)
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
            Self::Index(lhs, idx) => write!(f, "<{lhs}>[{idx}]"),
            Self::Variable(var_id) => write!(f, "<var {var_id:?}>"),
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
            Self::Either(lhs, rhs) => write!(f, "{lhs} ? {rhs}"),
            Self::Range(lhs, rhs) => write!(f, "{lhs} .. {rhs}"),
            Self::List(list) => {
                write!(
                    f,
                    "[{}]",
                    list.iter().map(|val| val.to_string()).collect::<Vec<_>>().join(", ")
                )
            }
            Self::TextSegments(segments) => {
                write!(
                    f,
                    "{}",
                    segments.iter().map(|val| val.to_string()).collect::<Vec<_>>().join("")
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
                    Equality::Gt => ">",
                    Equality::Gte => ">=",
                    Equality::Lt => "<",
                    Equality::Lte => "<=",
                };
                write!(f, "{lhs} {equality} {rhs}")
            }
            Self::LogicalOp(lhs, rhs, op) => {
                let op = match op {
                    LogicalOp::And => "&&",
                    LogicalOp::Or => "||",
                };
                write!(f, "{lhs} {op} {rhs}")
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
//   - Either -
// -----------------------------------------------------------------------------
pub fn either(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Either(lhs, rhs).into()
}

pub fn range(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::Range(lhs, rhs).into()
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

pub fn chr(c: char) -> Box<Expression> {
    Expression::Primitive(c.into()).into()
}

pub fn hex(h: impl Into<Hex>) -> Box<Expression> {
    let h = h.into();
    Expression::Primitive(h.into()).into()
}

pub fn boolean(b: bool) -> Box<Expression> {
    Expression::Primitive(b.into()).into()
}

pub fn strlit(lit: &str) -> Box<Expression> {
    Expression::Str(lit.into()).into()
}

// -----------------------------------------------------------------------------
//   - List, map and text segment -
// -----------------------------------------------------------------------------
pub fn list<E: Into<Expression>>(input: impl IntoIterator<Item = E>) -> Box<Expression> {
    let vec = input.into_iter().map(|val| val.into()).collect::<Vec<_>>();
    Expression::List(vec).into()
}

pub fn text_segments<E: Into<Expression>>(input: impl IntoIterator<Item = E>) -> Box<Expression> {
    let vec = input.into_iter().map(|val| val.into()).collect::<Vec<_>>();
    Expression::TextSegments(vec).into()
}

pub fn map<E: Into<Expression>>(input: impl IntoIterator<Item = (&'static str, E)>) -> Box<Expression> {
    let input = input.into_iter().map(|(k, v)| (k.into(), v.into()));
    let hm: HashMap<String, Expression> = HashMap::from_iter(input);
    Expression::Map(hm).into()
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
    Expression::LogicalOp(lhs, rhs, LogicalOp::And).into()
}

pub fn or(lhs: Box<Expression>, rhs: Box<Expression>) -> Box<Expression> {
    Expression::LogicalOp(lhs, rhs, LogicalOp::Or).into()
}
