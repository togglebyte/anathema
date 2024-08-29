use std::fs::read_to_string;
use std::path::PathBuf;

use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::Strings;

use crate::blueprints::Blueprint;
use crate::components::{ComponentSource, ComponentTemplates, SourceKind};
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
    pub strings: Strings,
    globals: Variables,
    components: ComponentTemplates,
    pub hot_reload: bool,
}

impl Document {
    pub fn new(template: impl Into<String>) -> Self {
        let template = template.into();
        Self {
            template,
            strings: Strings::empty(),
            globals: Variables::default(),
            components: ComponentTemplates::new(),
            hot_reload: true,
        }
    }

    #[allow(private_bounds)]
    pub fn add_component(&mut self, name: impl Into<String>, src: SourceKind) -> Result<usize> {
        let name = name.into();

        let component_src = match src {
            SourceKind::Str(s) => ComponentSource::InMemory(s),
            SourceKind::Path(path) => {
                let template = read_to_string(&path)?;
                ComponentSource::File { path, template }
            }
        };

        let id = self.components.insert(name, component_src);
        Ok(id.into())
    }

    pub fn compile(&mut self) -> Result<(Blueprint, Globals)> {
        self.strings = Strings::empty();
        self.globals = Variables::default();

        let tokens = Lexer::new(&self.template, &mut self.strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, self.template.len());
        let parser = Parser::new(tokens, &mut self.strings, &self.template, &mut self.components);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context {
            globals: &mut self.globals,
            strings: &mut self.strings,
            components: &mut self.components,
            slots: SmallMap::empty(),
            current_component_parent: None,
        };

        let mut blueprints = Scope::new(statements).eval(&mut context)?;
        match blueprints.is_empty() {
            true => Err(Error::EmptyTemplate),
            false => Ok((blueprints.remove(0), self.globals.take().into())),
        }
    }

    pub fn template_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.components.file_paths()
    }

    pub fn reload_templates(&mut self) -> Result<()> {
        self.components.reload()
    }
}
