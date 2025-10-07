use std::fs::read_to_string;
use std::ops::Deref;
use std::path::PathBuf;

use anathema_store::slab::{Index, SlabIndex};
use anathema_store::smallmap::SmallMap;
use anathema_store::stack::Stack;
use anathema_store::storage::Storage;

use crate::Lexer;
use crate::blueprints::Blueprint;
use crate::error::{Error, ErrorKind, Result};
use crate::expressions::Expressions;
use crate::statements::eval::Scope;
use crate::statements::parser::Parser;
use crate::statements::{Context, Statements};
use crate::strings::{StringId, Strings};
use crate::token::Tokens;
use crate::variables::Variables;

pub trait ToSourceKind {
    fn to_path(self) -> SourceKind;

    fn to_template(self) -> SourceKind;

    fn to_source_kind(self) -> SourceKind
    where
        Self: Sized,
    {
        self.to_path()
    }
}

impl ToSourceKind for String {
    fn to_path(self) -> SourceKind {
        SourceKind::Path(self.into())
    }

    fn to_template(self) -> SourceKind {
        SourceKind::Str(self)
    }
}

impl ToSourceKind for &str {
    fn to_path(self) -> SourceKind {
        SourceKind::Path(self.into())
    }

    fn to_template(self) -> SourceKind {
        SourceKind::Str(self.into())
    }
}

impl ToSourceKind for PathBuf {
    fn to_path(self) -> SourceKind {
        SourceKind::Path(self)
    }

    fn to_template(self) -> SourceKind {
        panic!("PathBuf can not be a template, only a path to one")
    }
}

pub enum SourceKind {
    Path(PathBuf),
    Str(String),
}

impl ToSourceKind for SourceKind {
    fn to_path(self) -> SourceKind {
        self
    }

    fn to_template(self) -> SourceKind {
        self
    }
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

pub enum TemplateSource {
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AssocEventMapping {
    pub internal: StringId,
    pub external: StringId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ComponentBlueprintId(u32);

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
    dependencies: Stack<ComponentBlueprintId>,
    components: Storage<ComponentBlueprintId, StringId, TemplateSource>,
}

impl ComponentTemplates {
    pub(crate) fn new() -> Self {
        Self {
            dependencies: Stack::empty(),
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
        &mut self,
        component_id: ComponentBlueprintId,
        variables: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
        expressions: &mut Expressions,
    ) -> Result<Vec<Blueprint>> {
        let ticket = self.components.checkout(component_id);
        let (_, component_src) = &*ticket;

        if self.dependencies.contains(&component_id) {
            let path = component_src.path();
            self.components.restore(ticket);
            return Err(Error::new(path, ErrorKind::CircularDependency));
        }

        self.dependencies.push(component_id);

        // NOTE
        // The ticket has to be restored to the component store,
        // this is why the error is returned rather than using `?` on `self.compile`.
        let ret = self.compile(component_src, variables, slots, strings, expressions, component_id);
        self.components.restore(ticket);
        self.dependencies.pop();
        ret
    }

    fn compile(
        &mut self,
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
