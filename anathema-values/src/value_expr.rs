use std::fmt::Display;
use std::rc::Rc;

use smallvec::SmallVec;

use crate::hashmap::HashMap;
use crate::value::{ExpressionMap, Expressions};
use crate::{Context, NodeId, Num, Owned, Path, ScopeValue, ValueRef};

// // -----------------------------------------------------------------------------
// //   - This is so insanely gross... -
// //   that it should be set on fire and forgotten
// //   TODO: rewrite this to something less gross
// // -----------------------------------------------------------------------------
// fn lookup_static_collection<'a>(
//     value_ref: ValueRef<'a>,
//     path: &Path,
//     resolver: &mut impl Resolver<'a>,
// ) -> ValueRef<'a> {
//     match value_ref {
//         ValueRef::ExpressionMap(_) | ValueRef::Expressions(_) => {}
//         val => return val,
//     }

//     match value_ref {
//         ValueRef::ExpressionMap(map) => {}
//         ValueRef::Expressions(list) => {}
//         _ => unreachable!(),
//     }

//     if let Some(path) = path.rhs() {
//         match path {
//             Path::Index(idx) => {
//                 if let ValueRef::Expressions(list) = value_ref {
//                     return list
//                         .0
//                         .get(*idx)
//                         .map(|val| val.eval(resolver))
//                         .unwrap_or(ValueRef::Empty);
//                 }
//             }
//             Path::Key(key) => {
//                 if let ValueRef::ExpressionMap(map) = value_ref {
//                     return map
//                         .0
//                         .get(key)
//                         .map(|val| val.eval(resolver))
//                         .unwrap_or(ValueRef::Empty);
//                 }
//             }
//             Path::Composite(lhs, rhs) => match &**lhs {
//                 Path::Index(idx) => {
//                     if let ValueRef::Expressions(list) = value_ref {
//                         match list
//                             .0
//                             .get(*idx)
//                             .map(|val| val.eval(resolver))
//                             .unwrap_or(ValueRef::Empty)
//                         {
//                             val @ ValueRef::Expressions(_) | val @ ValueRef::ExpressionMap(_) => {
//                                 return lookup_static_collection(val, path, resolver);
//                             }
//                             _ => return ValueRef::Empty,
//                         }
//                     }
//                 }
//                 Path::Key(key) => {
//                     if let ValueRef::ExpressionMap(map) = value_ref {
//                         match map
//                             .0
//                             .get(key)
//                             .map(|val| val.eval(resolver))
//                             .unwrap_or(ValueRef::Empty)
//                         {
//                             val @ ValueRef::Expressions(_) | val @ ValueRef::ExpressionMap(_) => {
//                                 return lookup_static_collection(val, path, resolver);
//                             }
//                             _ => return ValueRef::Empty,
//                         }
//                     }
//                 }
//                 _ => unreachable!(),
//             },
//         }
//     }

//     value_ref
// }

