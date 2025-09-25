use anathema_state::States;
use anathema_templates::expressions::Expressions;
use anathema_templates::VariableStorage;

use crate::expression::ResolvedExpressions;
use crate::AttributeStorage;
use crate::functions::{Function, FunctionTable};
use crate::scope::Scope;

pub struct ResolverCtx<'frame, 'bp> {
    pub(crate) scope: &'frame Scope<'frame, 'bp>,
    pub(crate) variables: &'bp VariableStorage,
    pub(crate) states: &'frame States,
    pub(crate) attribute_storage: &'frame AttributeStorage<'bp>,
    pub(crate) function_table: &'bp FunctionTable,
    pub(crate) expressions: &'bp Expressions,
    pub(crate) resolved_expressions: &'frame mut ResolvedExpressions<'bp>,
}

impl<'frame, 'bp> ResolverCtx<'frame, 'bp> {
    pub fn new(
        variables: &'bp VariableStorage,
        scope: &'frame Scope<'frame, 'bp>,
        states: &'frame States,
        attribute_storage: &'frame AttributeStorage<'bp>,
        function_table: &'bp FunctionTable,
        expressions: &'bp Expressions,
        resolved_expressions: &'frame mut ResolvedExpressions<'bp>,
    ) -> Self {
        Self {
            scope,
            variables,
            states,
            attribute_storage,
            function_table,
            expressions,
            resolved_expressions,
        }
    }

    pub(crate) fn lookup_function(&self, ident: &str) -> Option<&'bp Function> {
        self.function_table.lookup(ident)
    }
}
