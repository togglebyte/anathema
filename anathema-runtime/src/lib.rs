// -----------------------------------------------------------------------------
//   - Runtime -
//   1. Creating the initial widget tree
//   2. Runtime loop >----------------------------------------+
//    ^  2.1. Wait for messages                               |
//    |  2.2. Wait for events                                 v
//    |  2.4. Was there events / messages / data changes? (no) (yes)
//    |                                                    |    |
//    +----------------------------------------------------+    |
//    |       +-------------------------------------------------+
//    |       |
//    |       V
//    |       1. Layout
//    |       2. Position
//    |       3. Draw
//    +-----< 4. Run again
//
// -----------------------------------------------------------------------------

use std::fmt::Write;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use anathema_backend::{Backend, WidgetCycle};
use anathema_default_widgets::register_default_widgets;
use anathema_state::{clear_all_changes, clear_all_subs, drain_changes, Change, Changes, States};
use anathema_store::tree::{root_node, AsNodePath, TreeView};
use anathema_templates::blueprints::Blueprint;
use anathema_templates::{Document, Globals, ToSourceKind};
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::components::deferred::DeferredComponents;
use anathema_widgets::components::events::Event;
use anathema_widgets::components::{
    AssociatedEvents, Component, ComponentId, ComponentKind, ComponentRegistry, Emitter, UntypedContext, ViewMessage,
};
use anathema_widgets::layout::{Constraints, Viewport};
use anathema_widgets::{
    eval_blueprint, update_widget, ChangeList, Components, DirtyWidgets, Factory, FloatingWidgets, GlyphMap,
    LayoutForEach, WidgetKind, WidgetTree,
};
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher};

pub use crate::builder::Builder;
pub use crate::error::{Error, Result};
pub use crate::runtime::{Frame, Runtime};

static REBUILD: AtomicBool = AtomicBool::new(false);

mod error;

pub mod builder;
mod events;
pub mod runtime;
