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

    fn quit_test(&self, event: Event) -> bool;

    fn next_event(&mut self, timeout: Duration) -> Option<Event>;

    fn resize(&mut self, new_size: Size);

    fn paint<'bp>(
        &mut self,
        element: &mut Element<'bp>,
        children: &[Node],
        values: &mut TreeValues<WidgetKind<'bp>>,
        text: &mut StringSession<'_>,
        attribute_storage: &AttributeStorage<'bp>,
        ignore_floats: bool,
    );

    fn render(&mut self);

    fn clear(&mut self);
}
