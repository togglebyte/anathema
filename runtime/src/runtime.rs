use std::time::Duration;

use templates::parse;
use templates::DataCtx;
use templates::WidgetLookup;
use widgets::WidgetContainer;

use crate::appstate::{AppState, Run, Sender, UserModel, WaitFor};
use crate::error::Result;
use crate::events::Events;
use crate::Event;
use crate::{OutputConfig, Stdout};

pub struct Runtime<T> {
    /// Configuration for the output.
    pub output_cfg: OutputConfig,
    /// Widget lookup.
    pub lookup: WidgetLookup,
    /// The amount of time between render / update calls.
    pub frame_time: Duration,
    events: Events<T>,
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
    pub fn tx(&self) -> Sender<T> {
        self.events.tx()
    }

    /// Start the runtime with a custom user model.
    pub fn with_usermodel(self, template: impl AsRef<str>, user_model: impl UserModel<Message = T>) -> Result<()> {
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

    pub fn start<F>(self, template: impl AsRef<str>, initial: DataCtx, f: F) -> Result<()>
    where
        F: FnMut(Event<T>, &mut WidgetContainer, &mut DataCtx, &mut Sender<T>),
    {
        let tx = self.tx();
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
