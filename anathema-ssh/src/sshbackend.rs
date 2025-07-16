use std::time::Duration;

use anathema_backend::{Backend, tui::TuiBackend};
use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::{GlyphMap, PaintChildren, components::events::Event};

use crate::sshserver::TerminalHandle;

/// SSH Backend that wraps TuiBackend and handles SSH-specific input/output
pub struct SSHBackend {
    tui_backend: TuiBackend<TerminalHandle>,
}

impl SSHBackend {
    pub fn new(terminal_handle: TerminalHandle) -> anyhow::Result<Self> {
        let tui_backend = TuiBackend::builder_with_output(terminal_handle)
            .enable_alt_screen()
            .enable_raw_mode()
            .enable_mouse()
            .hide_cursor()
            .finish()?;

        Ok(Self { tui_backend })
    }

    pub fn output_mut(&mut self) -> &mut TerminalHandle {
        self.tui_backend.output()
    }
}

impl Backend for SSHBackend {
    fn size(&self) -> Size {
        self.tui_backend.size()
    }

    fn next_event(&mut self, _timeout: Duration) -> Option<Event> {
        // First check if there are any events in the terminal handle
        self.tui_backend.output().pop_event()
    }

    fn resize(&mut self, new_size: Size, glyph_map: &mut GlyphMap) {
        self.tui_backend.resize(new_size, glyph_map)
    }

    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        self.tui_backend.paint(glyph_map, widgets, attribute_storage)
    }

    fn render(&mut self, glyph_map: &mut GlyphMap) {
        self.tui_backend.render(glyph_map);
    }

    fn clear(&mut self) {
        self.tui_backend.clear()
    }

    fn finalize(&mut self) {
        self.tui_backend.finalize()
    }
}
