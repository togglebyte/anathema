//! The tui module contains the essential types for drawing in the terminal.
//!
//! It uses two buffers and only draws the diffs from top left to bottom right, making it less
//! likely to flicker when moving the cursor etc.
#![deny(missing_docs)]
use std::io::{Stdout, Write};
use std::ops::Add;
use std::time::Duration;

use anathema_geometry::{LocalPos, Pos, Size};
use anathema_store::tree::{Node, TreeValues};
use anathema_widgets::components::events::Event;
use anathema_widgets::{AttributeStorage, Element, WidgetKind, WidgetRenderer};
use crossterm::terminal::size;
pub use screen::Screen;

pub use self::buffer::Buffer;
use self::events::Events;
pub use self::style::{Attributes, Style};
use crate::Backend;

mod buffer;
/// Events
pub mod events;
mod screen;
mod style;

/// Backend builder for a tui backend.
pub struct TuiBackendBuilder {
    output: Stdout,
    quit_on_ctrl_c: bool,

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

    /// Consume self and create the tui backend.
    pub fn finish(self) -> Result<TuiBackend, std::io::Error> {
        let size = size()?;
        let screen = Screen::new(size);

        let backend = TuiBackend {
            quit_on_ctrl_c: self.quit_on_ctrl_c,
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
pub struct TuiBackend {
    /// Stop the runtime if Ctrl+c was pressed.
    pub quit_on_ctrl_c: bool,
    screen: Screen,
    output: Stdout,
    events: Events,

    // Settings
    hide_cursor: bool,
    enable_raw_mode: bool,
    enable_alt_screen: bool,
    enable_mouse: bool,
}

impl TuiBackend {
    /// Create a new instance of the tui backend.
    pub fn builder() -> TuiBackendBuilder {
        let output = std::io::stdout();

        TuiBackendBuilder {
            output,
            quit_on_ctrl_c: true,

            hide_cursor: false,
            enable_raw_mode: false,
            enable_alt_screen: false,
            enable_mouse: false,
        }
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

    fn resize(&mut self, new_size: Size) {
        self.screen.resize(new_size);
    }

    fn paint<'bp>(
        &mut self,
        element: &mut Element<'bp>,
        children: &[Node],
        values: &mut TreeValues<WidgetKind<'bp>>,
        attribute_storage: &AttributeStorage<'bp>,
        ignore_floats: bool,
    ) {
        anathema_widgets::paint::paint(
            &mut self.screen,
            element,
            children,
            values,
            attribute_storage,
            ignore_floats,
        );
        // TODO: decide if we need `paint` to return a Result or not
    }

    fn render(&mut self) {
        let _ = self.screen.render(&mut self.output);
    }

    fn clear(&mut self) {
        self.screen.erase();
    }

    fn finalize(&mut self) {
        if self.hide_cursor {
            // This is to fix an issue with Windows cmd.exe
            let _ = Screen::show_cursor(&mut self.output);
            let _ = Screen::hide_cursor(&mut self.output);
        }

        if self.enable_raw_mode {
            let _ = Screen::enable_raw_mode();
        }

        if self.enable_alt_screen {
            let _ = Screen::enter_alt_screen(&mut self.output);
        }

        if self.enable_mouse {
            let _ = Screen::enable_mouse(&mut self.output);
        }

        let _ = self.output.flush();
    }
}

impl Drop for TuiBackend {
    fn drop(&mut self) {
        let _ = self.screen.restore(&mut self.output);
    }
}

/// Represents a position on the screen, meaning this should never
/// be a value outside of the screen size.
///
/// It will be ignored if the value is used in a drawing operation and it's outside the current
/// screen size.
///
/// `Screen::ZERO` is the top left of a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenPos {
    /// The x coordinate on screen
    pub x: u16,
    /// The y coordinate on screen
    pub y: u16,
}

impl ScreenPos {
    /// A zero screen size
    pub const ZERO: Self = Self::new(0, 0);

    /// Create a new instance of a `ScreenPos`
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

impl Add<ScreenPos> for LocalPos {
    type Output = Self;

    fn add(self, rhs: ScreenPos) -> Self::Output {
        LocalPos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl TryFrom<LocalPos> for ScreenPos {
    type Error = <u16 as TryFrom<usize>>::Error;

    fn try_from(value: LocalPos) -> std::result::Result<ScreenPos, Self::Error> {
        let x = value.x;
        let y = value.y;
        Ok(ScreenPos::new(x, y))
    }
}

impl TryFrom<Pos> for ScreenPos {
    type Error = <i32 as TryFrom<usize>>::Error;

    fn try_from(value: Pos) -> std::result::Result<ScreenPos, Self::Error> {
        let x: u16 = value.x.try_into()?;
        let y: u16 = value.y.try_into()?;
        Ok(ScreenPos::new(x, y))
    }
}
