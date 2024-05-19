use anathema_store::storage::strings::Strings;

use crate::blueprints::Blueprint;
use crate::components::ComponentTemplates;
use crate::error::{Error, Result};
use crate::statements::eval::Scope;
use crate::statements::parser::Parser;
use crate::statements::{Context, Statements};
use crate::token::Tokens;
use crate::variables::Variables;
use crate::{Globals, Lexer};

/// A document containing templates and components
/// ```
/// # use anathema_templates::Document;
/// let mut doc = Document::new("text 'I am a widget'");
/// ```
pub struct Document {
    template: String,
    strings: Strings,
    globals: Variables,
    components: ComponentTemplates,
}

impl Document {
    pub fn new(template: impl Into<String>) -> Self {
        let template = template.into();
        Self {
            template,
            strings: Strings::empty(),
            globals: Variables::new(),
            components: ComponentTemplates::new(),
        }
    }

    pub fn add_component(&mut self, name: impl Into<String>, template: impl Into<String>) -> usize {
        let name = name.into();
        let id = self.components.insert(name, template.into());
        id.into()
    }

    pub fn compile(mut self) -> Result<(Blueprint, Globals)> {
        let tokens = Lexer::new(&self.template, &mut self.strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, self.template.len());
        let parser = Parser::new(tokens, &mut self.strings, &self.template, &mut self.components);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context {
            globals: &mut self.globals,
            strings: &mut self.strings,
            components: &mut self.components,
        };

        let mut blueprints = Scope::new(statements).eval(&mut context)?;
        match blueprints.is_empty() {
            true => Err(Error::EmptyTemplate),
            false => Ok((blueprints.remove(0), self.globals.into())),
        }
    }
}
