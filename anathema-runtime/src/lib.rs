use std::io::{stdout, Stdout};
use std::time::Instant;

use anathema_render::{size, Screen, Size};
use anathema_widgets::error::Result;
use anathema_widgets::template::Template;
use anathema_widgets::{
    Constraints, DataCtx, Generator, Lookup, PaintCtx, Pos, Store, WidgetContainer,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use events::Event;

use self::meta::Meta;
use crate::events::{EventProvider, Events};

pub mod events;
mod meta;

pub struct Runtime<'tpl, E, ER> {
    pub enable_meta: bool,
    meta: Meta,
    templates: &'tpl [Template],
    screen: Screen,
    output: Stdout,
    lookup: Lookup,
    constraints: Constraints,
    current_frame: Vec<WidgetContainer<'tpl>>,
    ctx: DataCtx,
    events: E,
    event_receiver: ER,
}

impl<E, ER> Drop for Runtime<'_, E, ER> {
    fn drop(&mut self) {
        let _ = Screen::show_cursor(&mut self.output);
        let _ = disable_raw_mode();
    }
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
        enable_raw_mode()?;

        let mut stdout = stdout();
        let size: Size = size()?.into();
        let constraints = Constraints::new(Some(size.width), Some(size.height));
        Screen::hide_cursor(&mut stdout)?;
        let screen = Screen::new(size);
        let lookup = Lookup::default();

        let inst = Self {
            output: stdout,
            meta: Meta::new(size),
            screen,
            lookup,
            constraints,
            templates,
            current_frame: vec![],
            ctx,
            events,
            event_receiver,
            enable_meta: false,
        };

        Ok(inst)
    }

    fn layout(&mut self) -> Result<()> {
        // TODO: diffing!
        self.current_frame.clear();
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

        'run: loop {
            while let Some(event) = self.event_receiver.next() {
                let event = self
                    .events
                    .event(event, &mut self.ctx, &mut self.current_frame);
                match event {
                    Event::Resize(width, height) => {
                        let size = Size::from((width, height));
                        self.screen.erase();
                        self.screen.render(&mut self.output)?;
                        self.screen.resize(size);

                        self.constraints.max_width = size.width;
                        self.constraints.max_height = size.height;

                        self.meta.size = size;
                    }
                    Event::Blur => self.meta.focus = false,
                    Event::Focus => self.meta.focus = true,
                    Event::Quit => break 'run Ok(()),
                    _ => {}
                }
            }

            let total = Instant::now();
            self.layout()?;
            self.meta.timings.layout = total.elapsed();

            let now = Instant::now();
            self.position();
            self.meta.timings.position = now.elapsed();

            let now = Instant::now();
            self.paint();
            self.meta.timings.paint = now.elapsed();

            let now = Instant::now();
            self.screen.render(&mut self.output)?;
            self.meta.timings.render = now.elapsed();
            self.meta.timings.total = total.elapsed();
            self.screen.erase();

            if self.enable_meta {
                self.meta.update(&mut self.ctx, &self.current_frame);
            }
        }
    }
}
