use anathema_state::States;
use anathema_templates::Globals;

use crate::scope::Scope;

pub struct ResolverCtx<'frame, 'bp> {
    pub(crate) scopes: &'frame Scope<'frame, 'bp>,
    pub(crate) globals: &'bp Globals,
    pub(crate) states: &'frame States,
}

impl<'frame, 'bp> ResolverCtx<'frame, 'bp> {
    pub fn new(globals: &'bp Globals, scopes: &'frame Scope<'frame, 'bp>, states: &'frame States) -> Self {
        Self {
            scopes,
            globals,
            states,
        }
    }
}