// -----------------------------------------------------------------------------
//   - Value resolver trait -
// -----------------------------------------------------------------------------
pub trait Resolver<'expr> {
    fn resolve(&mut self, path: &Path) -> ValueRef<'expr>;

    fn lookup(&mut self, ident: &str) -> ValueRef<'expr>;

    fn resolve_list_lookup(&mut self, list: &'expr ValueExpr, index: usize) -> ValueRef<'expr>;

    fn resolve_map_lookup(&mut self, map: &'expr ValueExpr, ident: &str) -> ValueRef<'expr>;

    // fn resolve_number(&mut self, value: &'expr ValueExpr) -> Option<Num>;

    // fn resolve_bool(&mut self, value: &'expr ValueExpr) -> bool;

    // fn resolve_path(&mut self, value: &'expr ValueExpr) -> Option<Path>;

    // fn lookup_path(&mut self, path: &Path) -> ValueRef<'expr>;
}

// -----------------------------------------------------------------------------
//   - Deferred -
// -----------------------------------------------------------------------------
/// Only resolve up until a deferred path.
/// This means `ValueExpr::Deferred` will not be resolved, and instead returned.
pub struct Deferred<'a, 'expr> {
    context: &'a Context<'a, 'expr>,
}

impl<'a, 'expr> Deferred<'a, 'expr> {
    pub fn new(context: &'a Context<'a, 'expr>) -> Self {
        Self { context }
    }
}

impl<'a, 'expr> Resolver<'expr> for Deferred<'a, 'expr> {
    fn resolve(&mut self, path: &Path) -> ValueRef<'expr> {
        match self.context.scopes.lookup(path) {
            None => ValueRef::Deferred,
            Some(ScopeValue::Value(value)) => value.clone(),
            Some(ScopeValue::Deferred(..)) => panic!("not sure what to do here yet, can this even happen?"),
            Some(ScopeValue::DeferredList(..)) => panic!("not sure what to do here yet, can this even happen?"),
        }
    }

    fn lookup(&mut self, ident: &str) -> ValueRef<'expr> {
        let val = self.context.scopes.lookup(&(ident.into()));
        match val {
            Some(ScopeValue::Value(value)) => *value,
            _ => ValueRef::Deferred,
        }
    }

    fn resolve_list_lookup(&mut self, list: &'expr ValueExpr, index: usize) -> ValueRef<'expr> {
        match list.eval(self) {
            ValueRef::Expressions(list) => list
                .get(index)
                .map(|expr| expr.eval(self))
                .unwrap_or(ValueRef::Empty),
            ValueRef::Deferred => ValueRef::Deferred,
            lark => panic!("{lark:?}"),
            _ => ValueRef::Empty,
        }
    }

    fn resolve_map_lookup(&mut self, map: &'expr ValueExpr, ident: &str) -> ValueRef<'expr> {
        match map.eval(self) {
            ValueRef::ExpressionMap(map) => map
                .get(ident)
                .map(|expr| expr.eval(self))
                .unwrap_or(ValueRef::Empty),
            ValueRef::Deferred => ValueRef::Deferred,
            lark => panic!("{lark:?}"),
            _ => ValueRef::Empty,
        }
    }

    // fn resolve_number(&mut self, value: &'expr ValueExpr) -> Option<Num> {
    //     match value.eval(self) {
    //         ValueRef::Owned(Owned::Num(num)) => Some(num),
    //         _ => None,
    //     }
    // }

    // fn resolve_bool(&mut self, value: &'expr ValueExpr) -> bool {
    //     value.eval(self).is_true()
    // }

    // fn resolve_path(&mut self, value: &'expr ValueExpr) -> Option<Path> {
    //     match value {
    //         ValueExpr::Ident(path) => Some(Path::from(&**path)),
    //         ValueExpr::Index(lhs, index) => {
    //             // lhs can only be either an ident or an index
    //             let lhs = self.resolve_path(lhs)?;
    //             let index = self.resolve_number(index)?.to_usize();
    //             Some(lhs.compose(index))
    //         }
    //         ValueExpr::Dot(lhs, rhs) => {
    //             let lhs = self.resolve_path(lhs)?;
    //             let rhs = self.resolve_path(rhs)?;
    //             Some(lhs.compose(rhs))
    //         }
    //         _ => None,
    //     }
    // }

    // fn lookup_path(&mut self, path: &Path) -> ValueRef<'expr> {
    //     match self.context.scopes.lookup(path) {
    //         value @ ValueRef::ExpressionMap(_) => lookup_static_collection(value, path, self),
    //         value @ ValueRef::Expressions(_) => lookup_static_collection(value, path, self),
    //         ValueRef::Empty => ValueRef::Deferred(path.clone()),
    //         val => val,
    //     }
    // }
}

// -----------------------------------------------------------------------------
//   - Resolver -
//   This should never return a deferred value, instead
//   it should resolve any deferred value before returning
//
//   The immediate resolver is the only resolver that will
//   access the state, therefore no other resolver needs a NodeId
// -----------------------------------------------------------------------------
/// Resolve the expression, including deferred values.
pub struct Immediate<'ctx, 'state> {
    context: &'ctx Context<'state, 'state>,
    node_id: &'state NodeId,
    is_deferred: bool,
}

impl<'ctx, 'state> Immediate<'ctx, 'state> {
    pub fn new(context: &'ctx Context<'state, 'state>, node_id: &'state NodeId) -> Self {
        Self {
            context,
            node_id,
            is_deferred: false,
        }
    }
}

impl<'state> Immediate<'_, 'state> {
    // pub fn resolve(&mut self, value: &'state ValueExpr) -> ValueRef<'state> {
    //     match value.eval(self) {
    //         ValueRef::Deferred => {
    //             self.is_deferred = true;
    //             self.context.state.get(&path, self.node_id)
    //         }
    //         val => val,
    //     }
    // }

    pub fn is_deferred(&self) -> bool {
        self.is_deferred
    }

    // pub fn resolve_string(&mut self, value: &'state ValueExpr) -> Option<String> {
    //     match value.eval(self) {
    //         ValueRef::Str(s) => Some(s.into()),
    //         ValueRef::Owned(s) => Some(s.to_string()),
    //         ValueRef::Expressions(Expressions(list)) => {
    //             let mut s = String::new();
    //             for expr in list {
    //                 let res = self.resolve_string(expr);
    //                 if let Some(res) = res {
    //                     s.push_str(&res);
    //                 }
    //             }
    //             Some(s)
    //         }
    //         ValueRef::Deferred(path) => {
    //             self.is_deferred = true;
    //             match self.context.state.get(&path, self.node_id) {
    //                 ValueRef::Str(val) => Some(val.into()),
    //                 ValueRef::Owned(val) => Some(val.to_string()),
    //                 ValueRef::Empty => None,
    //                 _ => None,
    //             }
    //         }
    //         ValueRef::Empty => None,
    //         _ => None,
    //     }
    // }

    // pub fn resolve_list<T>(&mut self, value: &'state ValueExpr) -> SmallVec<[T; 4]>
    // where
    //     T: for<'b> TryFrom<ValueRef<'b>>,
    // {
    //     let mut output = SmallVec::<[T; 4]>::new();
    //     let value = value.eval(self);
    //     let value = match value {
    //         ValueRef::Deferred(path) => {
    //             self.is_deferred = true;
    //             self.context.state.get(&path, self.node_id)
    //         }
    //         val => val,
    //     };

    //     let mut resolver = Self::new(self.context, self.node_id);
    //     match value {
    //         ValueRef::Expressions(Expressions(list)) => {
    //             for expr in list {
    //                 let val = expr.eval(&mut resolver);
    //                 let Ok(val) = T::try_from(val) else { continue };
    //                 output.push(val);
    //             }

    //             if resolver.is_deferred {
    //                 self.is_deferred = true;
    //             }

    //             output
    //         }
    //         val => {
    //             let Ok(val) = T::try_from(val) else {
    //                 return output;
    //             };
    //             output.push(val);
    //             output
    //         }
    //     }
    // }
}

