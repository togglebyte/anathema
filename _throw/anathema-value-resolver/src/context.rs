use anathema_state::States;
use anathema_templates::Variables;
use anathema_templates::expressions::Expressions;

use crate::AttributeStorage;
use crate::functions::{Function, FunctionTable};
use crate::scope::Scope;

pub struct ResolverCtx<'frame, 'bp> {
    pub(crate) scope: &'frame Scope<'frame, 'bp>,
    pub(crate) variables: &'frame Variables,
    pub(crate) states: &'frame States,
    pub(crate) attribute_storage: &'frame AttributeStorage<'bp>,
    pub(crate) function_table: &'bp FunctionTable,
    pub(crate) expressions: &'bp Expressions,
}

impl<'frame, 'bp> ResolverCtx<'frame, 'bp> {
    pub fn new(
        variables: &'frame Variables,
        scope: &'frame Scope<'frame, 'bp>,
        states: &'frame States,
        attribute_storage: &'frame AttributeStorage<'bp>,
        function_table: &'bp FunctionTable,
        expressions: &'bp Expressions,
    ) -> Self {
        Self {
            scope,
            variables,
            states,
            attribute_storage,
            function_table,
            expressions,
        }
    }

    pub(crate) fn lookup_function(&self, ident: &str) -> Option<&'bp Function> {
        self.function_table.lookup(ident)
    }
}
