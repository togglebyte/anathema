use std::collections::HashMap;
use std::rc::Rc;

use anathema_store::smallmap::SmallMap;

use crate::components::ComponentId;
use crate::Expression;

#[derive(Debug, Clone, PartialEq)]
pub struct Single {
    pub ident: Rc<str>,
    pub children: Vec<Blueprint>,
    pub attributes: SmallMap<Rc<str>, Expression>,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub binding: Rc<str>,
    pub data: Expression,
    pub body: Vec<Blueprint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlFlow {
    pub if_node: If,
    pub elses: Vec<Else>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct If {
    pub cond: Expression,
    pub body: Vec<Blueprint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Else {
    pub cond: Option<Expression>,
    pub body: Vec<Blueprint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub id: ComponentId,
    pub body: Vec<Blueprint>,
    pub attributes: SmallMap<Rc<str>, Expression>,
    pub state: Option<Rc<HashMap<Rc<str>, Expression>>>,
}

/// A blueprint represents what widget should be built from the information
#[derive(Clone, Debug, PartialEq)]
pub enum Blueprint {
    Single(Single),
    For(For),
    ControlFlow(ControlFlow),
    Component(Component),
}

#[macro_export]
macro_rules! single {
    ($ident:expr) => {
        $crate::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: vec![],
            attributes: SmallMap::empty(),
            value: None,
        })
    };
    ($ident:expr, $children:expr) => {
        $crate::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: $children,
            attributes: SmallMap::empty(),
            value: None,
        })
    };
}