impl<'state> Resolver<'state> for Immediate<'_, 'state> {
    fn resolve(&mut self, path: &Path) -> ValueRef<'state> {
        match self.context.scopes.lookup(path) {
            Some(ScopeValue::Value(value)) => *value,
            Some(ScopeValue::Deferred(expr)) => {
                self.is_deferred = true;
                expr.eval(self)
            }
            Some(&ScopeValue::DeferredList(index, ref expr)) => {
                self.is_deferred = true;
                match expr.eval(self) {
                    ValueRef::List(list) => {
                        let path = index.into();
                        list.state_get(&path, self.node_id)
                    }
                    // TODO: this might be unreachable, investimagate!
                    _ => panic!(),
                }
            }
            None => match self.context.state.state_get(path, self.node_id) {
                ValueRef::Empty => ValueRef::Empty,
                val => {
                    self.is_deferred = true;
                    val
                }
            },
        }
    }

    fn lookup(&mut self, ident: &str) -> ValueRef<'state> {
        let path = ident.into();
        self.resolve(&path)
    }

    fn resolve_list_lookup(&mut self, list: &'state ValueExpr, index: usize) -> ValueRef<'state> {
        match list.eval(self) {
            ValueRef::List(list) => {
                let index = index.into();
                list.state_get(&index, self.node_id)
            }
            ValueRef::Expressions(list) => {
                let value_expr = match list.get(index) {
                    None => return ValueRef::Empty,
                    Some(expr) => expr,
                };
                value_expr.eval(self)
            }
            lark => panic!("{lark:?}"),
            _ => ValueRef::Empty,
        }
    }

    fn resolve_map_lookup(&mut self, map: &'state ValueExpr, ident: &str) -> ValueRef<'state> {
        match map.eval(self) {
            ValueRef::Map(map) => {
                let ident = ident.into();
                map.state_get(&ident, self.node_id)
            }
            ValueRef::ExpressionMap(map) => {
                let value_expr = match map.get(ident) {
                    None => return ValueRef::Empty,
                    Some(expr) => expr,
                };
                value_expr.eval(self)
            }
            _ => ValueRef::Empty,
        }
    }

    // fn resolve_number(&mut self, value: &'state ValueExpr) -> Option<Num> {
    //     match value.eval(self) {
    //         ValueRef::Owned(Owned::Num(num)) => Some(num),
    //         ValueRef::Deferred(path) => {
    //             self.is_deferred = true;
    //             match self.context.state.get(&path, self.node_id) {
    //                 ValueRef::Owned(Owned::Num(num)) => Some(num),
    //                 _ => None,
    //             }
    //         }
    //         _ => None,
    //     }
    // }

    // fn resolve_bool(&mut self, value: &'state ValueExpr) -> bool {
    //     match value.eval(self) {
    //         ValueRef::Deferred(path) => {
    //             self.is_deferred = true;
    //             self.context.state.get(&path, self.node_id).is_true()
    //         }
    //         val => val.is_true(),
    //     }
    // }

    // fn resolve_path(&mut self, value: &'state ValueExpr) -> Option<Path> {
    //     match value {
    //         ValueExpr::Ident(path) => {
    //             let path = Path::from(&**path);
    //             match self.context.scopes.lookup(&path) {
    //                 ValueRef::Deferred(path) => Some(path),
    //                 ValueRef::Empty => Some(path),
    //                 val => {
    //                     Some(path)
    //                     // panic!("this should never be anythign but a deferred path: {val:?}")
    //                 }
    //             }
    //         }
    //         ValueExpr::Index(lhs, index) => {
    //             // lhs can only be either an ident or an index
    //             let lhs = self.resolve_path(lhs)?;
    //             let index = self.resolve_number(index)?.to_usize();
    //             Some(lhs.compose(index))
    //         }
    //         ValueExpr::Dot(lhs, rhs) => {
    //             let lhs = self.resolve_path(lhs)?;
    //             let rhs = self.resolve_path(rhs)?;
    //             Some(lhs.compose(rhs))
    //         }
    //         _ => None,
    //     }
    // }

    // fn lookup_path(&mut self, path: &Path) -> ValueRef<'state> {
    //     match self.context.scopes.lookup(path) {
    //         ValueRef::Deferred(ref path) => {
    //             self.is_deferred = true;
    //             self.context.state.get(path, self.node_id)
    //         }
    //         ValueRef::Empty => {
    //             self.is_deferred = true;
    //             self.context.state.get(path, self.node_id)
    //         }
    //         value @ ValueRef::ExpressionMap(_) => lookup_static_collection(value, path, self),
    //         value @ ValueRef::Expressions(_) => lookup_static_collection(value, path, self),
    //         val => val,
    //     }
    // }
}

// -----------------------------------------------------------------------------
//   - Value expressoin -
// -----------------------------------------------------------------------------
// TODO: rename this to `Expression` and rename `compiler::Expression` to something else
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    Owned(Owned),
    String(Rc<str>),

    Not(Box<ValueExpr>),
    Negative(Box<ValueExpr>),
    And(Box<ValueExpr>, Box<ValueExpr>),
    Or(Box<ValueExpr>, Box<ValueExpr>),
    Equality(Box<ValueExpr>, Box<ValueExpr>),

    Ident(Rc<str>),
    Dot(Box<ValueExpr>, Box<ValueExpr>),
    Index(Box<ValueExpr>, Box<ValueExpr>),

    // TODO: does the list and the hashmap even need to be RCd?
    List(Rc<[ValueExpr]>),
    Map(Rc<HashMap<String, ValueExpr>>),

    Add(Box<ValueExpr>, Box<ValueExpr>),
    Sub(Box<ValueExpr>, Box<ValueExpr>),
    Div(Box<ValueExpr>, Box<ValueExpr>),
    Mul(Box<ValueExpr>, Box<ValueExpr>),
    Mod(Box<ValueExpr>, Box<ValueExpr>),
}

