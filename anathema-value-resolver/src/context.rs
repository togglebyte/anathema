use anathema_state::States;
use anathema_templates::Globals;

use crate::{scope::Scope, AttributeStorage};

pub struct ResolverCtx<'frame, 'bp> {
    pub(crate) scope: &'frame Scope<'frame, 'bp>,
    pub(crate) globals: &'bp Globals,
    pub(crate) states: &'frame States,
    pub(crate) attributes: &'frame AttributeStorage<'bp>,
}

impl<'frame, 'bp> ResolverCtx<'frame, 'bp> {
    pub fn new(globals: &'bp Globals, scope: &'frame Scope<'frame, 'bp>, states: &'frame States, attributes: &'frame AttributeStorage<'bp>) -> Self {
        Self {
            scope,
            globals,
            states,
            attributes,
        }
    }
}
