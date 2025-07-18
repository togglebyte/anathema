//! A TUI library with a custom template language and runtime.
//!
//! See the guide to get stared: https://togglebyte.github.io/anathema-guide/
pub use {
    anathema_backend as backend,                 // backend
    anathema_default_widgets as default_widgets, // default widgets
    anathema_geometry as geometry,               // geometry
    anathema_runtime as runtime,                 // runtime
    anathema_state as state,                     // state
    anathema_state_derive as derive,             // derive
    anathema_store as store,                     // store
    anathema_templates as templates,             // templates
    anathema_value_resolver as resolver,         // resolver
    anathema_widgets as widgets,                 // wigets
};

pub mod prelude {
    pub use crate::backend::Backend;
    pub use crate::backend::tui::TuiBackend;
    pub use crate::runtime::Runtime;
    pub use crate::templates::{ComponentBlueprintId, Document, SourceKind, ToSourceKind};
    pub use crate::widgets::components::MessageReceiver;
}

pub mod component {
    pub use crate::state::{Color, List, Map, Maybe, Nullable, State, Value};
    pub use crate::widgets::components::events::{
        Event, KeyCode, KeyEvent, KeyState, MouseButton, MouseEvent, MouseState,
    };
    pub use crate::widgets::components::{Component, ComponentId, Context, Emitter, UserEvent};
    pub use crate::widgets::query::Children;
}
