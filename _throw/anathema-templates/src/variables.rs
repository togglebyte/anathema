use std::collections::HashMap;
use std::sync::OnceLock;

use anathema_store::slab::{Slab, SlabIndex};

use crate::error::ErrorKind;
use crate::expressions::{Expression, ExpressionId, Expressions};

#[derive(Debug, Copy, Clone)]
enum Global {
    // The global value was set from the runtime
    Runtime(ExpressionId),
    // The global value originates from a template
    Template(ExpressionId),
}

#[derive(Debug, Default, Clone)]
pub struct Globals(HashMap<String, Global>);

impl Globals {
    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn contains(&self, ident: &str) -> bool {
        self.0.contains_key(ident)
    }

    pub fn get(&self, ident: &str) -> Option<ExpressionId> {
        match self.0.get(ident).copied()? {
            Global::Runtime(expression) | Global::Template(expression) => Some(expression),
        }
    }

    fn set(&mut self, ident: String, value: Global) {
        if self.0.contains_key(&ident) {
            return;
        }
        _ = self.0.insert(ident, value);
    }

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

#[derive(Debug, Clone)]
pub enum Variable {
    /// A variable is defined but the value will be available at runtime, e.g `for-loops` and
    /// `with`
    Definition(ExpressionId),
    /// A value is declared, either as a local value or a global value
    Declaration(ExpressionId),
}

impl Variable {
    fn as_expression(&self) -> ExpressionId {
        match self {
            Variable::Definition(expr) | Variable::Declaration(expr) => *expr,
        }
    }
}

/// The scope id acts as a path made up of indices
/// into the scope tree.
/// E.g `[0, 1, 0]` would point to `root.children[0].children[1].children[0]`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ScopeId(Box<[u16]>);

impl ScopeId {
    pub(crate) fn root() -> &'static Self {
        static ROOT: OnceLock<ScopeId> = OnceLock::new();
        ROOT.get_or_init(|| ScopeId(Box::new([])))
    }

    // Create the next child id.
    fn next(&self, index: u16) -> Self {
        let mut scope_id = Vec::with_capacity(self.0.len() + 1);
        scope_id.extend_from_slice(&self.0);
        scope_id.push(index);
        Self(scope_id.into())
    }

    // Get the parent id as a slice.
    fn parent(&self) -> &[u16] {
        // Can't get the parent of the root
        assert!(self.0.len() > 1);

        let to = self.0.len() - 1;
        &self.0[..to]
    }

    // Check if either `id` or `self` is a sub path of the other.
    // If it is, return the length of the shortest of the two.
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

/// A scope stores versioned values
#[derive(Debug)]
pub struct Scope {
    id: ScopeId,
    children: Vec<Scope>,
}

impl Scope {
    fn new(id: ScopeId) -> Self {
        Self { id, children: vec![] }
    }

    // Create the next child scope id.
    // ```
    // let mut current = ScopeId::from([0]);
    // let next = current.next_scope(); // scope 0,0
    // let next = current.next_scope(); // scope 0,1
    // ```
    fn create_child(&mut self) -> ScopeId {
        let index = self.children.len();
        let id = self.id.next(index as u16);
        self.children.push(Scope::new(id.clone()));
        id
    }
}

#[derive(Debug)]
struct Declarations(HashMap<String, Vec<(ScopeId, VarId)>>);

impl Declarations {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn add(&mut self, ident: impl Into<String>, scope_id: impl Into<ScopeId>, value_id: impl Into<VarId>) {
        let value_id = value_id.into();
        let ids = self.0.entry(ident.into()).or_default();
        ids.push((scope_id.into(), value_id));
    }

    // Get the scope id that is closest to the argument
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

/// Variable access, declaration and assignment
/// during the compilation step.
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

    /// Reset the globals defined in the template.
    /// This keeps any globals registered through Rust
    pub fn reset_globals(&mut self) {
        self.globals.clear_template_globals();
    }

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

    pub fn define_global(&mut self, ident: impl Into<String>, expression: ExpressionId) -> Result<(), ErrorKind> {
        let global = Global::Template(expression);
        self.set_global(ident, global)
    }

    pub fn define_local(&mut self, ident: impl Into<String>, value: ExpressionId) -> VarId {
        let scope_id = self.current.clone();
        let var_id = self.store.insert(Variable::Declaration(value));
        self.declare_at(ident, var_id, scope_id)
    }

    pub fn declare_local(&mut self, ident: impl Into<String>, expressions: &mut Expressions) -> VarId {
        let scope_id = self.current.clone();
        let ident = ident.into();
        let id = expressions.insert(Expression::Ident(ident.clone()), scope_id.clone());
        let value = Variable::Definition(id);
        let var_id = self.store.insert(value);
        self.declare_at(ident, var_id, scope_id)
    }

    /// Fetch a value starting from the current path.
    pub fn fetch(&self, ident: &str) -> Option<VarId> {
        self.declarations.get(ident, &self.current, self.boundary_ref())
    }

    /// Create a new scope and set that scope as a boundary.
    /// This prevents inner components from accessing values
    /// declared outside of the component.
    pub(crate) fn push_scope_boundary(&mut self) {
        self.push();
        self.boundary.push(self.current.clone());
    }

    /// Pop the scope boundary.
    pub(crate) fn pop_scope_boundary(&mut self) {
        self.pop();
        self.boundary.pop();
    }

    /// Create a new child and set the new child's id as the `current` id.
    /// Any operations done from here on out are acting upon the new child scope.
    pub(crate) fn push(&mut self) {
        let parent = self.root.get_scope_mut(&self.current);
        self.current = parent.create_child();
    }

    /// Pop the current child scope, making the current into the parent of
    /// the child.
    ///
    /// E.e if the current id is `[0, 1, 2]` `pop` would result in a new
    /// id of `[0, 1]`.
    pub(crate) fn pop(&mut self) {
        self.current = self.current.parent().into();
    }

    /// Load a variable from the store
    pub fn load(&self, var: VarId) -> Option<ExpressionId> {
        self.store.get(var).map(Variable::as_expression)
    }

    // Fetch and load a value from its ident
    #[cfg(test)]
    fn fetch_load(&self, ident: &str) -> Option<ExpressionId> {
        let id = self.declarations.get(ident, &self.current, self.boundary_ref())?;
        self.load(id)
    }

    pub fn global_lookup(&self, ident: &str) -> Option<ExpressionId> {
        self.globals.get(ident)
    }

    fn boundary_ref(&self) -> &ScopeId {
        self.boundary.last().unwrap_or(ScopeId::root())
    }

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
