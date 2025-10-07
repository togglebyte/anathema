use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;

use crate::ComponentBlueprintId;
use crate::components::AssocEventMapping;
use crate::expressions::ExpressionId;

#[derive(Debug, Clone, PartialEq)]
pub struct Single {
    pub ident: String,
    pub children: Vec<Blueprint>,
    pub attributes: SmallMap<String, ExpressionId>,
    pub value: Option<ExpressionId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub binding: String,
    pub data: ExpressionId,
    pub body: Vec<Blueprint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct With {
    pub binding: String,
    pub data: ExpressionId,
    pub body: Vec<Blueprint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlFlow {
    // pub if_node: If,
    pub elses: Vec<Else>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Else {
    pub cond: Option<ExpressionId>,
    pub body: Vec<Blueprint>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub name: String,
    pub name_id: StringId,
    pub id: ComponentBlueprintId,
    pub body: Vec<Blueprint>,
    pub attributes: SmallMap<String, ExpressionId>,
    pub assoc_functions: Vec<AssocEventMapping>,
    /// The parent component in the blueprint
    pub parent: Option<ComponentBlueprintId>,
}

/// A blueprint represents what widget should be built from the information
#[derive(Clone, Debug, PartialEq)]
pub enum Blueprint {
    Single(Single),
    For(For),
    With(With),
    ControlFlow(ControlFlow),
    Component(Component),
    Slot(Vec<Self>),
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