impl Display for ValueExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Owned(val) => write!(f, "{val}"),
            Self::String(val) => write!(f, "{val}"),
            Self::Ident(s) => write!(f, "{s}"),
            Self::Index(lhs, idx) => write!(f, "{lhs}[{idx}]"),
            Self::Dot(lhs, rhs) => write!(f, "{lhs}.{rhs}"),
            Self::Not(expr) => write!(f, "!{expr}"),
            Self::Negative(expr) => write!(f, "-{expr}"),
            Self::Add(lhs, rhs) => write!(f, "{lhs} + {rhs}"),
            Self::Sub(lhs, rhs) => write!(f, "{lhs} - {rhs}"),
            Self::Mul(lhs, rhs) => write!(f, "{lhs} * {rhs}"),
            Self::Div(lhs, rhs) => write!(f, "{lhs} / {rhs}"),
            Self::Mod(lhs, rhs) => write!(f, "{lhs} % {rhs}"),
            Self::List(list) => {
                write!(
                    f,
                    "[{}]",
                    list.iter()
                        .map(|val| val.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
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
            Self::And(lhs, rhs) => write!(f, "{lhs} && {rhs}"),
            Self::Or(lhs, rhs) => write!(f, "{lhs} || {rhs}"),
            Self::Equality(lhs, rhs) => write!(f, "{lhs} == {rhs}"),
        }
    }
}

