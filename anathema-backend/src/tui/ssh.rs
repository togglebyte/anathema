use std::io::Write;
use std::time::Duration;

use anathema_geometry::Size;
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::{GlyphMap, PaintChildren, components::events::Event};

use crate::{
    Backend,
    tui::{Screen, events::Events},
};

/// Custom SSH-based backend that writes to SSH terminal instead of stdout
pub struct SSHBackend<W: Write> {
    /// Handle to the SSH terminal for writing output
    pub output: W,
    screen: Screen,
    events: Events,
    size: Size,
}

impl<W: Write> SSHBackend<W> {
    /// Create a new SSH backend.
    pub fn new(terminal_handle: W, size: Size) -> Self {
        Self {
            terminal_handle,
            screen: Screen::new(size),
            events: Events,
            size,
        }
    }
}

impl<W: Write> Backend for SSHBackend<W> {
    fn size(&self) -> Size {
        self.size
    }

    fn next_event(&mut self, timeout: Duration) -> Option<Event> {
        self.events.poll(timeout)
    }

    fn resize(&mut self, new_size: Size, _glyph_map: &mut GlyphMap) {
        self.size = new_size;
        self.screen.resize(new_size);
    }

    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        anathema_widgets::paint::paint(&mut self.screen, glyph_map, widgets, attribute_storage);
    }

    fn render(&mut self, glyph_map: &mut GlyphMap) {
        let _ = self.screen.render(&mut self.terminal_handle, glyph_map);
    }

    fn clear(&mut self) {
        self.screen.erase();
    }

    fn finalize(&mut self) {
        // SSH connections don't need special finalization like raw mode
    }
}

/// Backend builder for a tui backend.
pub struct SSHBackendBuilder {
    output: impl std::io::Write,
    hide_cursor: bool,
    enable_raw_mode: bool,
    enable_alt_screen: bool,
    enable_mouse: bool,
}

impl TuiBackendBuilder {
    /// Enable an alternative screen.
    /// When using this with stdout it means the output will not persist
    /// once the program exits.
    pub fn enable_alt_screen(mut self) -> Self {
        self.enable_alt_screen = true;
        self
    }

    /// Enable mouse support.
    pub fn enable_mouse(mut self) -> Self {
        self.enable_mouse = true;
        self
    }

    /// When raw mode is enabled, every key press is sent to the terminal.
    /// If raw mode is not enabled, the return key has to be pressed to
    /// send characters to the terminal.
    pub fn enable_raw_mode(mut self) -> Self {
        self.enable_raw_mode = true;
        self
    }

    /// Hide the text cursor.
    pub fn hide_cursor(mut self) -> Self {
        self.hide_cursor = true;
        self
    }

    /// Clear the screen using ansi escape codes.
    pub fn clear(mut self) -> Self {
        let _ = execute!(&mut self.output, Clear(ClearType::All));
        self
    }

    /// Consume self and create the tui backend.
    pub fn finish(self) -> Result<TuiBackend, std::io::Error> {
        let size = size()?;
        let screen = Screen::new(size);

        let backend = TuiBackend {
            screen,
            output: self.output,
            events: Events,

            hide_cursor: self.hide_cursor,
            enable_raw_mode: self.enable_raw_mode,
            enable_alt_screen: self.enable_alt_screen,
            enable_mouse: self.enable_mouse,
        };

        Ok(backend)
    }
}
