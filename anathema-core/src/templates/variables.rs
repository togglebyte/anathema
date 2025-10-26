//! Variable scoping and management for template compilation.
//!
//! This module provides the infrastructure for managing variables during template
//! compilation. It implements a sophisticated scoping system that supports nested
//! scopes, variable shadowing, component boundaries, and both global and local variables.
//!
//! # Overview
//!
//! The variable system manages:
//!
//! - **Global Variables**: Values accessible from anywhere in the template
//! - **Local Variables**: Values scoped to specific blocks or components
//! - **Scope Hierarchy**: Nested scopes with lexical scoping rules
//! - **Component Boundaries**: Isolation between component scopes
//! - **Variable Shadowing**: Inner scopes can shadow outer variables
//!
//! # Scoping Rules
//!
//! Anathema templates use lexical scoping similar to most programming languages:
//!
//! - Variables declared in an outer scope are accessible in inner scopes
//! - Variables can be shadowed by declarations in inner scopes
//! - Component boundaries create scope boundaries that prevent access to outer variables
//! - Each `for` loop and `with` statement creates a new scope
//!
//! # Global Variables
//!
//! Global variables come in two flavors:
//!
//! - **Runtime Globals**: Registered from Rust code before compilation
//! - **Template Globals**: Defined in the template itself
//!
//! Runtime globals persist across template recompilations, while template globals
//! are cleared when the template is recompiled.
//!
//! ## Example
//!
//! ```rust,ignore
//! use anathema_core::templates::{Variables, Expression, Expressions};
//!
//! let mut variables = Variables::new();
//! let mut expressions = Expressions::empty();
//!
//! // Register a global from Rust
//! variables.register_global("app_name", "My App", &mut expressions)?;
//!
//! // In the template, this global can be accessed anywhere:
//! // text app_name
//! ```
//!
//! # Local Variables
//!
//! Local variables are created by template constructs like `for` and `with`:
//!
//! ```text
//! for item in items
//!     text item.name  // 'item' is a local variable
//!
//! with user.profile as profile
//!     text profile.bio  // 'profile' is a local variable
//! ```
//!
//! # Scope Hierarchy
//!
//! The scope system is organized as a tree, where each node is identified by a
//! [`ScopeId`]. Scope IDs are paths through the tree represented as arrays of indices:
//!
//! - Root scope: `[]`
//! - First child of root: `[0]`
//! - Second child of root: `[1]`
//! - First child of first child: `[0, 0]`
//!
//! This structure allows efficient variable lookup by walking up the tree from the
//! current scope to the root.
//!
//! # Component Boundaries
//!
//! Components create scope boundaries that prevent inner components from accessing
//! variables declared in outer components. This provides encapsulation and prevents
//! unintended dependencies between components.
//!
//! ```text
//! // Outer component
//! let outer_var = 123
//!
//! // Inner component - cannot access outer_var
//! @inner_component
//! ```
//!
//! # Variable Lifecycle
//!
//! 1. **Declaration**: Variable is created and stored with a unique ID
//! 2. **Scope Association**: Variable is associated with the current scope
//! 3. **Lookup**: Variable is looked up by name, walking up the scope tree
//! 4. **Access**: Variable's expression ID is retrieved for evaluation
//! 5. **Shadowing**: Inner scopes can declare variables with the same name
//!
//! # Implementation Details
//!
//! The system uses several data structures:
//!
//! - [`Variables`]: Main API for variable management
//! - [`ScopeId`]: Identifies a position in the scope tree
//! - [`VarId`]: Unique identifier for a variable instance
//! - [`Globals`]: Storage for global variables
//! - [`Declarations`]: Maps variable names to their IDs per scope
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use anathema_core::templates::{Variables, Expressions, Expression};
//!
//! let mut variables = Variables::new();
//! let mut expressions = Expressions::empty();
//!
//! // Define a local variable
//! let expr_id = expressions.insert_at_root(Expression::from(42));
//! let var_id = variables.define_local("count", expr_id);
//!
//! // Look up the variable
//! if let Some(var_id) = variables.fetch("count") {
//!     let expr_id = variables.load(var_id).unwrap();
//!     // Use the expression ID to evaluate the variable
//! }
//!
//! // Create a new scope
//! variables.push();
//! // Variables from outer scope are still accessible
//! // Can shadow outer variables here
//! variables.pop();
//! ```

