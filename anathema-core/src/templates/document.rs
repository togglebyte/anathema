use std::fs::read_to_string;
use std::path::PathBuf;

use anathema_store::smallmap::SmallMap;

use super::blueprints::Blueprint;
use super::components::{ComponentTemplates, SourceKind, TemplateSource};
use super::error::{Error, ErrorKind, Result};
use super::expressions::Expressions;
use super::statements::eval::Scope;
use super::statements::parser::Parser;
use super::statements::{Context, Statements};
use super::strings::Strings;
use super::token::Tokens;
use super::{ComponentBlueprintId, Lexer, Variables};

/// A document containing templates and components
/// ```
/// # use anathema_core::templates::Document;
/// let mut doc = Document::new("text 'I am a widget'");
/// ```
pub struct Document {
    template: TemplateSource,
    pub(crate) strings: Strings,
    pub(crate) expressions: Expressions,
    components: ComponentTemplates,
}

impl Document {
    /// Create a new instance of a document.
    pub fn new(template: impl Into<TemplateSource>) -> Self {
        let template = template.into();
        Self {
            template,
            strings: Strings::new(),
            expressions: Expressions::empty(),
            components: ComponentTemplates::new(),
        }
    }

    #[allow(private_bounds)]
    pub(crate) fn add_component(
        &mut self,
        name: impl Into<String>,
        src: impl Into<SourceKind>,
    ) -> Result<ComponentBlueprintId> {
        let src = src.into();
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

    /// Compile the document to a [`Blueprint`].
    pub fn compile(&mut self, globals: &mut Variables) -> Result<Blueprint> {
        globals.reset_globals();
        self.expressions.clear();

        let tokens = Lexer::new(&self.template, &mut self.strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, self.template.len());
        let parser = Parser::new(tokens, &mut self.strings, &self.template, &mut self.components);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context {
            template: &self.template,
            variables: globals,
            strings: &mut self.strings,
            expressions: &mut self.expressions,
            components: &mut self.components,
            slots: SmallMap::empty(),
            current_component_parent: None,
        };

        let mut blueprints = Scope::new(statements).eval(&mut context)?;
        match blueprints.is_empty() {
            true => Err(Error::no_template(ErrorKind::EmptyTemplate)),
            false => Ok(blueprints.remove(0)),
        }
    }

    /// Get an iterator of all the file paths for all the templates
    pub fn template_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.components.file_paths()
    }

    /// Reload all the templates (that have a file path associated with them)
    pub fn reload_templates(&mut self) -> Result<()> {
        self.components.reload()
    }

    /// Get the file path for a component if it has one
    pub fn get_component_source(&self, component_id: ComponentBlueprintId) -> Option<PathBuf> {
        self.components.path(component_id)
    }
}
