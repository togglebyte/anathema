use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use anathema_render::{size, Screen};
use anathema_widgets::error::Result;
use anathema_widgets::template::Template;
use anathema_widgets::{
    Constraints, DataCtx, Generator, Lookup, PaintCtx, Pos, Store, WidgetContainer,
};

use crate::events::{EventProvider, Events};

pub mod events;

#[derive(Debug, Default)]
struct Timings {
    layout: Duration,
    position: Duration,
    paint: Duration,
    render: Duration,
}

pub struct Runtime<'tpl, E, ER> {
    templates: &'tpl [Template],
    screen: Screen,
    output: Stdout,
    lookup: Lookup,
    constraints: Constraints,
    current_frame: Vec<WidgetContainer<'tpl>>,
    ctx: DataCtx,
    timings: Timings,
    events: E,
    event_receiver: ER,
}

impl<'tpl, E, ER> Runtime<'tpl, E, ER>
where
    E: Events,
    ER: EventProvider,
{
    pub fn new(
        templates: &'tpl [Template],
        ctx: DataCtx,
        events: E,
        event_receiver: ER,
    ) -> Result<Self> {
        let mut stdout = stdout();
        let (width, height) = size()?;
        let constraints = Constraints::new(Some(width as usize), Some(height as usize));
        Screen::hide_cursor(&mut stdout)?;
        let screen = Screen::new((width, height));
        let lookup = Lookup::default();

        let inst = Self {
            output: stdout,
            screen,
            lookup,
            constraints,
            templates,
            current_frame: vec![],
            ctx,
            timings: Default::default(),
            events,
            event_receiver,
        };

        Ok(inst)
    }

    fn layout(&mut self) -> Result<()> {
        let mut values = Store::new(&self.ctx);
        let mut widgets = Generator::new(&self.templates, &self.lookup, &mut values);
        while let Some(mut widget) = widgets.next(&mut values).transpose()? {
            widget.layout(self.constraints, &values, &self.lookup)?;
            self.current_frame.push(widget);
        }
        Ok(())
    }

    fn position(&mut self) {
        for widget in &mut self.current_frame {
            widget.position(Pos::ZERO);
        }
    }

    fn paint(&mut self) {
        for widget in &mut self.current_frame {
            widget.paint(PaintCtx::new(&mut self.screen, None));
        }
    }

    pub fn run(mut self) -> Result<()> {
        self.screen.clear_all(&mut self.output)?;

        loop {
            while let Some(event) = self.event_receiver.next() {
                self.events.event(event, &mut self.current_frame);
            }

            let now = Instant::now();
            self.layout()?;
            self.timings.layout = now.elapsed();

            let now = Instant::now();
            self.position();
            self.timings.position = now.elapsed();

            let now = Instant::now();
            self.paint();
            self.timings.paint = now.elapsed();

            let now = Instant::now();
            self.screen.render(&mut self.output)?;
            self.timings.render = now.elapsed();
            self.screen.erase();
        }
    }
}
