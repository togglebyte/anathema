use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;

use super::components::AssocEventMapping;
use super::expressions::ExpressionId;
use super::ComponentBlueprintId;

/// A singular named node
#[derive(Debug, Clone, PartialEq)]
pub struct Single {
    pub ident: String,
    pub children: Vec<Blueprint>,
    pub attributes: SmallMap<String, ExpressionId>,
    pub value: Option<ExpressionId>,
}

/// A `for-each` node
#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub binding: String,
    pub data: ExpressionId,
    pub body: Vec<Blueprint>,
}

/// Scoping a value with a given name:
/// ```text
/// with 123 as number
///     text number
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct With {
    pub binding: String,
    pub data: ExpressionId,
    pub body: Vec<Blueprint>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ControlFlow {
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
    /// A singular widget
    Single(Single),
    /// A for-loop
    For(For),
    /// A `with` statement
    With(With),
    /// If / else
    ControlFlow(ControlFlow),
    /// A component
    Component(Component),
    /// A slot for a component
    Slot(Vec<Self>),
}

macro_rules! single {
    ($ident:expr) => {
        $crate::templates::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: vec![],
            attributes: SmallMap::empty(),
            value: None,
        })
    };
    (value @ $ident:expr, $value:expr) => {
        $crate::templates::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: vec![],
            attributes: SmallMap::empty(),
            value: Some($value.into()),
        })
    };
    (children @ $ident:expr, $children:expr) => {
        $crate::templates::blueprints::Blueprint::Single(Single {
            ident: $ident.into(),
            children: $children,
            attributes: SmallMap::empty(),
            value: None,
        })
    };
}

macro_rules! forloop {
    ($binding:expr, $data:expr, $body:expr) => {
        $crate::templates::blueprints::Blueprint::For(For {
            binding: $binding.into(),
            data: $data,
            body: $body,
        })
    };
}

pub(crate) use {single, forloop};
