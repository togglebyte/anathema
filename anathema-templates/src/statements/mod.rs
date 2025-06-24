use anathema_store::smallmap::SmallMap;

use crate::ComponentBlueprintId;
use crate::blueprints::Blueprint;
use crate::components::{AssocEventMapping, ComponentTemplates};
use crate::error::Result;
use crate::expressions::Expression;
use crate::strings::{StringId, Strings};
use crate::variables::Variables;

mod const_eval;
pub(crate) mod eval;
pub(super) mod parser;

pub(crate) struct Context<'vars> {
    pub(crate) globals: &'vars mut Variables,
    pub(crate) components: &'vars mut ComponentTemplates,
    pub(crate) strings: &'vars mut Strings,
    pub(crate) slots: SmallMap<StringId, Vec<Blueprint>>,
    pub(crate) current_component_parent: Option<ComponentBlueprintId>,
}

impl<'vars> Context<'vars> {
    pub fn new(
        globals: &'vars mut Variables,
        components: &'vars mut ComponentTemplates,
        strings: &'vars mut Strings,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        current_component_parent: Option<ComponentBlueprintId>,
    ) -> Self {
        Self {
            globals,
            components,
            strings,
            slots,
            current_component_parent,
        }
    }
}

impl Context<'_> {
    pub fn component_parent(&self) -> Option<ComponentBlueprintId> {
        self.current_component_parent
    }

    fn fetch(&self, key: &str) -> Option<Expression> {
        self.globals.fetch(key)
    }

    fn load_component(
        &mut self,
        parent_component_id: ComponentBlueprintId,
        slots: SmallMap<StringId, Vec<Blueprint>>,
    ) -> Result<Vec<Blueprint>> {
        self.components
            .load(parent_component_id, self.globals, slots, self.strings)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Statement {
    LoadValue(Expression),
    LoadAttribute { key: StringId, value: Expression },
    AssociatedFunction(AssocEventMapping),
    Component(ComponentBlueprintId),
    ComponentSlot(StringId),
    Node(StringId),
    For { binding: StringId, data: Expression },
    With { binding: StringId, data: Expression },
    Declaration { binding: StringId, value: Expression },
    If(Expression),
    Switch(Expression),
    Case(Expression),
    Default,
    Else(Option<Expression>),
    ScopeStart,
    ScopeEnd,
    Eof,
}

#[derive(Debug, PartialEq)]
pub(crate) struct Statements(Vec<Statement>);

impl From<Vec<Statement>> for Statements {
    fn from(value: Vec<Statement>) -> Self {
        Self(value)
    }
}

impl FromIterator<Statement> for Statements {
    fn from_iter<T: IntoIterator<Item = Statement>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl Statements {
    fn next(&mut self) -> Option<Statement> {
        match self.is_empty() {
            false => Some(self.0.remove(0)),
            true => None,
        }
    }

    fn take_value(&mut self) -> Option<Expression> {
        match matches!(&self.0.first(), Some(Statement::LoadValue(_))) {
            true => match self.0.remove(0) {
                Statement::LoadValue(expr) => Some(expr),
                _ => unreachable!(),
            },
            false => None,
        }
    }

    fn take_attributes(&mut self) -> Vec<(StringId, Expression)> {
        let mut v = vec![];
        while matches!(&self.0.first(), Some(Statement::LoadAttribute { .. })) {
            match self.0.remove(0) {
                Statement::LoadAttribute { key, value } => v.push((key, value)),
                _ => unreachable!(),
            }
        }
        v
    }

    fn take_assoc_functions(&mut self) -> Vec<AssocEventMapping> {
        let mut v = vec![];
        while matches!(&self.0.first(), Some(Statement::AssociatedFunction { .. })) {
            match self.0.remove(0) {
                Statement::AssociatedFunction(map) => v.push(map),
                _ => unreachable!(),
            }
        }
        v
    }

    fn take_scope(&mut self) -> Statements {
        if self.is_empty() {
            return vec![].into();
        }

        if self.0[0] != Statement::ScopeStart {
            return vec![].into();
        }

        let mut level = 0;

        for i in 0..self.0.len() {
            match &self.0[i] {
                Statement::ScopeStart => level += 1,
                Statement::ScopeEnd if level - 1 == 0 => {
                    let mut scope = self.0.split_off(i);
                    scope.remove(0); // remove the scope start
                    std::mem::swap(&mut scope, &mut self.0);
                    scope.remove(0); // remove the scope end
                    return scope.into();
                }
                Statement::ScopeEnd => level -= 1,
                _ => continue,
            }
        }

        panic!("Unclosed scope. this is a bug with Anathema and should be reported");
    }

    fn take_until_case_or_default(&mut self) -> Statements {
        if self.is_empty() {
            return vec![].into();
        }

        let mut statements = vec![];

        while !self.0.is_empty() {
            match self.0[0] {
                Statement::Case(_) | Statement::Default | Statement::Eof => break,
                _ => {}
            }
            statements.push(self.0.remove(0));
        }

        statements.into()
    }

    fn next_else(&mut self) -> Option<Option<Expression>> {
        match matches!(self.0.first(), Some(Statement::Else(_))) {
            true => match self.0.remove(0) {
                Statement::Else(cond) => Some(cond),
                _ => unreachable!(),
            },
            false => None,
        }
    }

    fn next_case(&mut self) -> Option<Expression> {
        match matches!(self.0.first(), Some(Statement::Case(_))) {
            true => match self.0.remove(0) {
                Statement::Case(cond) => Some(cond),
                _ => unreachable!(),
            },
            false => None,
        }
    }

    fn next_default(&mut self) -> bool {
        match matches!(self.0.first(), Some(Statement::Default)) {
            true => {
                _ = self.0.remove(0);
                true
            }
            false => false,
        }
    }

    fn next_slot(&mut self) -> Option<StringId> {
        match matches!(self.0.first(), Some(Statement::ComponentSlot(_))) {
            true => match self.0.remove(0) {
                Statement::ComponentSlot(slot_id) => Some(slot_id),
                _ => unreachable!(),
            },
            false => None,
        }
    }

    fn is_next_slot(&mut self) -> bool {
        matches!(self.0.first(), Some(Statement::ComponentSlot(_)))
    }

    fn is_next_scope(&mut self) -> bool {
        matches!(self.0.first(), Some(Statement::ScopeStart))
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// This is useful for testing
#[cfg(test)]
fn with_context<F>(mut f: F)
where
    F: FnMut(Context<'_>),
{
    let mut globals = Variables::new();
    let mut strings = Strings::new();
    let mut components = ComponentTemplates::new();

    let context = Context {
        globals: &mut globals,
        strings: &mut strings,
        components: &mut components,
        slots: SmallMap::empty(),
        current_component_parent: None,
    };

    f(context)
}

// -----------------------------------------------------------------------------
//   - Statements -
// -----------------------------------------------------------------------------
#[cfg(test)]
mod test {
    use anathema_store::slab::SlabIndex;

    use super::*;

    pub(crate) fn load_value(expr: impl Into<Expression>) -> Statement {
        Statement::LoadValue(expr.into())
    }

    pub(crate) fn load_attrib(key: usize, expr: impl Into<Expression>) -> Statement {
        let key = StringId::from_usize(key);
        Statement::LoadAttribute {
            key,
            value: expr.into(),
        }
    }

    pub(crate) fn component(id: u32) -> Statement {
        let comp_id = ComponentBlueprintId::from(id);
        Statement::Component(comp_id)
    }

    pub(crate) fn slot(id: usize) -> Statement {
        let id = StringId::from_usize(id);
        Statement::ComponentSlot(id)
    }

    pub(crate) fn associated_fun(internal: usize, external: usize) -> Statement {
        Statement::AssociatedFunction(AssocEventMapping {
            internal: internal.into(),
            external: external.into(),
        })
    }

    pub(crate) fn node(id: usize) -> Statement {
        let id = SlabIndex::from_usize(id);
        Statement::Node(id)
    }

    pub(crate) fn for_loop(binding: usize, data: impl Into<Expression>) -> Statement {
        let binding = StringId::from_usize(binding);
        Statement::For {
            binding,
            data: data.into(),
        }
    }

    pub(crate) fn with(binding: usize, data: impl Into<Expression>) -> Statement {
        let binding = StringId::from_usize(binding);
        Statement::With {
            binding,
            data: data.into(),
        }
    }

    pub(crate) fn decl(binding: usize, value: impl Into<Expression>) -> Statement {
        let binding = StringId::from_usize(binding);
        Statement::Declaration {
            binding,
            value: value.into(),
        }
    }

    pub(crate) fn if_stmt(cond: impl Into<Expression>) -> Statement {
        Statement::If(cond.into())
    }

    pub(crate) fn switch(cond: impl Into<Expression>) -> Statement {
        Statement::Switch(cond.into())
    }

    pub(crate) fn case(cond: impl Into<Expression>) -> Statement {
        Statement::Case(cond.into())
    }

    pub(crate) fn if_else(cond: impl Into<Expression>) -> Statement {
        Statement::Else(Some(cond.into()))
    }

    pub(crate) fn else_stmt() -> Statement {
        Statement::Else(None)
    }

    pub(crate) fn scope_start() -> Statement {
        Statement::ScopeStart
    }

    pub(crate) fn scope_end() -> Statement {
        Statement::ScopeEnd
    }

    pub(crate) fn eof() -> Statement {
        Statement::Eof
    }

    #[test]
    pub(crate) fn split_scope() {
        let mut statements = Statements::from_iter([scope_start(), node(0), scope_end(), node(1)]);
        let scope = statements.take_scope();
        assert_eq!(Statements::from_iter([node(0)]), scope);
        assert_eq!(Statements::from_iter([node(1)]), statements);
    }
}
