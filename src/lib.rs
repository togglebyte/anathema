//! A TUI library with a custom template language and runtime.
//!
//! See the guide to get stared: https://togglebyte.github.io/anathema-guide/
pub use {
    anathema_core as core,           // core
    anathema_geometry as geometry,   // geometry
    anathema_state as state,         // state
    anathema_state_derive as derive, // derive
    anathema_store as store,         // store
};

pub mod prelude {
    pub use crate::core::templates::{ComponentBlueprintId, Document, SourceKind};
}

pub mod component {
    pub use crate::state::{Color, List, Map, Maybe, Nullable, State, Value};
}
