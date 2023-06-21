use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use anathema_render::{size, Screen};
use anathema_widgets::error::Result;
use anathema_widgets::template::Template;
use anathema_widgets::{
    Constraints, DataCtx, Generator, Lookup, PaintCtx, Pos, Store, WidgetContainer,
};

#[derive(Debug, Default)]
struct Timings {
    layout: Duration,
    position: Duration,
    paint: Duration,
    render: Duration,
}

pub struct Runtime<'tpl> {
    templates: &'tpl [Template],
    screen: Screen,
    output: Stdout,
    lookup: Lookup,
    constraints: Constraints,
    last_frame: Vec<WidgetContainer<'tpl>>,
    current_frame: Vec<WidgetContainer<'tpl>>,
    ctx: DataCtx,
    timings: Timings,
}

impl<'tpl> Runtime<'tpl> {
    pub fn new(templates: &'tpl [Template], ctx: DataCtx) -> Result<Self> {
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
            last_frame: vec![],
            current_frame: vec![],
            ctx,
            timings: Default::default(),
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
        let mut counter = 0;

        loop {
            // self.update();
            let now = Instant::now();
            self.layout();
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
            // thread::sleep(Duration::from_millis(500));
            self.screen.erase();

            counter += 1;
            if counter > 1 {
                break;
            }
        }

        eprintln!("{:#?}", self.timings);
        eprintln!("count: {}", count(&self.current_frame));
        Ok(())
    }
}

fn count(w: &[WidgetContainer<'_>]) -> usize {
    let mut c = w.len();
    for wc in w {
        c += count(&wc.children);
    }

    c
}
