//! A TUI library with a custom template language and runtime.
//!
//! See the guide to get stared: https://togglebyte.github.io/anathema-guide/
use anathema_compiler::Variables;
use runtime::ComponentId;
pub use {
    anathema_compiler as compiler,   // compiler
    anathema_frontend as frontend,   // frontend
    anathema_geometry as geometry,   // geometry
    anathema_runtime as runtime,     // runtime
    anathema_state_derive as derive, // derive
    anathema_store as store,         // store
    anathema_widgets as widgets,     // widgets
};

pub mod prelude {
    pub use crate::compiler::{ComponentBlueprintId, Document, SourceKind};
    pub use crate::frontend::Crossterm;
    pub use crate::geometry::{Pos, Region, Size};
    pub use crate::runtime::Runtime;
}

pub mod component {
    pub use crate::compiler::Color;
    pub use crate::runtime::State;
    pub use crate::runtime::value::{List, Map, Maybe, Value};
}

pub struct Anathema {
    doc: compiler::Document,
}

impl Anathema {
    pub fn new(mut doc: compiler::Document) -> Self {
        Self { doc }
    }

    pub fn run(mut self, fe: impl frontend::Frontend) {
        let mut widget_factory = runtime::widgets::RegisteredWidgets::empty();
        widgets::register_default_widgets(&mut widget_factory);
        let mut globals = Variables::new();
        let blueprint = self.doc.compile(&mut globals).unwrap();
        let mut rt = runtime::Runtime::new(self.doc, blueprint, globals, fe, widget_factory);
        rt.run();
    }
}
