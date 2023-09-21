use std::borrow::Cow;
use std::fmt::{self, Display, Write};
use std::ops::{Add, Deref, Mul, Sub, Div, Rem};
use std::rc::Rc;
use std::str::FromStr;

use anathema_render::{Color, Size, Style};

use crate::hashmap::HashMap;
use crate::{NodeId, Path, State, ValueExpr};

#[derive(Debug)]
pub enum Collection {
    Rc(Rc<[ScopeValue]>),
    State { path: Path, len: usize },
    Empty,
}

impl Collection {
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Rc(col) => col.len(),
            Self::State { len, .. } => *len,
        }
    }

    /// Increase the length of a state collection.
    /// This is a manual step for state bound lists
    /// as we don't access the entire list, only
    /// one value at a time when needed.
    pub fn add(&mut self) {
        if let Collection::State { len, .. } = self {
            *len += 1;
        }
    }

    /// Decrease the length of a state collection.
    /// This is a manual step (see `Self::add`)
    pub fn remove(&mut self) {
        if let Collection::State { len, .. } = self {
            *len -= 1;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Num {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
}

impl Display for Num {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signed(n) => write!(f, "{n}"),
            Self::Unsigned(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
        }
    }
}

macro_rules! into_unsigned_num {
    ($t:ty) => {
        impl From<$t> for Num {
            fn from(n: $t) -> Self {
                Self::Unsigned(n as u64)
            }
        }
    };
}

macro_rules! into_signed_num {
    ($t:ty) => {
        impl From<$t> for Num {
            fn from(n: $t) -> Self {
                Self::Signed(n as i64)
            }
        }
    };
}

into_unsigned_num!(u8);
into_unsigned_num!(u16);
into_unsigned_num!(u32);
into_unsigned_num!(u64);
into_unsigned_num!(usize);

into_signed_num!(i8);
into_signed_num!(i16);
into_signed_num!(i32);
into_signed_num!(i64);
into_signed_num!(isize);

impl Mul for Num {
    type Output = Num;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs * rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs * rhs),
            _ => panic!(),
        }
    }
}

impl Add for Num {
    type Output = Num;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs + rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs + rhs),


            (Self::Signed(lhs), Self::Unsigned(rhs)) if lhs.is_negative() => {
                if lhs.abs() as u64 >= rhs {
                    Self::Signed(-((lhs.abs() as u64 - rhs) as i64))
                } else {
                    Self::Unsigned(rhs - lhs.abs() as u64)
                }
            }

            (Self::Unsigned(lhs), Self::Signed(rhs)) if rhs.is_negative() => {
                if rhs.abs() as u64 >= lhs {
                    Self::Signed(-((rhs.abs() as u64 - lhs) as i64))
                } else {
                    Self::Unsigned(lhs - rhs.abs() as u64)
                }
            }

            (Self::Signed(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs as u64 + rhs),
            (Self::Unsigned(lhs), Self::Signed(rhs)) => Self::Unsigned(rhs as u64 + lhs),
            _ => panic!(),
        }
    }
}

impl Sub for Num {
    type Output = Num;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs - rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs - rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs - rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs - rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            _ => panic!(),
        }
    }
}

impl Div for Num {
    type Output = Num;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs / rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs / rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs / rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs / rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            _ => panic!(),
        }
    }
}

impl Rem for Num {
    type Output = Num;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Signed(lhs), Self::Signed(rhs)) => Self::Signed(lhs % rhs),
            (Self::Unsigned(lhs), Self::Unsigned(rhs)) => Self::Unsigned(lhs % rhs),

            (Self::Signed(lhs), Self::Unsigned(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs % rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            (Self::Unsigned(lhs), Self::Signed(rhs)) => {
                let lhs = lhs as i128;
                let rhs = rhs as i128;
                let res = lhs % rhs;
                if res.is_negative() {
                    Self::Signed(res as i64)
                } else {
                    Self::Unsigned(res as u64)
                }
            }
            _ => panic!(),
        }
    }
}

// TODO: give this a better name.
// If we evaluate a ValueExpr it should not always return a "static" value, the name
// is confusing.
#[derive(Debug, Clone, PartialEq)]
pub enum StaticValue {
    Str(Rc<str>),
    Num(Num),
    Bool(bool),
    Color(Color),
}

impl TryFrom<StaticValue> for Color {
    type Error = ();

    fn try_from(value: StaticValue) -> Result<Self, Self::Error> {
        match value {
            StaticValue::Color(color) => Ok(color),
            _ => Err(()),
        }
    }
}

impl TryFrom<StaticValue> for usize {
    type Error = ();

