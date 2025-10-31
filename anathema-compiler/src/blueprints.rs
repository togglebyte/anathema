use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;

use super::ComponentBlueprintId;
use super::components::AssocEventMapping;
use super::expressions::ExpressionId;

/// A singular named node in the blueprint tree (e.g., "text", "container")
#[derive(Debug, Clone, PartialEq)]
pub struct Single {
    pub ident: String,
    /// Associated children
    pub children: Vec<Blueprint>,
    /// Key-value pairs of widget attributes mapped to their expressions
    pub attributes: SmallMap<String, ExpressionId>,
    /// Expression representing the widget's value content
    pub value: Option<ExpressionId>,
}

/// Iterating over a collection with a binding:
/// ```text
/// for item in items
///     text item
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct For {
    /// The loop variable name binding
    pub binding: String,
    /// Expression representing the collection
    pub data: ExpressionId,
    /// The body of the for-loop
    pub body: Vec<Blueprint>,
}

/// Scoping a value with a given name:
/// ```text
/// with 123 as number
///     text number
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct With {
    /// The variable name binding for the scoped value
    pub binding: String,
    /// Data to be scoped
    pub data: ExpressionId,
    /// The body of the with-scope
    pub body: Vec<Blueprint>,
}

/// Conditional control flow structure for if/else statements:
/// ```text
/// if condition
///     text "true branch"
/// else
///     text "false branch"
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct ControlFlow {
    /// A vector of else branches, where the first element represents the initial `if` condition
    /// and subsequent elements represent `else if` or final `else` branches
    pub elses: Vec<Else>,
}

/// A single branch in a conditional control flow structure.
/// Represents an `if`, `else if`, or `else` branch.
#[derive(Debug, Clone, PartialEq)]
pub struct Else {
    /// Optional condition expression; `None` represents a final `else` branch
    pub cond: Option<ExpressionId>,
    /// The body of this branch
    pub body: Vec<Blueprint>,
}

/// Components have attributes, body content, and event handlers
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    /// The name of the component as a string
    pub name: String,
    /// The interned string identifier for the component name
    pub name_id: StringId,
    /// Unique identifier for this component blueprint
    pub id: ComponentBlueprintId,
    /// The body of the component
    pub body: Vec<Blueprint>,
    /// Component attributes
    pub attributes: SmallMap<String, ExpressionId>,
    /// Associated event handler mappings for this component
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

pub(crate) use {forloop, single};
