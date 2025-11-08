//! A TUI library with a custom template language and runtime.
//!
//! See the guide to get stared: https://togglebyte.github.io/anathema-guide/
pub use {
    anathema_compiler as compiler,   // compiler
    anathema_core as core,           // core
    anathema_geometry as geometry,   // geometry
    anathema_runtime as runtime,     // runtime
    anathema_state_derive as derive, // derive
    anathema_store as store,         // store
    anathema_frontend as frontend,   // frontend
};

pub mod prelude {
    pub use crate::compiler::{ComponentBlueprintId, Document, SourceKind};
}

pub mod component {
    pub use crate::compiler::Color;
    pub use crate::runtime::value::{List, Map, Maybe, Value};
    pub use crate::runtime::State;
}