    fn try_from(value: StaticValue) -> Result<Self, Self::Error> {
        match value {
            StaticValue::Num(Num::Unsigned(num)) => Ok(num as usize),
            _ => Err(())
        }
    }
}

impl Display for StaticValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{s}"),
            Self::Num(num) => write!(f, "{num}"),
            Self::Color(color) => write!(f, "{color:?}"),
            Self::Bool(b) => write!(f, "{b:?}"),
        }
    }
}

impl From<bool> for StaticValue {
    fn from(b: bool) -> StaticValue {
        StaticValue::Bool(b)
    }
}

impl From<String> for StaticValue {
    fn from(s: String) -> StaticValue {
        StaticValue::Str(s.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeValue {
    Static(StaticValue),
    Expr(ValueExpr),
    // List(Rc<[ScopeValue]>),
    // Dyn(Path),
    Invalid,
}

// impl<const N: usize> From<[ScopeValue; N]> for ScopeValue {
//     fn from(arr: [ScopeValue; N]) -> Self {
//         if N == 1 {
//             arr.into_iter()
//                 .next()
//                 .expect("this is always going to be an array with a size of one")
//         } else {
//             ScopeValue::List(Rc::new(arr))
//         }
//     }
// }

// TODO: add a testing flag for this
impl From<String> for ScopeValue {
    fn from(s: String) -> Self {
        Self::Static(StaticValue::Str(s.into()))
    }
}

#[derive(Debug)]
pub struct Scope<'a> {
    inner: Vec<HashMap<Path, Cow<'a, ScopeValue>>>,
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Scope<'_>>) -> Self {
        Self {
            inner: vec![HashMap::new()],
        }
    }

    pub fn scope(&mut self, path: Path, value: Cow<'a, ScopeValue>) {
        self.inner
            .last_mut()
            .map(|values| values.insert(path, value));
    }

    pub fn push(&mut self) {
        self.inner.push(HashMap::new())
    }

    pub fn pop(&mut self) {
        self.inner.pop();
    }

    // /// Scope a value for a collection.
    // /// TODO: Review if the whole cloning business here makes sense
    // pub fn scope_collection(&mut self, binding: Path, collection: &Collection, value_index: usize) {
    //     let value = match collection {
    //         Collection::Rc(list) => Cow::Owned(list[value_index].clone()),
    //         Collection::State { path, .. } => {
    //             let path = path.compose(value_index);
    //             Cow::Owned(ScopeValue::Dyn(path))
    //         }
    //         Collection::Empty => return,
    //     };

    //     self.scope(binding, value);
    // }

    pub fn lookup(&self, path: &Path) -> Option<&ScopeValue> {
        self.inner
            .iter()
            .rev()
            .filter_map(|values| values.get(path).map(Deref::deref))
            .next()
    }

    // pub fn lookup_list(&self, path: &Path) -> Option<Rc<[ScopeValue]>> {
    //     self.lookup(path).and_then(|value| match value {
    //         ScopeValue::List(list) => Some(list.clone()),
    //         _ => None,
    //     })
    // }
}

pub struct Context<'a, 'val, T> {
    pub state: &'a mut T, //dyn State,
    pub scope: &'a mut Scope<'val>,
}

impl<'a, 'val, T: State> Context<'a, 'val, T> {
    pub fn new(state: &'a mut dyn State, scope: &'a mut Scope<'val>) -> Self {
        Self { state, scope }
    }

    /// Resolve a value based on paths.
    pub fn resolve(&self, value: &ScopeValue) -> ScopeValue {
        // TODO toodles
        panic!()
        // match value {
        //     ScopeValue::Static(_) => value.clone(),
        //     ScopeValue::Dyn(path) => match self.scope.lookup(path) {
        //         Some(lark @ ScopeValue::Dyn(p)) => self.resolve(lark),
        //         Some(_) => value.clone(),
        //         None => ScopeValue::Dyn(path.clone()),
        //     },
        //     ScopeValue::List(list) => {
        //         let values = list.iter().map(|v| self.resolve(v)).collect();
        //         ScopeValue::List(values)
        //     }
        // }
    }

    pub fn get_scope(&mut self, path: &Path, node_id: Option<&NodeId>) -> Option<StaticValue> {
        match self.scope.lookup(&path).cloned() {
            Some(ScopeValue::Static(val)) => Some(val),
            Some(val) => match val {
                ScopeValue::Static(val) => Some(val),
                ScopeValue::Expr(expr) => expr.eval(self, node_id),
                ScopeValue::Invalid => panic!("lol"),
            }
            None => self
                .state
                .get(&path, node_id.into())
                .map(|val| val.into_owned())
        }
    }

