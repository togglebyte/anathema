use std::io::{self, Write};
use std::time::Duration;

use crate::display::Size;
use crate::display::{
    cursor, disable_raw_mode, enable_raw_mode, size, DisableMouseCapture, EnableMouseCapture, EnterAlternateScreen,
    ExecutableCommand, LeaveAlternateScreen, QueueableCommand,
};
use crate::templates::parse;
use crate::templates::DataCtx;
use crate::templates::WidgetLookup;
use crate::widgets::WidgetContainer;

pub mod error;

mod appstate;
mod events;

pub use appstate::{AppState, Run, Sender, UserModel, WaitFor};
pub use events::{CrossEvent, Event, Events, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

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

pub struct Runtime<T> {
    /// Configuration for the output.
    pub output_cfg: OutputConfig,
    /// Widget lookup.
    pub lookup: WidgetLookup,
    /// The amount of time between render / update calls.
    pub frame_time: Duration,
    events: Events<T>,
}

impl<T: Send + Sync + 'static> Default for Runtime<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + Sync + 'static> Runtime<T> {
    /// Create a new instance of the basic runtime.
    pub fn new() -> Self {
        let events = Events::unbounded();
        let output_cfg = OutputConfig { alt_screen: true, raw_mode: true, enable_mouse: false };
        Self { lookup: WidgetLookup::default(), events, output_cfg, frame_time: Duration::from_millis(20) }
    }

    /// Get an instance of the `Sender<T>`.
    /// This is used to pass events to the runtime.
    pub fn sender(&self) -> Sender<T> {
        self.events.sender()
    }

    /// Start the runtime with a custom user model.
    pub fn with_usermodel(
        self,
        template: impl AsRef<str>,
        user_model: impl UserModel<Message = T>,
    ) -> error::Result<()> {
        // -----------------------------------------------------------------------------
        //     - Output -
        // -----------------------------------------------------------------------------
        let output = Stdout::new(self.output_cfg)?;

        // -----------------------------------------------------------------------------
        //     - Nodes -
        // -----------------------------------------------------------------------------
        let nodes = parse(template.as_ref())?;

        // -----------------------------------------------------------------------------
        //     - App state -
        // -----------------------------------------------------------------------------
        let mut app =
            AppState::new(user_model, self.events, nodes, self.lookup, output, WaitFor::Timeout(self.frame_time))?;

        while let Ok(Run::Continue) = app.wait_for() {}

        Ok(())
    }

    pub fn start<F>(self, template: impl AsRef<str>, initial: DataCtx, f: F) -> error::Result<()>
    where
        F: FnMut(Event<T>, &mut WidgetContainer, &mut DataCtx, &mut Sender<T>),
    {
        let tx = self.sender();
        let state = DefaultState { tx, data: initial, f };
        self.with_usermodel(template, state)?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
//     - Dummy user model -
// -----------------------------------------------------------------------------
struct DefaultState<T, F>
where
    T: Send + Sync + 'static,
    F: FnMut(Event<T>, &mut WidgetContainer, &mut DataCtx, &mut Sender<T>),
{
    tx: Sender<T>,
    data: DataCtx,
    f: F,
}

impl<T, F> UserModel for DefaultState<T, F>
where
    T: Send + Sync + 'static,
    F: FnMut(Event<T>, &mut WidgetContainer, &mut DataCtx, &mut Sender<T>),
{
    type Message = T;

    fn event(&mut self, event: Event<Self::Message>, root: &mut WidgetContainer) {
        (self.f)(event, root, &mut self.data, &mut self.tx);
    }

    fn data(&mut self) -> &mut DataCtx {
        &mut self.data
    }
}
