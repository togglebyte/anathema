pub use {
    anathema_backend as backend,                 // backend
    anathema_debug as debug,                     // debug
    anathema_default_widgets as default_widgets, // default widgets
    anathema_geometry as geometry,               // geometry
    anathema_runtime as runtime,                 // runtime
    anathema_state as state,                     // state
    anathema_state_derive as derive,             // derive
    anathema_store as store,                     // store
    anathema_templates as templates,             // templates
    anathema_widgets as widgets,                 // wigets
};

pub mod prelude {
    pub use crate::backend::tui::TuiBackend;
    pub use crate::runtime::Runtime;
    pub use crate::templates::{Document, SourceKind, ToSourceKind, WidgetComponentId};
    pub use crate::widgets::components::Context;
}

pub mod component {
    pub use crate::state::{CommonVal, List, Map, State, Value};
    pub use crate::widgets::components::events::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseState};
    pub use crate::widgets::components::{Component, ComponentId, Emitter};
    pub use crate::widgets::Elements;
}
