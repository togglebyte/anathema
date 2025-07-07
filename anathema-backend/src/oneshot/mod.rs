//! The tui module contains the essential types for drawing in the terminal.
#![deny(missing_docs)]
use std::io::{Stdout, Write};
use std::ops::Add;
use std::time::Duration;

use anathema_geometry::{LocalPos, Pos, Size};
use anathema_value_resolver::AttributeStorage;
use anathema_widgets::components::events::Event;
pub use anathema_widgets::{Attributes, Style};
use anathema_widgets::{GlyphMap, PaintChildren, WidgetRenderer};
use crossterm::cursor::{RestorePosition, SavePosition};
use crossterm::execute;
use crossterm::terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate, size};
pub use screen::Screen;

pub use self::buffer::Buffer;
use self::events::Events;
use crate::Backend;

mod buffer;
/// Events
pub mod events;
mod screen;
mod style;

/// Backend builder for a tui backend.
pub struct OneshotBackendBuilder {
    output: Stdout,
}

impl OneshotBackendBuilder {
    /// Consume self and create the tui backend.
    pub fn finish(self) -> Result<OneshotBackend, std::io::Error> {
        let size = size()?;
        let screen = Screen::new(size);

        let cursor_pos = crossterm::cursor::position()?.into();
        let backend = OneshotBackend {
            cursor_pos,
            screen,
            output: self.output,
            events: Events,
        };

        Ok(backend)
    }
}

/// Terminal backend
pub struct OneshotBackend {
    screen: Screen,
    output: Stdout,
    events: Events,
    cursor_pos: LocalPos,
}

impl OneshotBackend {
    /// Create a new instance of the tui backend.
    pub fn builder() -> OneshotBackendBuilder {
        OneshotBackendBuilder {
            output: std::io::stdout(),
        }
    }
}

impl Backend for OneshotBackend {
    fn size(&self) -> Size {
        self.screen.size()
    }

    fn next_event(&mut self, timeout: Duration) -> Option<Event> {
        None
    }

    fn resize(&mut self, _: Size, _: &mut GlyphMap) { }

    fn paint<'bp>(
        &mut self,
        glyph_map: &mut GlyphMap,
        widgets: PaintChildren<'_, 'bp>,
        attribute_storage: &AttributeStorage<'bp>,
    ) {
        anathema_widgets::paint::paint(&mut self.screen, glyph_map, widgets, attribute_storage);
    }

    fn render(&mut self, glyph_map: &mut GlyphMap) {
        let _ = self.screen.render(&mut self.output, self.cursor_pos, glyph_map);
    }

    fn clear(&mut self) {}

    fn finalize(&mut self) {}
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
