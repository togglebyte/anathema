use std::fs::read_to_string;
use std::path::PathBuf;

use anathema_store::smallmap::SmallMap;

use crate::blueprints::Blueprint;
use crate::components::{ComponentTemplates, SourceKind, TemplateSource};
use crate::error::{Error, ErrorKind, Result};
use crate::statements::eval::Scope;
use crate::statements::parser::Parser;
use crate::statements::{Context, Statements};
use crate::strings::Strings;
use crate::token::Tokens;
use crate::{ComponentBlueprintId, Lexer, Variables};

/// A document containing templates and components
/// ```
/// # use anathema_templates::Document;
/// let mut doc = Document::new("text 'I am a widget'");
/// ```
pub struct Document {
    template: TemplateSource,
    pub strings: Strings,
    globals: Variables,
    components: ComponentTemplates,
    pub hot_reload: bool,
}

impl Document {
    pub fn new(template: impl Into<TemplateSource>) -> Self {
        let template = template.into();
        Self {
            template,
            strings: Strings::new(),
            globals: Variables::default(),
            components: ComponentTemplates::new(),
            hot_reload: true,
        }
    }

    #[allow(private_bounds)]
    pub fn add_component(&mut self, name: impl Into<String>, src: SourceKind) -> Result<ComponentBlueprintId> {
        let name = name.into();
        let name = self.strings.push(name);

        let component_src = match src {
            SourceKind::Str(s) => TemplateSource::InMemory(s),
            SourceKind::Path(path) => {
                let template = match read_to_string(&path) {
                    Err(e) => return Err(Error::new(Some(path), e)),
                    Ok(t) => t,
                };
                TemplateSource::File { path, template }
            }
        };

        let id = self.components.insert(name, component_src);
        Ok(id)
    }

    pub fn compile(&mut self) -> Result<(Blueprint, Variables)> {
        self.globals = Variables::default();

        let tokens = Lexer::new(&self.template, &mut self.strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, self.template.len());
        let parser = Parser::new(tokens, &mut self.strings, &self.template, &mut self.components);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context {
            template: &self.template,
            variables: &mut self.globals,
            strings: &mut self.strings,
            components: &mut self.components,
            slots: SmallMap::empty(),
            current_component_parent: None,
        };

        let mut blueprints = Scope::new(statements).eval(&mut context)?;
        match blueprints.is_empty() {
            true => Err(Error::no_template(ErrorKind::EmptyTemplate)),
            false => Ok((blueprints.remove(0), self.globals.take())),
        }
    }

    pub fn template_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.components.file_paths()
    }

    pub fn reload_templates(&mut self) -> Result<()> {
        self.components.reload()
    }

    pub fn get_component_source(&self, component_id: ComponentBlueprintId) -> Option<PathBuf> {
        self.components.path(component_id)
    }
}
