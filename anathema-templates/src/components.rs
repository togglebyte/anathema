use anathema_store::stack::Stack;
use anathema_store::storage::strings::Strings;
use anathema_store::storage::Storage;

use crate::blueprints::Blueprint;
use crate::error::{Error, Result};
use crate::statements::eval::Scope;
use crate::statements::parser::Parser;
use crate::statements::{Context, Statements};
use crate::token::Tokens;
use crate::variables::Variables;
use crate::Lexer;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TemplateComponentId(usize);

impl From<TemplateComponentId> for usize {
    fn from(value: TemplateComponentId) -> Self {
        value.0
    }
}

impl From<usize> for TemplateComponentId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub(crate) struct ComponentTemplates {
    dependencies: Stack<TemplateComponentId>,
    components: Storage<TemplateComponentId, String, Option<String>>,
}

impl ComponentTemplates {
    pub(crate) fn new() -> Self {
        Self {
            dependencies: Stack::empty(),
            components: Storage::empty(),
        }
    }

    pub(crate) fn insert_id(&mut self, name: impl Into<String>) -> TemplateComponentId {
        self.components.push(name.into(), None)
    }

    pub(crate) fn insert(&mut self, ident: impl Into<String>, template: impl Into<String>) -> TemplateComponentId {
        let ident = ident.into();
        self.components.insert(ident, Some(template.into()))
    }

    pub(crate) fn load(&mut self, id: TemplateComponentId, globals: &mut Variables) -> Result<Vec<Blueprint>> {
        if self.dependencies.contains(&id) {
            return Err(Error::CircularDependency);
        }

        self.dependencies.push(id);

        let ret = match self.components.remove(id) {
            Some((key, Some(template))) => {
                let ret = self.compile(&template, globals);
                // This will re-insert the component in the same location
                // as it was removed from since nothing else has
                // written to the component storage since the component
                // was removed.
                let new_id = self.components.insert(key, Some(template));
                assert_eq!(id, new_id);
                ret
            }
            _ => return Err(Error::MissingComponent),
        };

        self.dependencies.pop();

        ret
    }

    fn compile(&mut self, template: &str, globals: &mut Variables) -> Result<Vec<Blueprint>> {
        let mut strings = Strings::empty();
        let tokens = Lexer::new(template, &mut strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, template.len());
        let parser = Parser::new(tokens, &mut strings, template, self);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context {
            globals,
            components: self,
            strings: &strings,
        };

        Scope::new(statements).eval(&mut context)
    }
}
