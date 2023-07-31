use std::io::{stdout, Stdout};
use std::sync::Arc;
use std::time::Instant;

use anathema_generator::{Nodes, Expression};
use anathema_render::{size, Screen, Size, Attributes};
use anathema_values::Bucket;
use anathema_widget_core::contexts::PaintCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Constraints;
use anathema_widget_core::template::Template;
use anathema_widget_core::views::View;
use anathema_widget_core::{Pos, Value, WidgetContainer};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use events::Event;

use self::frame::Frame;
use self::meta::Meta;
use crate::events::{EventProvider, Events};

pub mod events;
mod frame;
mod meta;

pub struct Runtime<E, ER> {
    pub enable_meta: bool,
    pub enable_mouse: bool,
    meta: Meta,
    templates: Arc<[Template]>,
    screen: Screen,
    output: Stdout,
    constraints: Constraints,
    current_frame: Nodes<WidgetContainer>,
    ctx: Bucket<Value>,
    events: E,
    event_receiver: ER,
}

impl<E, ER> Drop for Runtime<E, ER> {
    fn drop(&mut self) {
        let _ = Screen::show_cursor(&mut self.output);
        let _ = disable_raw_mode();
    }
}

impl<E, ER> Runtime<E, ER>
where
    E: Events,
    ER: EventProvider,
{
    pub fn new(
        templates: impl Into<Arc<[Expression<Attributes>]>>,
        ctx: Bucket<Value>,
        events: E,
        event_receiver: ER,
    ) -> Result<Self> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        let size: Size = size()?.into();
        let constraints = Constraints::new(Some(size.width), Some(size.height));
        Screen::hide_cursor(&mut stdout)?;
        let screen = Screen::new(size);

        let inst = Self {
            output: stdout,
            meta: Meta::new(size),
            screen,
            constraints,
            templates: templates.into(),
            current_frame: Nodes::empty(),
            ctx,
            events,
            event_receiver,
            enable_meta: false,
            enable_mouse: false,
        };

        Ok(inst)
    }

    fn initial(&mut self) {}

    pub fn register_view(&mut self, name: impl Into<String>, view: impl View + 'static) {
        panic!()
        // self.ctx.views.register(name.into(), view);
    }

    fn layout(&mut self) -> Result<()> {
        panic!()
        // let mut values = Values::new(&self.ctx);
        // let mut widgets = Generator::new(&self.templates, &mut values);
        // let mut frame = Frame::empty();
        // while let Some(mut widget) = widgets.next(&mut values).transpose()? {
        //     widget.layout(self.constraints, &values)?;
        //     frame.push(widget);
        // }
        // self.current_frame = frame;
        // Ok(())
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
        if self.enable_mouse {
            Screen::enable_mouse(&mut self.output)?;
        }

        self.screen.clear_all(&mut self.output)?;

        'run: loop {
            while let Some(event) = self.event_receiver.next() {
                let event =
                    self.events
                        .event(event, self.ctx.write(), &mut self.current_frame.inner);
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
                self.meta.update(self.ctx.write(), &self.current_frame);
            }
        }
    }
}
