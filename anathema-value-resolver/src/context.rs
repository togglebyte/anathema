use anathema_state::States;
use anathema_templates::Globals;

use crate::AttributeStorage;
use crate::functions::{Function, FunctionTable};
use crate::scope::Scope;

pub struct ResolverCtx<'frame, 'bp> {
    pub(crate) scope: &'frame Scope<'frame, 'bp>,
    pub(crate) globals: &'bp Globals,
    pub(crate) states: &'frame States,
    pub(crate) attribute_storage: &'frame AttributeStorage<'bp>,
    pub(crate) function_table: &'bp FunctionTable,
}

impl<'frame, 'bp> ResolverCtx<'frame, 'bp> {
    pub fn new(
        globals: &'bp Globals,
        scope: &'frame Scope<'frame, 'bp>,
        states: &'frame States,
        attribute_storage: &'frame AttributeStorage<'bp>,
        function_table: &'bp FunctionTable,
    ) -> Self {
        Self {
            scope,
            globals,
            states,
            attribute_storage,
            function_table,
        }
    }

    pub(crate) fn lookup_function(&self, ident: &str) -> Option<&'bp Function> {
        self.function_table.lookup(ident)
    }
}
