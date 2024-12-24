use std::rc::Rc;

use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;

use crate::{Expression, ComponentBlueprintId};

#[derive(Debug, Clone, PartialEq)]
pub struct Single {
    pub ident: String,
    pub children: Vec<Blueprint>,
    pub attributes: SmallMap<String, Expression>,
    pub value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub binding: String,
    pub data: Expression,
    pub body: Vec<Blueprint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlFlow {
    // pub if_node: If,
    pub elses: Vec<Else>,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct If {
//     pub cond: Expression,
//     pub body: Vec<Blueprint>,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Else {
    pub cond: Option<Expression>,
    pub body: Vec<Blueprint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub id: ComponentBlueprintId,
    pub body: Vec<Blueprint>,
    pub attributes: SmallMap<String, Expression>,
    pub assoc_functions: Vec<(StringId, StringId)>,
    /// The parent component in the blueprint
    pub parent: Option<ComponentBlueprintId>,
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
    (value @ $ident:expr, $value:expr) => {
        $crate::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: vec![],
            attributes: SmallMap::empty(),
            value: Some($value.into()),
        })
    };
    (children @ $ident:expr, $children:expr) => {
        $crate::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: $children,
            attributes: SmallMap::empty(),
            value: None,
        })
    };
}

#[macro_export]
macro_rules! forloop {
    ($binding:expr, $data:expr, $body:expr) => {
        $crate::blueprints::Blueprint::For(For {
            binding: $binding.into(),
            data: $data,
            body: $body,
        })
    };
}