    /// Try to find the value in the current scope,
    /// if there is no value fallback to look for the value in the state.
    /// This will recursively lookup dynamic values
    pub fn get<U>(&self, path: &Path, node_id: Option<&NodeId>) -> Option<U>
    where
        U: for<'b> TryFrom<&'b StaticValue>,
    {
        panic!()
        // match self.scope.lookup(&path) {
        //     Some(val) => match val {
        //         ScopeValue::Dyn(path) => self.get(path, node_id),
        //         ScopeValue::Static(s) => T::try_from(s).ok(),
        //         ScopeValue::List(_) => None,
        //     },
        //     None => self
        //         .state
        //         .get(&path, node_id.into())
        //         .and_then(|val| val.as_ref().try_into().ok()),
        // }
    }

    pub fn attribute<U>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<U>
    where
        U: for<'attr> TryFrom<&'attr StaticValue>,
    {
        panic!()
        // let attrib = attributes.get(key.as_ref())?;

        // match attrib {
        //     ScopeValue::Static(val) => val.try_into().ok(),
        //     ScopeValue::Dyn(path) => self.get(path, node_id),
        //     _ => None,
        // }
    }

    pub fn primitive<U>(
        &self,
        key: impl AsRef<str>,
        node_id: Option<&NodeId>,
        attributes: &HashMap<String, ScopeValue>,
    ) -> Option<U>
    where
        U: for<'b> TryFrom<&'b StaticValue>,
    {
        panic!()
        // let attrib = attributes.get(key.as_ref())?;

        // match attrib {
        //     ScopeValue::Static(val) => T::try_from(val).ok(),
        //     ScopeValue::Dyn(path) => self.get::<T>(path, node_id),
        //     _ => None,
        // }
    }

    pub fn list_to_string(
        &self,
        list: &Rc<[ScopeValue]>,
        buffer: &mut String,
        node_id: Option<&NodeId>,
    ) {
        panic!()
        // for val in list.iter() {
        //     match val {
        //         ScopeValue::List(list) => self.list_to_string(list, buffer, node_id),
        //         ScopeValue::Dyn(path) => buffer.push_str(&self.get_string(path, node_id)),
        //         ScopeValue::Static(s) => drop(write!(buffer, "{s}")),
        //     }
        // }
    }

    pub fn get_string(&self, path: &Path, node_id: Option<&NodeId>) -> String {
        panic!()
        // match self.scope.lookup(path) {
        //     Some(val) => match val {
        //         ScopeValue::Dyn(path) => self.get_string(path, node_id),
        //         ScopeValue::Static(s) => s.to_string(),
        //         ScopeValue::List(list) => {
        //             let mut buffer = String::new();
        //             self.list_to_string(list, &mut buffer, node_id);
        //             buffer
        //         }
        //     },
        //     None => self
        //         .state
        //         .get(&path, node_id)
        //         .map(|val| val.to_string())
        //         .unwrap_or_else(String::new),
        // }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::testing::*;

    type Sub = usize;

    #[test]
    fn scope_value() {
        panic!("can't deref to str");
        // let mut scope = Scope::new(None);
        // scope.scope(
        //     "value".into(),
        //     Cow::Owned(ScopeValue::Static("hello world".into())),
        // );

        // // let mut inner = Scope::new(Some(&scope));
        // scope.push();
        // scope.scope(
        //     "value".into(),
        //     Cow::Owned(ScopeValue::Static("inner hello".into())),
        // );

        // let value = scope.lookup(&"value".into()).unwrap();

        // let ScopeValue::Static(lhs) = scope.lookup(&"value".into()).unwrap() else {
        //     panic!()
        // };
        // assert_eq!(&**lhs, "inner hello");

        // scope.pop();

        // let ScopeValue::Static(lhs) = scope.lookup(&"value".into()).unwrap() else {
        //     panic!()
        // };
        // assert_eq!(&**lhs, "hello world");
    }

    #[test]
    fn dynamic_attribute() {
        panic!("see above test");
        // let mut state = TestState::new();
        // let mut root = Scope::new(None);
        // let mut ctx = Context::new(&mut state, &mut root);
        // let mut attributes = HashMap::new();
        // attributes.insert(
        //     "name".to_string(),
        //     ScopeValue::Dyn(Path::Key("name".into())),
        // );

        // let id = Some(123.into());
        // let name: Option<String> = ctx.attribute("name", id.as_ref(), &attributes);

        // assert_eq!("Dirk Gently", name.unwrap());
    }
}
