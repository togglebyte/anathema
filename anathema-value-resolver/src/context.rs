use anathema_state::States;
use anathema_templates::Globals;

use crate::scope::Scope;
use crate::AttributeStorage;

pub struct ResolverCtx<'frame, 'bp> {
    pub(crate) scope: &'frame Scope<'frame, 'bp>,
    pub(crate) globals: &'bp Globals,
    pub(crate) states: &'frame States,
    pub(crate) attribute_storage: &'frame AttributeStorage<'bp>,
}

impl<'frame, 'bp> ResolverCtx<'frame, 'bp> {
    pub fn new(
        globals: &'bp Globals,
        scope: &'frame Scope<'frame, 'bp>,
        states: &'frame States,
        attribute_storage: &'frame AttributeStorage<'bp>,
    ) -> Self {
        Self {
            scope,
            globals,
            states,
            attribute_storage,
        }
    }
}
