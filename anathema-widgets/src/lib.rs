pub mod layout;
// #[cfg(feature = "testing")]
// pub mod testing;

mod alignment;
mod border;
mod expand;
mod hstack;
mod position;
mod spacer;
mod text;
// // // mod viewport;
mod vstack;
mod zstack;

use anathema_widget_core::error::Result;
use anathema_widget_core::Factory;

// -----------------------------------------------------------------------------
//   - Export widgets -
// -----------------------------------------------------------------------------
pub use crate::alignment::Alignment;
pub use crate::border::{Border, BorderStyle, Sides};
pub use crate::expand::Expand;
pub use crate::hstack::HStack;
pub use crate::position::Position;
pub use crate::spacer::Spacer;
pub use crate::text::{Text, TextSpan};
// // pub use crate::viewport::Viewport;
pub use crate::vstack::VStack;
pub use crate::zstack::ZStack;

// -----------------------------------------------------------------------------
//   - Widget factories -
// -----------------------------------------------------------------------------
mod factories {
    pub(super) use crate::alignment::AlignmentFactory;
    pub(super) use crate::border::BorderFactory;
    pub(super) use crate::expand::ExpandFactory;
    pub(super) use crate::hstack::HStackFactory;
    pub(super) use crate::position::PositionFactory;
    pub(super) use crate::spacer::SpacerFactory;
    pub(super) use crate::text::{SpanFactory, TextFactory};
    // // pub(super) use crate::viewport::ViewportFactory;
    pub(super) use crate::vstack::VStackFactory;
    pub(super) use crate::zstack::ZStackFactory;
}

/// Register the default widgets.
pub fn register_default_widgets() -> Result<()> {
    let results = [
        Factory::register("alignment".to_string(), factories::AlignmentFactory),
        Factory::register("border".to_string(), factories::BorderFactory),
        Factory::register("expand".to_string(), factories::ExpandFactory),
        Factory::register("hstack".to_string(), factories::HStackFactory),
        Factory::register("position".to_string(), factories::PositionFactory),
        Factory::register("spacer".to_string(), factories::SpacerFactory),
        Factory::register("span".to_string(), factories::SpanFactory),
        Factory::register("text".to_string(), factories::TextFactory),
        Factory::register("vstack".to_string(), factories::VStackFactory),
        Factory::register("zstack".to_string(), factories::ZStackFactory),
        // // Factory::register("viewport".to_string(), factories::ViewportFactory),
    ];

    for result in results {
        result?;
    }

    Ok(())
}