macro_rules! eval_num {
    ($e:expr, $resolver:expr) => {
        match $e.eval($resolver) {
            ValueRef::Owned(Owned::Num(num)) => num,
            ValueRef::Deferred => return ValueRef::Deferred,
            _ => return ValueRef::Empty,
        }
    };
}

impl ValueExpr {
    fn eval_list_lookup<'expr>(
        &'expr self,
        resolver: &mut impl Resolver<'expr>,
        index: &'expr ValueExpr,
    ) -> ValueRef<'expr> {
        let index = match index.eval(resolver) {
            ValueRef::Owned(Owned::Num(n)) => n.to_usize(),
            ValueRef::Deferred => return ValueRef::Deferred,
            _ => return ValueRef::Empty,
        };

        resolver.resolve_list_lookup(self, index)
    }

    fn eval_map_lookup<'expr>(
        &'expr self,
        resolver: &mut impl Resolver<'expr>,
        ident: &'expr ValueExpr,
    ) -> ValueRef<'expr> {
        let ident = match ident {
            ValueExpr::Ident(ident) => ident,
            _ => return ValueRef::Empty,
        };

        resolver.resolve_map_lookup(self, ident)
    }

    pub fn eval_string<'expr>(&'expr self, resolver: &mut impl Resolver<'expr>) -> Option<String> {
        match self.eval(resolver) {
            ValueRef::Str(s) => Some(s.into()),
            ValueRef::Owned(s) => Some(s.to_string()),
            ValueRef::Expressions(Expressions(list)) => {
                let mut s = String::new();
                for expr in list {
                    let res = expr.eval_string(resolver);
                    if let Some(res) = res {
                        s.push_str(&res);
                    }
                }
                Some(s)
            }
            ValueRef::Deferred => {
                panic!()
                // self.is_deferred = true;
                // match self.context.state.get(&path, self.node_id) {
                //     ValueRef::Str(val) => Some(val.into()),
                //     ValueRef::Owned(val) => Some(val.to_string()),
                //     ValueRef::Empty => None,
                //     _ => None,
                // }
            }
            ValueRef::Empty => None,
            _ => None,
        }
    }

    // Even though the lifetime is named `'expr`, the value isn't necessarily tied to an expression.
    //
    // Static values originate from expressions and will have the aforementioned lifetime,
    // however a value could also stem from a state (by resolving a deferred value).
    // A value that originates from `State` can only live for the duration of the layout phase.
    pub fn eval<'expr>(&'expr self, resolver: &mut impl Resolver<'expr>) -> ValueRef<'expr> {
        match self {
            Self::Owned(value) => ValueRef::Owned(*value),
            Self::String(value) => ValueRef::Str(value),

            // -----------------------------------------------------------------------------
            //   - Maths -
            // -----------------------------------------------------------------------------
            op @ (Self::Add(lhs, rhs)
            | Self::Sub(lhs, rhs)
            | Self::Mul(lhs, rhs)
            | Self::Mod(lhs, rhs)
            | Self::Div(lhs, rhs)) => {
                let lhs = eval_num!(lhs, resolver);
                let rhs = eval_num!(rhs, resolver);

                match op {
                    Self::Add(..) => ValueRef::Owned(Owned::Num(lhs + rhs)),
                    Self::Sub(..) => ValueRef::Owned(Owned::Num(lhs - rhs)),
                    Self::Mul(..) => ValueRef::Owned(Owned::Num(lhs * rhs)),
                    Self::Mod(..) => ValueRef::Owned(Owned::Num(lhs % rhs)),
                    Self::Div(..) if !rhs.is_zero() => ValueRef::Owned(Owned::Num(lhs / rhs)),
                    Self::Div(..) => ValueRef::Empty,
                    _ => unreachable!(),
                }
            }

            Self::Negative(expr) => {
                let num = eval_num!(expr, resolver);
                ValueRef::Owned(Owned::Num(num.to_negative()))
            }

            // -----------------------------------------------------------------------------
            //   - Conditions -
            // -----------------------------------------------------------------------------
            Self::Not(expr) => {
                let b = expr.eval(resolver).is_true();
                ValueRef::Owned((!b).into())
            }
            Self::Equality(lhs, rhs) => {
                let lhs = lhs.eval(resolver);
                let rhs = rhs.eval(resolver);
                ValueRef::Owned((lhs == rhs).into())
            }
            Self::Or(lhs, rhs) => {
                let lhs = lhs.eval(resolver);
                let rhs = rhs.eval(resolver);
                ValueRef::Owned((lhs.is_true() || rhs.is_true()).into())
            }
            Self::And(lhs, rhs) => {
                let lhs = lhs.eval(resolver);
                let rhs = rhs.eval(resolver);
                ValueRef::Owned((lhs.is_true() && rhs.is_true()).into())
            }

            // -----------------------------------------------------------------------------
            //   - Paths -
            // -----------------------------------------------------------------------------
            Self::Ident(ident) => resolver.lookup(ident),
            Self::Index(lhs, index) => lhs.eval_list_lookup(resolver, index),
            Self::Dot(lhs, rhs) => lhs.eval_map_lookup(resolver, rhs),

            // -----------------------------------------------------------------------------
            //   - Collection -
            // -----------------------------------------------------------------------------
            Self::List(list) => ValueRef::Expressions(Expressions::new(list)),
            Self::Map(map) => ValueRef::ExpressionMap(ExpressionMap::new(map)),
        }
    }
}

