pub mod layout;
pub mod testing;

mod alignment;
mod border;
mod expand;
mod hstack;
mod position;
mod spacer;
mod text;
mod viewport;
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
pub use crate::viewport::Viewport;
pub use crate::vstack::VStack;
pub use crate::zstack::ZStack;

// -----------------------------------------------------------------------------
//   - Widget factories -
// -----------------------------------------------------------------------------
use crate::alignment::AlignmentFactory;
use crate::border::BorderFactory;
use crate::expand::ExpandFactory;
use crate::hstack::HStackFactory;
use crate::position::PositionFactory;
use crate::spacer::SpacerFactory;
use crate::text::{TextFactory, SpanFactory};
use crate::viewport::ViewportFactory;
use crate::vstack::VStackFactory;
use crate::zstack::ZStackFactory;

/// Register the default widgets.
pub fn register_default_widgets() -> Result<()> {
    let result1 = Factory::register("alignment".to_string(), AlignmentFactory);
    let result2 = Factory::register("border".to_string(), BorderFactory);
    let result3 = Factory::register("expand".to_string(), ExpandFactory);
    let result4 = Factory::register("hstack".to_string(), HStackFactory);
    let result5 = Factory::register("position".to_string(), PositionFactory);
    let result6 = Factory::register("spacer".to_string(), SpacerFactory);
    let result7 = Factory::register("span".to_string(), SpanFactory);
    let result8 = Factory::register("text".to_string(), TextFactory);
    let result9 = Factory::register("vstack".to_string(), VStackFactory);
    let result10 = Factory::register("zstack".to_string(), ZStackFactory);
    let result11 = Factory::register("viewport".to_string(), ViewportFactory);

    result1?;
    result2?;
    result3?;
    result4?;
    result5?;
    result6?;
    result7?;
    result8?;
    result9?;
    result10?;
    result11?;

    Ok(())
}