use std::collections::HashMap;
use std::sync::OnceLock;

use anathema_store::slab::{Slab, SlabIndex};

use super::error::ErrorKind;
use super::expressions::{Expression, ExpressionId, Expressions};

#[derive(Debug, Copy, Clone)]
enum Global {
    // The global value was set from the runtime
    Runtime(ExpressionId),
    // The global value originates from a template
    Template(ExpressionId),
}

#[derive(Debug, Default, Clone)]
/// Storage for global variables accessible throughout the template.
///
/// Global variables can be registered from Rust code (runtime globals) or defined
/// in the template itself (template globals). Runtime globals persist across
/// template recompilations, while template globals are cleared on recompilation.
pub struct Globals(HashMap<String, Global>);

impl Globals {
    /// Create a new empty globals storage.
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    /// Check if a global variable with the given name exists.
    pub fn contains(&self, ident: &str) -> bool {
        self.0.contains_key(ident)
    }

    /// Get the expression ID for a global variable by name.
    pub fn get(&self, ident: &str) -> Option<ExpressionId> {
        match self.0.get(ident).copied()? {
            Global::Runtime(expression) | Global::Template(expression) => Some(expression),
        }
    }

    /// Set a global variable if it doesn't already exist.
    fn set(&mut self, ident: String, value: Global) {
        if self.0.contains_key(&ident) {
            return;
        }
        _ = self.0.insert(ident, value);
    }

    /// Clear all template-defined globals, keeping runtime globals.
    fn clear_template_globals(&mut self) {
        let mut clear = vec![];
        for (key, glob) in &self.0 {
            match glob {
                Global::Runtime(_) => continue,
                Global::Template(_) => clear.push(key.to_owned()),
            }
        }

        for key in clear {
            _ = self.0.remove(&key);
        }
    }
}

/// Unique identifier for a variable instance.
///
/// Each variable declaration gets a unique `VarId` that distinguishes it from
/// all other variables, even those with the same name in different scopes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarId(u32);

impl SlabIndex for VarId {
    const MAX: usize = usize::MAX;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u32)
    }
}

/// A variable binding with its associated expression.
///
/// Variables can either be definitions (where the value is computed at runtime) or
/// declarations (where the value is known at compile time).
#[derive(Debug, Clone)]
pub enum Variable {
    /// A variable whose value will be available at runtime.
    ///
    /// Used for `for` loop variables and `with` bindings where the actual value
    /// depends on runtime state.
    Definition(ExpressionId),

    /// A variable with a compile-time value.
    ///
    /// Used for local declarations and global values that are known during
    /// template compilation.
    Declaration(ExpressionId),
}

impl Variable {
    fn as_expression(&self) -> ExpressionId {
        match self {
            Variable::Definition(expr) | Variable::Declaration(expr) => *expr,
        }
    }
}

/// Identifier for a position in the scope tree.
///
/// A scope ID is a path through the scope tree, represented as a sequence of
/// child indices. This allows efficient navigation through the scope hierarchy
/// and precise identification of any scope.
///
/// # Structure
///
/// - Root scope: `[]`
/// - First child of root: `[0]`
/// - Second child of first child: `[0, 0]`
///
/// # Example
///
/// ```text
/// Root []
///   ├─ Child 0 [0]
///   │  └─ Grandchild [0, 0]
///   └─ Child 1 [1]
///      ├─ Grandchild [1, 0]
///      └─ Grandchild [1, 1]
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ScopeId(Box<[u16]>);