impl From<Box<ValueExpr>> for ValueExpr {
    fn from(val: Box<ValueExpr>) -> Self {
        *val
    }
}

impl<T> From<T> for ValueExpr
where
    T: Into<Owned>,
{
    fn from(val: T) -> Self {
        Self::Owned(val.into())
    }
}

impl From<String> for ValueExpr {
    fn from(val: String) -> Self {
        Self::String(val.into())
    }
}

impl From<&str> for ValueExpr {
    fn from(val: &str) -> Self {
        Self::String(val.into())
    }
}

#[cfg(test)]
mod test {
    use crate::map::Map;
    use crate::testing::{
        add, and, div, dot, eq, ident, inum, list, modulo, mul, neg, not, or, strlit, sub, unum,
    };
    use crate::ValueRef;

    #[test]
    fn add_dyn() {
        let expr = add(neg(inum(1)), neg(unum(2)));
        expr.with_data([("counter", 2usize)]).expect_owned(-3);
    }

    #[test]
    fn add_static() {
        let expr = add(neg(inum(1)), neg(unum(2)));
        expr.test().expect_owned(-3);
    }

    #[test]
    fn sub_static() {
        let expr = sub(unum(10), unum(2));
        expr.test().expect_owned(8u8);
    }

    #[test]
    fn mul_static() {
        let expr = mul(unum(10), unum(2));
        expr.test().expect_owned(20u8);
    }

