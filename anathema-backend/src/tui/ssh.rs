use crossterm::execute;

/// Backend builder for a tui backend.
pub struct SSHBackendBuilder {
    output: Stdout,
    hide_cursor: bool,
    enable_raw_mode: bool,
    enable_alt_screen: bool,
    enable_mouse: bool,
}

impl SSHBackendBuilder {
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

/// Terminal backend
pub struct SSHBackend {
    screen: Screen,
    output: Stdout,
    events: Events,

    // Settings
    hide_cursor: bool,
    enable_raw_mode: bool,
    enable_alt_screen: bool,
    enable_mouse: bool,
}

impl SSHBackend {
    /// Create a new instance of the tui backend.
    pub fn builder() -> SSHBackendBuilder {
        SSHBackendBuilder {
            output: std::io::stdout(),
            hide_cursor: false,
            enable_raw_mode: false,
            enable_alt_screen: false,
            enable_mouse: false,
        }
    }

    /// Convenience function this is the same as calling
    /// ```no_run
    /// # use anathema_backend::tui::TuiBackend;
    /// # use anathema_backend::Backend;
    /// let mut backend = TuiBackend::builder()
    ///     .enable_alt_screen()
    ///     .enable_raw_mode()
    ///     .hide_cursor()
    ///     .finish()
    ///     .unwrap();
    /// backend.finalize();
    /// ```
    pub fn full_screen() -> Self {
        let mut inst = Self::builder()
            .enable_alt_screen()
            .enable_raw_mode()
            .hide_cursor()
            .finish()
            .unwrap();
        inst.finalize();
        inst
    }

    /// Disable raw mode.
    pub fn disable_raw_mode(self) -> Self {
        let _ = Screen::disable_raw_mode();
        self
    }
}

impl Backend for TuiBackend {
    fn size(&self) -> Size {
        self.screen.size()
    }

    fn next_event(&mut self, timeout: Duration) -> Option<Event> {
        self.events.poll(timeout)
    }

    fn resize(&mut self, new_size: Size, _: &mut GlyphMap) {
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
        let _ = execute!(&mut self.output, BeginSynchronizedUpdate);
        let _ = self.screen.render(&mut self.output, glyph_map);
        let _ = execute!(&mut self.output, EndSynchronizedUpdate);
    }

    fn clear(&mut self) {
        self.screen.erase();
    }

    fn finalize(&mut self) {
        if self.enable_alt_screen {
            let _ = execute!(&mut self.output, SavePosition);
            let _ = Screen::enter_alt_screen(&mut self.output);
        }

        if self.hide_cursor {
            // This is to fix an issue with Windows cmd.exe
            let _ = Screen::show_cursor(&mut self.output);
            let _ = Screen::hide_cursor(&mut self.output);
        }

        if self.enable_raw_mode {
            let _ = Screen::enable_raw_mode();
        }

        if self.enable_mouse {
            let _ = Screen::enable_mouse(&mut self.output);
        }

        let _ = self.output.flush();
    }
}
