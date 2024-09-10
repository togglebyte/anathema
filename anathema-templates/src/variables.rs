use std::collections::HashMap;
use std::rc::Rc;

#[cfg(not(target_os = "windows"))]
use anathema_debug::DebugWriter;
use anathema_store::slab::Slab;

use crate::expressions::Expression;
use crate::primitives::Primitive;

#[derive(Debug, Default, Clone)]
pub struct Globals(HashMap<Rc<str>, Expression>);

impl Globals {
    pub fn new(hm: HashMap<Rc<str>, Expression>) -> Self {
        Self(hm)
    }

    pub fn get(&self, ident: &str) -> Option<&Expression> {
        self.0.get(ident)
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl From<Variables> for Globals {
    fn from(value: Variables) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VarId(usize);

impl From<usize> for VarId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<VarId> for usize {
    fn from(value: VarId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variable {
    Static(Primitive),
    Str(Rc<str>),
}

impl From<&str> for Variable {
    fn from(value: &str) -> Self {
        Self::Str(value.into())
    }
}

impl From<Primitive> for Variable {
    fn from(value: Primitive) -> Self {
        Self::Static(value)
    }
}

/// The scope id acts as a path made up of indices
/// into the scope tree.
/// E.g `[0, 1, 0]` would point to `root.children[0].children[1].children[0]`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ScopeId(Rc<[u16]>);

impl ScopeId {
    // Create the next child id.
    #[cfg(test)]
    fn next(&self, index: u16) -> Self {
        let mut scope_id = Vec::with_capacity(self.0.len() + 1);
        scope_id.extend_from_slice(&self.0);
        scope_id.push(index);
        Self(scope_id.into())
    }

    // Get the parent id as a slice.
    #[cfg(test)]
    fn parent(&self) -> &[u16] {
        // Can't get the parent of the root
        debug_assert!(self.0.len() > 1);

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

    #[cfg(test)]
    // Does other contain self
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

    // Get the value id "closest" to the given scope id.
    //
    // e.g
    // ident0 @ scope [0]
    // ident1 @ scope [0, 0]
    // ident2 @ scope [0, 1]
    // ident3 @ scope [0, 1, 1]
    //
    // given an id of [0, 1, 1, 2, 3] would find `ident3` as the closest.
    //
    // If there is no value with the given ident within reach
    // then return `None`.
    fn get_var_id(&self, id: impl AsRef<[u16]>, ident: &str) -> Option<VarId> {
        let mut scope = &self.0;
        let mut id = &id.as_ref()[1..];
        let mut var = self.0.variables.get(ident).and_then(|values| values.last()).copied();

        while !id.is_empty() {
            scope = &scope.children[id[0] as usize];
            id = &id[1..];

            if let val @ Some(_) = scope.variables.get(ident).and_then(|values| values.last()).copied() {
                var = val;
            }
        }

        var
    }

    #[cfg(test)]
    fn id(&self) -> &ScopeId {
        &self.0.id
    }

    #[cfg(test)]
    fn insert(&mut self, ident: impl Into<Rc<str>>, var: VarId) {
        self.0.insert(ident.into(), var)
    }

    #[cfg(test)]
    fn create_child(&mut self) -> ScopeId {
        self.0.create_child()
    }
}

/// A scope stores versioned values
#[derive(Debug)]
pub struct Scope {
    variables: HashMap<Rc<str>, Vec<VarId>>,
    id: ScopeId,
    children: Vec<Scope>,
}

impl Scope {
    fn new(id: ScopeId) -> Self {
        Self {
            id,
            variables: Default::default(),
            children: vec![],
        }
    }

    // Create the next child scope id.
    // ```
    // let mut current = ScopeId::from([0]);
    // let next = current.next_scope(); // scope 0,0
    // let next = current.next_scope(); // scope 0,1
    // ```
    #[cfg(test)]
    fn create_child(&mut self) -> ScopeId {
        let index = self.children.len();
        let id = self.id.next(index as u16);
        self.children.push(Scope::new(id.clone()));
        id
    }

    // Every call to `insert` will shadow the previous value, not replace it.
    fn insert(&mut self, ident: impl Into<Rc<str>>, value: VarId) {
        let entry = self.variables.entry(ident.into()).or_default();
        entry.push(value);
    }
}

#[derive(Debug)]
struct Declarations(HashMap<Rc<str>, Vec<(ScopeId, VarId)>>);

impl Declarations {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn add(&mut self, ident: impl Into<Rc<str>>, id: impl Into<ScopeId>, value_id: impl Into<VarId>) {
        let value_id = value_id.into();
        let ids = self.0.entry(ident.into()).or_default();
        ids.push((id.into(), value_id));
    }

    #[cfg(test)]
    // Get the scope id that is closest to the argument
    fn get(&self, ident: &str, id: impl AsRef<[u16]>) -> Option<(&ScopeId, VarId)> {
        self.0
            .get(ident)
            .unwrap()
            .iter()
            .rev()
            .filter_map(|(scope, value)| scope.contains(&id).map(|s| (s, *value)))
            .next()
    }

    #[cfg(test)]
    fn get_ref(&self, ident: &str, id: impl AsRef<[u16]>) -> &[u16] {
        self.get(ident, id).unwrap().0.as_ref()
    }
}

/// Variable access, declaration and assignment
/// during the compilation step.
#[derive(Debug)]
pub struct Variables {
    root: RootScope,
    current: ScopeId,
    store: Slab<VarId, Expression>,
    declarations: Declarations,
}

impl Default for Variables {
    fn default() -> Self {
        let root = RootScope::default();
        Self {
            current: root.0.id.clone(),
            root,
            store: Slab::empty(),
            declarations: Declarations::new(),
        }
    }
}

impl Variables {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    fn declare_at(&mut self, ident: impl Into<Rc<str>>, var_id: VarId, id: ScopeId) -> VarId {
        let ident = ident.into();
        let scope = self.root.get_scope_mut(id);
        scope.insert(ident.clone(), var_id);
        self.declarations.add(ident, scope.id.clone(), var_id);
        var_id
    }

    pub fn declare(&mut self, ident: impl Into<Rc<str>>, value: impl Into<Expression>) -> VarId {
        let value = value.into();
        let var_id = self.store.insert(value);
        let scope_id = self.current.clone();
        self.declare_at(ident, var_id, scope_id)
    }

    /// Fetch a value starting from the current path.
    pub fn fetch(&self, ident: &str) -> Option<Expression> {
        self.root
            .get_var_id(&self.current, ident)
            .and_then(|id| self.store.get(id).cloned())
    }

    /// Create a new child and set the new childs id as the `current` id.
    /// Any operations done from here on out are acting upon the new child scope.
    #[cfg(test)]
    pub(crate) fn push(&mut self) {
        let parent = self.root.get_scope_mut(&self.current);
        self.current = parent.create_child();
    }

    /// Pop the current child scope, making the current into the parent of
    /// the child.
    ///
    /// E.e if the current id is `[0, 1, 2]` `pop` would result in a new
    /// id of `[0, 1]`.
    #[cfg(test)]
    pub(crate) fn pop(&mut self) {
        // panic!("drain and insert phi");
        self.current = self.current.parent().into();
    }

    #[cfg(test)]
    fn by_value_ref(&self, var: VarId) -> Expression {
        self.store
            .get(var)
            .cloned()
            .expect("it would be an Anathema compilation error if this failed")
    }
}

impl From<Variables> for HashMap<Rc<str>, Expression> {
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

#[cfg(not(target_os = "windows"))]
pub struct ScopeDebug<'a> {
    level: usize,
    scope: &'a Scope,
    store: &'a Slab<VarId, Expression>,
}

#[cfg(not(target_os = "windows"))]
impl DebugWriter for ScopeDebug<'_> {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        let indent = " ".repeat(self.level * 4);

        for (key, var_ids) in self.scope.variables.iter() {
            let iter = var_ids.iter().filter_map(|id| self.store.get(*id));

            for (cntr, val) in iter.enumerate() {
                if cntr > 0 {
                    writeln!(output, ", ")?;
                }
                writeln!(output, "{indent}{key}: {val:?}")?;
            }
        }

        for child in &self.scope.children {
            ScopeDebug {
                level: self.level + 1,
                scope: child,
                store: self.store,
            }
            .write(output)?;
        }

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
pub struct VariablesDebug<'a>(pub(crate) &'a Variables);

#[cfg(not(target_os = "windows"))]
impl DebugWriter for VariablesDebug<'_> {
    fn write(&mut self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        ScopeDebug {
            level: 0,
            scope: &self.0.root.0,
            store: &self.0.store,
        }
        .write(output)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        let child_id = root.create_child();
        assert_eq!(root.0.children.len(), 1);
        assert_eq!(child_id.as_ref(), &[0, 0]);
    }

    #[test]
    fn get_value() {
        let expected: VarId = 123.into();

        let mut root = RootScope::default();
        root.insert("var", expected);
        let actual = root.get_var_id(root.id(), "var").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn child_get_value() {
        let expected: VarId = 1.into();
        let ident = "var";

        let mut root = RootScope::default();
        let child_id = root.create_child();
        let child = root.get_scope_mut(&child_id);
        child.insert(ident, expected);
        let actual = root.get_var_id(&child_id, ident).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn variable_declaration() {
        let mut vars = Variables::new();
        let expected = Expression::from(123i64);

        vars.declare("var", expected.clone());
        let value = vars.fetch("var").unwrap();

        assert_eq!(expected, value);
    }

    #[test]
    fn shadow_value() {
        let ident = "var";
        let mut vars = Variables::new();
        let value_a = Expression::from("1");
        let value_b = Expression::from("2");

        let first_value_ref = vars.declare(ident, value_a.clone());
        let second_value_ref = vars.declare(ident, value_b.clone());
        assert_eq!(value_a, vars.by_value_ref(first_value_ref));
        assert_eq!(value_b, vars.by_value_ref(second_value_ref));
    }

    #[test]
    fn scoping_variables_inaccessible_sibling() {
        // Declare a variable in a sibling and fail to access that value
        let mut vars = Variables::new();
        let ident = "var";

        vars.push();
        vars.declare(ident, "inaccessible");
        assert!(vars.fetch(ident).is_some());
        vars.pop();

        // Here we should have no access to the value via the root.
        assert!(vars.fetch(ident).is_none());

        // Here we should have no access to the value via the sibling.
        vars.push();
        assert!(vars.fetch(ident).is_none());
    }

    #[test]
    fn declaration_lookup() {
        let mut dec = Declarations::new();
        dec.add("var", [0], 0);
        let root = dec.get_ref("var", [0, 0]);
        assert_eq!(root, &[0]);
    }

    #[test]
    fn declaration_failed_lookup() {
        let mut dec = Declarations::new();
        dec.add("var", [0], 0);
        let root = dec.get("var", [1, 0]);
        assert!(root.is_none());
    }

    #[test]
    fn multi_level_declarations() {
        let mut dec = Declarations::new();
        let ident = "var";
        dec.add(ident, [0], 0);
        dec.add(ident, [0, 0], 0);
        dec.add(ident, [0, 0, 0], 0);

        assert_eq!(dec.get_ref(ident, [0, 0]), &[0, 0]);
        assert_eq!(dec.get_ref(ident, [0, 0, 0, 1, 1]), &[0, 0, 0]);
    }

    #[test]
    fn unreachable_declaration() {
        let mut dec = Declarations::new();
        dec.add("var", [0, 1], 0);
        assert!(dec.get("var", [0, 0, 1]).is_none());
    }
}