    #[test]
    fn div_static() {
        let expr = div(unum(10), unum(2));
        expr.test().expect_owned(5u8);
    }

    #[test]
    fn mod_static() {
        let expr = modulo(unum(5), unum(3));
        expr.test().expect_owned(2u8);
    }

    #[test]
    fn bools() {
        // false
        let expr = ident("is_false");
        expr.with_data([("is_false", false)]).expect_owned(false);

        // not is false
        let expr = not(ident("is_false"));
        expr.with_data([("is_false", false)]).expect_owned(true);

        // equality
        let expr = eq(ident("one"), ident("one"));
        expr.with_data([("one", 1)]).eval_bool(true);

        // not equality
        let expr = not(eq(ident("one"), ident("two")));
        expr.with_data([("one", 1), ("two", 2)]).eval_bool(true);

        // or
        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", true)])
            .eval_bool(true);

        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", false)])
            .eval_bool(true);

        let expr = or(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", false)])
            .eval_bool(false);

        // and
        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", true)])
            .eval_bool(true);

        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", false), ("two", true)])
            .eval_bool(false);

        let expr = and(ident("one"), ident("two"));
        expr.with_data([("one", true), ("two", false)])
            .eval_bool(false);
    }

    #[test]
    fn path() {
        let test = dot(ident("inner"), ident("name"))
            .with_data([("inner", Map::new([("name", "Fiddle McStick".to_string())]))]);
        let name = test.eval();
        assert!(matches!(name, ValueRef::Str("Fiddle McStick")));
    }

    #[test]
    fn string() {
        let expr = list(vec![strlit("Mr. "), dot(ident("inner"), ident("name"))]);
        let string = expr
            .with_data([("inner", Map::new([("name", "Fiddle McStick".to_string())]))])
            .eval_string()
            .unwrap();
        assert_eq!(string, "Mr. Fiddle McStick");
    }
}
