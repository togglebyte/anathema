use anathema_frontend::Frontend;
use anathema_geometry::{Pos, Region, Size};
use anathema_store::secondary_map::{GenerationalStorage, SecondaryMap};
use anathema_store::slab::{Generational, Key};

pub use self::iter::Children;
pub use self::layout::{LayoutSize, Layouts};
pub use self::registry::RegisteredWidgets;
pub(crate) use self::root::Root;
pub use self::view::{View, build_view_tree};
use crate::attributes::{Attributes, WidgetAttributes};
use crate::constraints::Constraints;
use crate::elements::ElementId;

pub mod iter;
mod layout;
mod registry;
mod root;
mod view;

/// A widget.
///
/// Attributes should not be preserved on the widgets themselves
/// except to act as a cache between layout, position and paint.
/// Since an attribute can change as a result of a component event,
/// and this will not update any cached values.
pub trait Widget: std::any::Any {
    /// Layout the widget
    fn layout<'bp>(
        &mut self,
        id: ElementId,
        children: &Children<'_, 'bp>,
        attributes: &WidgetAttributes<'_, 'bp>,
        layouts: &mut Layouts,
        constraints: Constraints,
    ) -> LayoutSize;

    /// Position the widget
    fn position<'bp>(
        &mut self,
        id: ElementId,
        children: &Children<'_, 'bp>,
        attributes: &WidgetAttributes<'_, 'bp>,
        layouts: &mut Layouts,
        pos: Pos,
    );

    /// Paint the widget
    fn paint<'bp>(
        &mut self,
        id: ElementId,
        region: Region,
        children: Children<'_, 'bp>,
        attributes: WidgetAttributes<'_, 'bp>,
        frontend: &mut dyn Frontend,
        layouts: &Layouts,
    );

    /// A function that described a widget in a debug context.
    fn describe(&self) -> &str {
        "<dyn Element>"
    }
}

impl std::fmt::Debug for dyn Widget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.describe())
    }
}