impl ScopeId {
    /// Get the root scope ID (empty path).
    pub(crate) fn root() -> &'static Self {
        static ROOT: OnceLock<ScopeId> = OnceLock::new();
        ROOT.get_or_init(|| ScopeId(Box::new([])))
    }

    /// Create the scope ID for a child at the given index.
    fn next(&self, index: u16) -> Self {
        let mut scope_id = Vec::with_capacity(self.0.len() + 1);
        scope_id.extend_from_slice(&self.0);
        scope_id.push(index);
        Self(scope_id.into())
    }

    /// Get the parent scope ID as a slice.
    ///
    /// # Panics
    ///
    /// Panics if called on the root scope (which has no parent).
    fn parent(&self) -> &[u16] {
        // Can't get the parent of the root
        assert!(self.0.len() > 1);

        let to = self.0.len() - 1;
        &self.0[..to]
    }

    /// Check if either scope ID is a sub-path of the other.
    ///
    /// Returns the length of the common prefix if one exists.
    #[cfg(test)]
    fn sub_path_len(&self, id: impl AsRef<[u16]>) -> Option<usize> {
        let id = id.as_ref();
        let len = id.len().min(self.0.len());
        let lhs = &self.0[..len];
        let rhs = &id[..len];
        (lhs == rhs).then_some(len)
    }

    #[cfg(test)]
    fn as_slice(&self) -> &[u16] {
        &self.0
    }

    /// Check if this scope contains another scope (is a prefix of it).
    fn contains(&self, other: impl AsRef<[u16]>) -> Option<&ScopeId> {
        let other = other.as_ref();
        let len = self.0.len();

        match other.len() >= len {
            true => (*self.0 == other[..len]).then_some(self),
            false => None,
        }
    }
}

impl AsRef<[u16]> for ScopeId {
    fn as_ref(&self) -> &[u16] {
        &self.0
    }
}

impl From<&[u16]> for ScopeId {
    fn from(value: &[u16]) -> Self {
        Self(value.into())
    }
}

impl<const N: usize> From<[u16; N]> for ScopeId {
    fn from(value: [u16; N]) -> Self {
        Self(value.into())
    }
}

#[derive(Debug)]
struct RootScope(Scope);

impl Default for RootScope {
    fn default() -> Self {
        Self(Scope::new(ScopeId(vec![0].into())))
    }
}

impl RootScope {
    fn get_scope_mut(&mut self, id: impl AsRef<[u16]>) -> &mut Scope {
        let mut scope = &mut self.0;
        let mut id = &id.as_ref()[1..];

        while !id.is_empty() {
            scope = &mut scope.children[id[0] as usize];
            id = &id[1..];
        }

        scope
    }
}

/// A node in the scope tree.
///
/// Each scope has an ID and can have child scopes, forming a hierarchical
/// structure that matches the nesting structure of the template.
#[derive(Debug)]
pub struct Scope {
    id: ScopeId,
    children: Vec<Scope>,
}

impl Scope {
    fn new(id: ScopeId) -> Self {
        Self { id, children: vec![] }
    }

    /// Create a new child scope and return its ID.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut scope = Scope::new(ScopeId::from([0]));
    /// let child_id = scope.create_child(); // Returns [0, 0]
    /// let child_id = scope.create_child(); // Returns [0, 1]
    /// ```
    fn create_child(&mut self) -> ScopeId {
        let index = self.children.len();
        let id = self.id.next(index as u16);
        self.children.push(Scope::new(id.clone()));
        id
    }
}

/// Maps variable names to their declarations across all scopes.
///
/// Each variable name can have multiple declarations in different scopes,
/// tracked as a list of (scope ID, variable ID) pairs. Lookup walks this
/// list to find the closest accessible declaration.
#[derive(Debug)]
struct Declarations(HashMap<String, Vec<(ScopeId, VarId)>>);

impl Declarations {
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// Add a variable declaration for the given scope.
    fn add(&mut self, ident: impl Into<String>, scope_id: impl Into<ScopeId>, value_id: impl Into<VarId>) {
        let value_id = value_id.into();
        let ids = self.0.entry(ident.into()).or_default();
        ids.push((scope_id.into(), value_id));
    }

    /// Look up a variable by name from the given scope.
    ///
    /// Searches for the closest accessible declaration within the boundary,
    /// walking up the scope tree from the given scope ID.
    fn get(&self, ident: &str, scope_id: impl AsRef<[u16]>, boundary: &ScopeId) -> Option<VarId> {
        self.0
            .get(ident)?
            .iter()
            .rev()
            // here we need to look up closest scope that is still within the last boundary
            .filter(|(scope, _)| boundary.contains(scope).is_some())
            .filter_map(|(scope, var)| scope.contains(&scope_id).map(|_| *var))
            .next()
    }
}

