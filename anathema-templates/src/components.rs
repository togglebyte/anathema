use std::fs::read_to_string;
use std::path::PathBuf;

use anathema_store::slab::{Index, SlabIndex};
use anathema_store::smallmap::SmallMap;
use anathema_store::stack::Stack;
use anathema_store::storage::Storage;

use crate::Lexer;
use crate::blueprints::Blueprint;
use crate::error::{Error, Result};
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

impl<T: AsRef<str>> ToSourceKind for T {
    fn to_path(self) -> SourceKind {
        SourceKind::Path(self.as_ref().into())
    }

    fn to_template(self) -> SourceKind {
        SourceKind::Str(self.as_ref().into())
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

pub(crate) enum ComponentSource {
    File { path: PathBuf, template: String },
    InMemory(String),
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
    components: Storage<ComponentBlueprintId, StringId, ComponentSource>,
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

    pub(crate) fn insert(&mut self, ident: StringId, template: ComponentSource) -> ComponentBlueprintId {
        self.components.insert(ident, template)
    }

    pub(crate) fn load(
        &mut self,
        parent_id: ComponentBlueprintId,
        globals: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
    ) -> Result<Vec<Blueprint>> {
        if self.dependencies.contains(&parent_id) {
            return Err(Error::CircularDependency);
        }

        self.dependencies.push(parent_id);

        let ticket = self.components.checkout(parent_id);
        let (_, component_src) = &*ticket;
        let template = match component_src {
            ComponentSource::File { template, .. } => template,
            ComponentSource::InMemory(template) => template,
        };

        // NOTE
        // The ticket has to be restored to the component store,
        // this is why the error is returned rather than using `?` on `self.compile`.
        let ret = self.compile(template, globals, slots, strings, parent_id);

        self.components.restore(ticket);
        self.dependencies.pop();

        ret
    }

    fn compile(
        &mut self,
        template: &str,
        globals: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
        parent: ComponentBlueprintId,
    ) -> Result<Vec<Blueprint>> {
        let tokens = Lexer::new(template, strings).collect::<Result<Vec<_>>>()?;
        let tokens = Tokens::new(tokens, template.len());
        let parser = Parser::new(tokens, strings, template, self);

        let statements = parser.collect::<Result<Statements>>()?;

        let mut context = Context::new(globals, self, strings, slots, Some(parent));

        Scope::new(statements).eval(&mut context)
    }

    pub(crate) fn file_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.components.iter().filter_map(|(_, (_, src))| match src {
            ComponentSource::File { path, .. } => Some(path),
            ComponentSource::InMemory(_) => None,
        })
    }

    pub(crate) fn reload(&mut self) -> std::prelude::v1::Result<(), Error> {
        for (_, component) in self.components.iter_mut() {
            match component {
                ComponentSource::File { path, template } => {
                    *template = read_to_string(path)?;
                }
                ComponentSource::InMemory(_) => (),
            }
        }
        Ok(())
    }

    pub(crate) fn get_component_by_string_id(&self, ident: StringId) -> Option<ComponentBlueprintId> {
        self.components.index_by_key(ident)
    }
}
