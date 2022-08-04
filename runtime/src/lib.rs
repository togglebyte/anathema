use display::Size;
use display::{
    cursor, disable_raw_mode, enable_raw_mode, size, DisableMouseCapture, EnableMouseCapture, EnterAlternateScreen,
    ExecutableCommand, LeaveAlternateScreen, QueueableCommand,
};
use std::io::{self, Write};

pub mod error;

mod appstate;
mod events;
mod runtime;

pub use crate::runtime::Runtime;
pub use crate::appstate::{AppState, UserModel, Sender};
pub use crate::events::{Event, Events, CrossEvent, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

pub trait Output: Write {
    /// The size of the output.
    fn size(&self) -> Size;
}

/// Configure the output.
pub struct OutputConfig {
    /// Raw mode (as opposed to cooked mode)
    pub raw_mode: bool,
    /// Enable mouse support
    pub enable_mouse: bool,
    /// Render to an alternate screen.
    /// Once the config is dropped it will restore the main screen.
    pub alt_screen: bool,
}

/// Stdout as [`Output`]
pub struct Stdout(io::Stdout);

impl Stdout {
    /// Create a new instance of [`self::Stdout`]
    pub fn new(config: OutputConfig) -> io::Result<Self> {
        let mut stdout = io::stdout();
        stdout.queue(cursor::Hide)?;

        if config.raw_mode {
            enable_raw_mode()?;
            if config.alt_screen {
                stdout.execute(EnterAlternateScreen)?;
            }
        }

        if config.enable_mouse {
            stdout.queue(EnableMouseCapture)?;
        }

        stdout.flush()?;
        Ok(Self(stdout))
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl Output for Stdout {
    fn size(&self) -> Size {
        size().expect("failed to get terminal size").into()
    }
}

impl Drop for Stdout {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = self.execute(LeaveAlternateScreen);
        let _ = self.execute(DisableMouseCapture);
        let _ = self.execute(cursor::Show);
    }
}