/// Variable management system for template compilation.
///
/// This is the main API for managing variables during template compilation.
/// It handles both global and local variables, scope management, and variable
/// lookup with proper scoping rules.
///
/// # Example
///
/// ```rust,ignore
/// use anathema_core::templates::{Variables, Expressions, Expression};
///
/// let mut variables = Variables::new();
/// let mut expressions = Expressions::empty();
///
/// // Define a local variable
/// let expr_id = expressions.insert_at_root(Expression::from("Hello"));
/// variables.define_local("greeting", expr_id);
///
/// // Look it up
/// if let Some(var_id) = variables.fetch("greeting") {
///     let expr_id = variables.load(var_id).unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct Variables {
    globals: Globals,
    root: RootScope,
    current: ScopeId,
    boundary: Vec<ScopeId>,
    store: Slab<VarId, Variable>,
    declarations: Declarations,
}

impl Default for Variables {
    fn default() -> Self {
        Self::new()
    }
}

impl Variables {
    pub fn new() -> Self {
        let root = RootScope::default();
        Self {
            globals: Globals::empty(),
            current: root.0.id.clone(),
            boundary: vec![],
            root,
            store: Slab::empty(),
            declarations: Declarations::new(),
        }
    }

    fn declare_at(&mut self, ident: impl Into<String>, var_id: VarId, id: ScopeId) -> VarId {
        let ident = ident.into();
        self.declarations.add(ident, id.clone(), var_id);
        var_id
    }

    fn set_global(&mut self, ident: impl Into<String>, global: Global) -> Result<(), ErrorKind> {
        let ident = ident.into();
        if self.globals.contains(&ident) {
            return Err(ErrorKind::GlobalAlreadyAssigned(ident));
        }

        self.globals.set(ident, global);
        Ok(())
    }

    /// Reset template-defined globals while keeping runtime globals.
    ///
    /// This is called when recompiling a template to clear any globals that
    /// were defined in the previous compilation, while preserving globals
    /// that were registered from Rust code.
    pub fn reset_globals(&mut self) {
        self.globals.clear_template_globals();
    }

    /// Register a global variable from Rust code.
    ///
    /// Runtime globals persist across template recompilations and are
    /// accessible from anywhere in the template.
    ///
    /// # Parameters
    ///
    /// - `ident`: The name of the global variable
    /// - `value`: The initial value as an expression
    /// - `expressions`: Expression storage for the value
    ///
    /// # Errors
    ///
    /// Returns an error if a global with this name already exists.
    pub fn register_global(
        &mut self,
        ident: impl Into<String>,
        value: impl Into<Expression>,
        expressions: &mut Expressions,
    ) -> Result<(), ErrorKind> {
        let id = expressions.insert(value.into(), ScopeId::root().clone());
        let global = Global::Runtime(id);
        self.set_global(ident, global)
    }

    /// Define a global variable from within the template.
    ///
    /// Template globals are cleared when the template is recompiled.
    ///
    /// # Errors
    ///
    /// Returns an error if a global with this name already exists.
    pub fn define_global(&mut self, ident: impl Into<String>, expression: ExpressionId) -> Result<(), ErrorKind> {
        let global = Global::Template(expression);
        self.set_global(ident, global)
    }

    /// Define a local variable in the current scope.
    ///
    /// # Parameters
    ///
    /// - `ident`: The name of the variable
    /// - `value`: The expression ID for the variable's value
    ///
    /// # Returns
    ///
    /// The unique [`VarId`] for this variable instance.
    pub fn define_local(&mut self, ident: impl Into<String>, value: ExpressionId) -> VarId {
        let scope_id = self.current.clone();
        let var_id = self.store.insert(Variable::Declaration(value));
        self.declare_at(ident, var_id, scope_id)
    }

