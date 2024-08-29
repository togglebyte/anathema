use std::fs::read_to_string;
use std::path::PathBuf;

use anathema_store::slab::Index;
use anathema_store::smallmap::SmallMap;
use anathema_store::stack::Stack;
use anathema_store::storage::strings::{StringId, Strings};
use anathema_store::storage::Storage;

use crate::blueprints::Blueprint;
use crate::error::{Error, Result};
use crate::statements::eval::Scope;
use crate::statements::parser::Parser;
use crate::statements::{Context, Statements};
use crate::token::Tokens;
use crate::variables::Variables;
use crate::Lexer;

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
    Empty,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WidgetComponentId(u32);

impl From<WidgetComponentId> for Index {
    fn from(value: WidgetComponentId) -> Self {
        value.0.into()
    }
}

impl From<WidgetComponentId> for usize {
    fn from(value: WidgetComponentId) -> Self {
        value.0 as usize
    }
}

impl From<usize> for WidgetComponentId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

pub(crate) struct ComponentTemplates {
    dependencies: Stack<WidgetComponentId>,
    components: Storage<WidgetComponentId, String, ComponentSource>,
}

impl ComponentTemplates {
    pub(crate) fn new() -> Self {
        Self {
            dependencies: Stack::empty(),
            components: Storage::empty(),
        }
    }

    pub(crate) fn insert_id(&mut self, name: impl Into<String>) -> WidgetComponentId {
        self.components.push(name.into(), ComponentSource::Empty)
    }

    pub(crate) fn insert(&mut self, ident: impl Into<String>, template: ComponentSource) -> WidgetComponentId {
        let ident = ident.into();
        self.components.insert(ident, template)
    }

    pub(crate) fn load(
        &mut self,
        parent_id: WidgetComponentId,
        globals: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
    ) -> Result<Vec<Blueprint>> {
        if self.dependencies.contains(&parent_id) {
            return Err(Error::CircularDependency);
        }

        self.dependencies.push(parent_id);

        let ret = match self.components.remove(parent_id) {
            Some((key, component_src)) => {
                let template = match &component_src {
                    ComponentSource::File { template, .. } => template,
                    ComponentSource::InMemory(template) => template,
                    ComponentSource::Empty => return Err(Error::MissingComponent(key)),
                };
                let ret = self.compile(template, globals, slots, strings, parent_id);
                // This will re-insert the component in the same location
                // as it was removed from since nothing else has
                // written to the component storage since the component
                // was removed.
                let new_id = self.components.insert(key, component_src);
                assert_eq!(parent_id, new_id);
                ret
            }
            _ => unreachable!("a component entry exists if it's mentioned in the template, even if the component it self doesn't exist"),
        };

        self.dependencies.pop();

        ret
    }

    fn compile(
        &mut self,
        template: &str,
        globals: &mut Variables,
        slots: SmallMap<StringId, Vec<Blueprint>>,
        strings: &mut Strings,
        parent: WidgetComponentId,
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
            ComponentSource::Empty => None,
        })
    }

    pub(crate) fn reload(&mut self) -> std::prelude::v1::Result<(), Error> {
        for (_, component) in self.components.iter_mut() {
            match component {
                ComponentSource::File { path, template } => {
                    *template = read_to_string(path)?;
                }
                ComponentSource::InMemory(_) | ComponentSource::Empty => (),
            }
        }
        Ok(())
    }
}
