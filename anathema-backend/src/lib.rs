use std::time::Duration;

use anathema_geometry::Size;
use anathema_store::tree::{Node, TreeValues};
use anathema_widgets::components::events::Event;
use anathema_widgets::layout::text::StringSession;
use anathema_widgets::{AttributeStorage, Element, WidgetKind};

pub mod test;
pub mod tui;

pub trait Backend {
    fn size(&self) -> Size;

    /// When [Backend::next_event] returns [Event::Stop], this function will be called to make sure the Backend wants anathema exit.
    fn quit_test(&self, event: Event) -> bool;

    fn next_event(&mut self, timeout: Duration) -> Option<Event>;

    fn resize(&mut self, new_size: Size);

    /// Paint the widgets
    fn paint<'bp>(
        &mut self,
        element: &mut Element<'bp>,
        children: &[Node],
        values: &mut TreeValues<WidgetKind<'bp>>,
        text: &mut StringSession<'_>,
        attribute_storage: &AttributeStorage<'bp>,
        ignore_floats: bool,
    );

    /// Publish the changes to the Buffer to the Screen.
    fn render(&mut self);

    /// Clear the internal buffer entirely. This should not change the screen.
    fn clear(&mut self);

    /// Finalizes the backend. This is called when the runtime starts.
    fn finalize(&mut self) {}
}