    /// Declare a local variable whose value is determined at runtime.
    ///
    /// Used for `for` loop variables and `with` bindings where the actual
    /// value depends on runtime state.
    ///
    /// # Parameters
    ///
    /// - `ident`: The name of the variable
    /// - `expressions`: Expression storage for creating the variable expression
    ///
    /// # Returns
    ///
    /// The unique [`VarId`] for this variable instance.
    pub fn declare_local(&mut self, ident: impl Into<String>, expressions: &mut Expressions) -> VarId {
        let scope_id = self.current.clone();
        let ident = ident.into();
        let id = expressions.insert(Expression::Ident(ident.clone()), scope_id.clone());
        let value = Variable::Definition(id);
        let var_id = self.store.insert(value);
        self.declare_at(ident, var_id, scope_id)
    }

    /// Look up a variable by name from the current scope.
    ///
    /// Searches up the scope tree from the current scope to find the closest
    /// accessible variable with the given name.
    pub fn fetch(&self, ident: &str) -> Option<VarId> {
        self.declarations.get(ident, &self.current, self.boundary_ref())
    }

    /// Create a new scope with a boundary for component isolation.
    ///
    /// Scope boundaries prevent inner components from accessing variables
    /// declared in outer components, providing encapsulation.
    pub(crate) fn push_scope_boundary(&mut self) {
        self.push();
        self.boundary.push(self.current.clone());
    }

    /// Exit a scope boundary, returning to the parent scope.
    pub(crate) fn pop_scope_boundary(&mut self) {
        self.pop();
        self.boundary.pop();
    }

    /// Create a new child scope and enter it.
    ///
    /// All subsequent variable operations will occur within this new scope.
    pub(crate) fn push(&mut self) {
        let parent = self.root.get_scope_mut(&self.current);
        self.current = parent.create_child();
    }

    /// Exit the current scope and return to its parent.
    ///
    /// # Example
    ///
    /// If the current scope ID is `[0, 1, 2]`, after `pop()` it becomes `[0, 1]`.
    pub(crate) fn pop(&mut self) {
        self.current = self.current.parent().into();
    }

    /// Load the expression ID for a variable.
    ///
    /// # Parameters
    ///
    /// - `var`: The variable ID to load
    ///
    /// # Returns
    ///
    /// The expression ID associated with this variable, or `None` if not found.
    pub fn load(&self, var: VarId) -> Option<ExpressionId> {
        self.store.get(var).map(Variable::as_expression)
    }

    /// Convenience method to fetch and load a variable by name (test only).
    #[cfg(test)]
    fn fetch_load(&self, ident: &str) -> Option<ExpressionId> {
        let id = self.declarations.get(ident, &self.current, self.boundary_ref())?;
        self.load(id)
    }

    /// Look up a global variable by name.
    pub fn global_lookup(&self, ident: &str) -> Option<ExpressionId> {
        self.globals.get(ident)
    }

    fn boundary_ref(&self) -> &ScopeId {
        self.boundary.last().unwrap_or(ScopeId::root())
    }

    /// Get the current scope boundary.
    pub(crate) fn boundary(&self) -> ScopeId {
        self.boundary.last().unwrap_or(ScopeId::root()).clone()
    }
}

impl From<Variables> for HashMap<String, Variable> {
    fn from(mut vars: Variables) -> Self {
        let mut hm = HashMap::new();

        for (key, mut ids) in vars.declarations.0.into_iter() {
            let (_, var_id) = ids
                .pop()
                .expect("there is always at least one var id associated with a key");
            let val = vars.store.remove(var_id);
            hm.insert(key, val);
        }

        hm
    }
}

#[cfg(test)]
mod test {
    use super::*;

    impl From<usize> for VarId {
        fn from(value: usize) -> Self {
            VarId(value as u32)
        }
    }

    #[test]
    fn scope_id_next() {
        let id = ScopeId::from([0]);
        assert_eq!(id.next(0).as_slice(), &[0, 0]);
    }

    #[test]
    fn scope_id_parent() {
        let id = ScopeId::from([1, 0]);
        assert_eq!(id.parent(), &[1]);
    }

