use std::cell::RefCell;
use std::fs::read_to_string;
use std::ops::Deref;
use std::path::PathBuf;

use anathema_store::slab::{Index, SlabIndex};
use anathema_store::smallmap::SmallMap;
use anathema_store::stack::Stack;
use anathema_store::storage::Storage;

use super::lexer::Lexer;
use super::blueprints::Blueprint;
use super::error::{Error, ErrorKind, Result};
use super::expressions::Expressions;
use super::statements::eval::Scope;
use super::statements::parser::Parser;
use super::statements::{Context, Statements};
use super::strings::{StringId, Strings};
use super::token::Tokens;
use super::variables::Variables;

/// Template source.
/// For hot reloading this has to be a `Path`.
pub enum SourceKind {
    /// A path to a file
    Path(PathBuf),
    /// The template as a string
    Str(String),
}

impl From<PathBuf> for SourceKind {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl From<String> for SourceKind {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<&str> for SourceKind {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

/// The template source used by the template compiler.
pub(crate) enum TemplateSource {
    File { path: PathBuf, template: String },
    InMemory(String),
    Static(&'static str),
}

impl TemplateSource {
    pub(crate) fn template(&self) -> &str {
        match self {
            Self::File { template, .. } | Self::InMemory(template) => template.as_str(),
            Self::Static(template) => template,
        }
    }

    pub(crate) fn path(&self) -> Option<PathBuf> {
        match self {
            TemplateSource::File { path, .. } => Some(path.clone()),
            TemplateSource::InMemory(_) => None,
            TemplateSource::Static(_) => None,
        }
    }
}

impl Deref for TemplateSource {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.template()
    }
}

impl From<&'static str> for TemplateSource {
    fn from(template: &'static str) -> Self {
        Self::Static(template)
    }
}

impl From<String> for TemplateSource {
    fn from(template: String) -> Self {
        Self::InMemory(template)
    }
}

/// An associated event mapping maps the internal name to the external name.
///
/// The following example maps the "press" event to "submit".
/// ```text
/// @button (press -> submit)
/// ```
///
/// When the button publishes the "press" event it can be found as "submit".
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AssocEventMapping {
    /// The name of the event used by the issuing component
    pub internal: StringId,
    /// The public event name used by components to catch the event.
    pub external: StringId,
}

/// This is used by both the templates and the runtime to identify components.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentBlueprintId(u32);

impl ComponentBlueprintId {
    /// This should never be used for anything other than testing.
    pub const ZERO: Self = Self(0);
}

impl SlabIndex for ComponentBlueprintId {
    const MAX: usize = u32::MAX as usize;

    fn as_usize(&self) -> usize {
        self.0 as usize
    }

    fn from_usize(index: usize) -> Self
    where
        Self: Sized,
    {
        Self(index as u32)
    }
}

#[cfg(test)]
impl From<u32> for ComponentBlueprintId {
    fn from(value: u32) -> Self {
        Self::from_usize(value as usize)
    }
}

impl From<ComponentBlueprintId> for Index {
    fn from(value: ComponentBlueprintId) -> Self {
        Index::from(value.0.as_usize() as u32)
    }
}

impl From<ComponentBlueprintId> for usize {
    fn from(value: ComponentBlueprintId) -> Self {
        value.as_usize()
    }
}

impl From<usize> for ComponentBlueprintId {
    fn from(value: usize) -> Self {
        Self::from_usize(value)
    }
}

pub(crate) struct ComponentTemplates {
    dependencies: RefCell<Stack<ComponentBlueprintId>>,
    components: Storage<ComponentBlueprintId, StringId, TemplateSource>,
}

impl ComponentTemplates {
    pub(crate) fn new() -> Self {
        Self {
            dependencies: RefCell::new(Stack::empty()),
            components: Storage::empty(),
        }
    }

    pub(crate) fn name(&self, blueprint_id: ComponentBlueprintId) -> StringId {
        let (k, _) = self
            .components
            .get(blueprint_id)
            .expect("if a component is registered it has a name");
        *k
    }

    pub(crate) fn path(&self, blueprint_id: ComponentBlueprintId) -> Option<PathBuf> {
        self.components.get(blueprint_id).and_then(|(_, comp)| match comp {
            TemplateSource::File { path, .. } => Some(path.clone()),
            TemplateSource::InMemory(_) | TemplateSource::Static(_) => None,
        })
    }

    pub(crate) fn insert(&mut self, ident: StringId, template: TemplateSource) -> ComponentBlueprintId {
        self.components.insert(ident, template)
    }

    pub(crate) fn load(
        &self,
        component_id: ComponentBlueprintId,
        variables: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
        expressions: &mut Expressions,
    ) -> Result<Vec<Blueprint>> {
        let component_src = &self.components[component_id].1;

        if self.dependencies.borrow().contains(&component_id) {
            let path = component_src.path();
            return Err(Error::new(path, ErrorKind::CircularDependency));
        }

        self.dependencies.borrow_mut().push(component_id);

        // NOTE
        // The ticket has to be restored to the component store,
        // this is why the error is returned rather than using `?` on `self.compile`.
        let ret = self.compile(component_src, variables, slots, strings, expressions, component_id);
        self.dependencies.borrow_mut().pop();
        ret
    }

    fn compile(
        &self,
        template: &TemplateSource,
        variables: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
        expressions: &mut Expressions,
        parent: ComponentBlueprintId,
    ) -> Result<Vec<Blueprint>> {
        let tokens = Lexer::new(template, strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, template.len());
        let parser = Parser::new(tokens, strings, template, self);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context::new(template, variables, self, strings, expressions, slots, Some(parent));

        Scope::new(statements).eval(&mut context)
    }

    pub(crate) fn file_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.components.iter().filter_map(|(_, (_, src))| match src {
            TemplateSource::File { path, .. } => Some(path),
            TemplateSource::InMemory(_) | TemplateSource::Static(_) => None,
        })
    }

    pub(crate) fn reload(&mut self) -> Result<()> {
        for (_, component) in self.components.iter_mut() {
            match component {
                TemplateSource::File { path, template } => {
                    *template = match read_to_string(&*path) {
                        Ok(template) => template,
                        Err(e) => return Err(Error::new(Some(path.clone()), ErrorKind::Io(e))),
                    }
                }
                TemplateSource::InMemory(_) | TemplateSource::Static(_) => (),
            }
        }
        Ok(())
    }

    pub(crate) fn get_component_by_string_id(&self, ident: StringId) -> Option<ComponentBlueprintId> {
        self.components.index_by_key(ident)
    }
}