    #[test]
    fn scope_min() {
        let a = ScopeId::from([1, 0]);
        let b = ScopeId::from([1, 0, 0, 1]);
        let expected = [1, 0].len();
        let actual = a.sub_path_len(b).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn create_child() {
        let mut root = RootScope::default();
        let child_id = root.0.create_child();
        assert_eq!(root.0.children.len(), 1);
        assert_eq!(child_id.as_ref(), &[0, 0]);
    }

    #[test]
    fn variable_declaration() {
        let mut vars = Variables::new();
        let mut expressions = Expressions::empty();

        let expected = expressions.insert_at_root(Expression::from(123i64));
        vars.define_local("var", expected);
        let id = vars.fetch("var").unwrap();
        let value = vars.load(id).unwrap();

        assert_eq!(expected, value);
    }

    #[test]
    fn shadow_value() {
        let mut vars = Variables::new();
        let mut expressions = Expressions::empty();
        let ident = "var";
        let value_a = expressions.insert_at_root(Expression::from("1"));
        let value_b = expressions.insert_at_root(Expression::from("2"));

        let first_value_ref = vars.define_local(ident, value_a);
        let second_value_ref = vars.define_local(ident, value_b);
        assert_eq!(value_a, vars.load(first_value_ref).unwrap());
        assert_eq!(value_b, vars.load(second_value_ref).unwrap());
    }

    #[test]
    fn scoping_variables_inaccessible_sibling() {
        // Declare a variable in a sibling and fail to access that value
        let mut vars = Variables::new();
        let mut expressions = Expressions::empty();
        let inaccessible = expressions.insert_at_root("inaccessible");
        let ident = "var";

        vars.push();
        vars.define_local(ident, inaccessible);
        assert!(vars.fetch(ident).is_some());
        vars.pop();

        // Here we should have no access to the value via the root.
        assert!(vars.fetch_load(ident).is_none());

        // Here we should have no access to the value via the sibling.
        vars.push();
        assert!(vars.fetch_load(ident).is_none());
    }

    #[test]
    fn declaration_lookup() {
        let mut dec = Declarations::new();
        dec.add("var", [0, 0], 0);
        let root = dec.get("var", [0, 0], ScopeId::root()).unwrap();
        assert_eq!(root, 0.into());
    }

    #[test]
    fn declaration_failed_lookup() {
        let mut dec = Declarations::new();
        dec.add("var", [0], 0);
        let root = dec.get("var", [1, 0], ScopeId::root());
        assert!(root.is_none());
    }

    #[test]
    fn multi_level_declarations() {
        let mut dec = Declarations::new();
        let ident = "var";
        dec.add(ident, [0], 0);
        dec.add(ident, [0, 0], 1);
        dec.add(ident, [0, 0, 0], 2);

        assert_eq!(dec.get(ident, [0], ScopeId::root()).unwrap().0, 0);
        assert_eq!(dec.get(ident, [0, 0], ScopeId::root()).unwrap().0, 1);
        assert_eq!(dec.get(ident, [0, 0, 0, 1, 1], ScopeId::root()).unwrap().0, 2);
    }

    #[test]
    fn unreachable_declaration() {
        let mut dec = Declarations::new();
        dec.add("var", [0, 1], 0);
        assert!(dec.get("var", [0, 0, 1], ScopeId::root()).is_none());
    }

    #[test]
    fn get_inside_boundary() {
        let mut vars = Variables::new();
        let mut expressions = Expressions::empty();
        let one = expressions.insert_at_root(1);
        let two = expressions.insert_at_root(2);
        let three = expressions.insert_at_root(3);

        // Define a variable in the root scope
        _ = vars.define_local("var", one);

        // Create a new unique scope and boundary.
        // * `var` should be inaccessible from within the new scope boundary
        // * `outer_var` should be inaccessible to the root scope
        vars.push_scope_boundary();
        assert!(vars.fetch("var").is_none());
        _ = vars.define_local("var", two);
        _ = vars.define_local("other_var", three);
        assert_eq!(vars.fetch_load("var").unwrap(), two);
        vars.push();
        assert_eq!(vars.fetch_load("other_var").unwrap(), three);
        vars.pop();

        // Return to root scope
        vars.pop_scope_boundary();
        assert_eq!(vars.fetch_load("var").unwrap(), one);
        assert!(vars.fetch("other_var").is_none());
    }
}
