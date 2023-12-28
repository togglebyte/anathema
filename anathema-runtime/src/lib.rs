use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use anathema_render::{size, Screen, Size};
use anathema_values::{drain_dirty_nodes, Context, Scope};
use anathema_widget_core::contexts::PaintCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::expressions::Expression;
use anathema_widget_core::layout::Constraints;
use anathema_widget_core::nodes::{make_it_so, Nodes};
use anathema_widget_core::views::Views;
use anathema_widget_core::{Event, Events, KeyCode, LayoutNodes, Padding, Pos};
use anathema_widgets::register_default_widgets;
use crossterm::terminal::enable_raw_mode;
pub use meta::Meta;
use tabindex::Direction;

use crate::tabindex::TabIndexing;

mod meta;
mod tabindex;

pub struct Runtime<'e> {
    pub enable_meta: bool,
    pub enable_mouse: bool,
    pub enable_ctrlc: bool,
    pub enable_tabindex: bool,
    pub enable_alt_screen: bool,
    pub fps: u8,
    screen: Screen,
    output: Stdout,
    constraints: Constraints,
    nodes: Nodes<'e>,
    events: Events,
    needs_layout: bool,
    meta: meta::Meta,
    tabindex: TabIndexing,
}

impl<'e> Drop for Runtime<'e> {
    fn drop(&mut self) {
        let _ = self.screen.restore(&mut self.output);
    }
}

impl<'e> Runtime<'e> {
    pub fn new(expressions: &'e [Expression]) -> Result<Self> {
        register_default_widgets()?;

        let nodes = make_it_so(expressions);

        let size: Size = size()?.into();
        let constraints = Constraints::new(Some(size.width), Some(size.height));
        let screen = Screen::new(size);

        let inst = Self {
            output: stdout(),
            screen,
            constraints,
            nodes,
            enable_meta: false,
            enable_mouse: false,
            enable_alt_screen: true,
            events: Events,
            fps: 30,
            needs_layout: true,
            meta: meta::Meta::new(size),
            tabindex: TabIndexing::new(),
            enable_ctrlc: true,
            enable_tabindex: true,
        };

        Ok(inst)
    }

    fn layout(&mut self) -> Result<()> {
        self.nodes.reset_cache();
        let scope = Scope::new();
        let context = Context::root(&(), &scope);
        let mut nodes =
            LayoutNodes::new(&mut self.nodes, self.constraints, Padding::ZERO, &context);

        nodes.for_each(|mut node| {
            node.layout(self.constraints)?;
            Ok(())
        })?;

        Ok(())
    }

    fn position(&mut self) {
        for (widget, children) in self.nodes.iter_mut() {
            widget.position(children, Pos::ZERO);
        }
    }

    fn paint(&mut self) {
        for (widget, children) in self.nodes.iter_mut() {
            widget.paint(children, PaintCtx::new(&mut self.screen, None));
        }
    }

    fn changes(&mut self) {
        let dirty_nodes = drain_dirty_nodes();
        if dirty_nodes.is_empty() {
            return;
        }

        self.needs_layout = true;
        let scope = Scope::new();
        let context = Context::root(&(), &scope);

        for (node_id, change) in dirty_nodes {
            self.nodes.update(node_id.as_slice(), &change, &context);
        }
    }

    fn tick_views(&mut self) {
        Views::for_each(|node_id, _| {
            self.nodes.with_view(node_id, |view| view.tick());
        });
    }

    fn global_event(&mut self, event: Event) -> Event {
        // -----------------------------------------------------------------------------
        //   - Ctrl-c to quite -
        //   This should be on by default.
        //   Give it a good name
        // -----------------------------------------------------------------------------
        if self.enable_ctrlc {
            if let Event::CtrlC = event {
                return Event::Quit;
            }
        }

        // -----------------------------------------------------------------------------
        //   - Handle tabbing between widgets -
        // -----------------------------------------------------------------------------
        if self.enable_tabindex {
            if let Event::KeyPress(code @ (KeyCode::Tab | KeyCode::BackTab), ..) = event {
                let dir = match code {
                    KeyCode::Tab => Direction::Forwards,
                    KeyCode::BackTab => Direction::Backwards,
                    _ => unreachable!(),
                };

                if let Some(old) = self.tabindex.next(dir) {
                    self.nodes.with_view(&old, |view| view.blur());
                }

                if let Some(next) = self.tabindex.current_node() {
                    self.nodes.with_view(next, |view| view.focus());
                }
            }
        }

        event
    }

    pub fn run(mut self) -> Result<()> {
        if self.enable_alt_screen {
            self.screen.enter_alt_screen(&mut self.output)?;
        }

        enable_raw_mode()?;
        Screen::hide_cursor(&mut self.output)?;

        self.layout()?;

        if self.enable_mouse {
            Screen::enable_mouse(&mut self.output)?;
        }

        if self.enable_tabindex {
            self.tabindex.next(Direction::Forwards);
        }

        self.screen.clear_all(&mut self.output)?;

        let mut fps_now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;

        'run: loop {
            while let Some(event) = self.events.poll(Duration::from_millis(1)) {
                let event = self.global_event(event);

                // Make sure event handling isn't holding up the rest of the event loop.
                if fps_now.elapsed().as_micros() > sleep_micros {
                    break;
                }

                match event {
                    Event::Resize(width, height) => {
                        let size = Size::from((width, height));
                        self.screen.erase();
                        self.screen.render(&mut self.output)?;
                        self.screen.resize(size);
                        self.screen.clear_all(&mut self.output)?;

                        self.constraints.max_width = size.width;
                        self.constraints.max_height = size.height;

                        self.meta.size = size;
                    }
                    Event::Blur => (),  //self.meta.focus = false,
                    Event::Focus => (), //self.meta.focus = true,
                    Event::Quit => break 'run Ok(()),
                    _ => {}
                }

                if self.enable_tabindex {
                    if let Some(view_id) = self.tabindex.current_node() {
                        self.nodes.with_view(view_id, |view| view.on_event(event));
                    }
                } else {
                    // TODO: this is a bit sketchy
                    let root = 0.into();
                    self.nodes.with_view(&root, |view| view.on_event(event));
                }
            }

            self.changes();

            self.meta.count = self.nodes.count();

            if self.needs_layout {
                let meta_total = Instant::now();

                self.layout()?;
                self.meta.timings.layout = meta_total.elapsed();

                let now = Instant::now();
                self.position();
                self.meta.timings.position = now.elapsed();

                let now = Instant::now();
                self.paint();
                self.meta.timings.paint = now.elapsed();

                let now = Instant::now();
                self.screen.render(&mut self.output)?;
                self.meta.timings.render = now.elapsed();
                self.meta.timings.total = meta_total.elapsed();
                self.screen.erase();

                self.needs_layout = false;
            }

            self.tick_views();

            let sleep = sleep_micros.saturating_sub(fps_now.elapsed().as_micros()) as u64;
            if sleep > 0 {
                std::thread::sleep(Duration::from_micros(sleep));
            }
            fps_now = Instant::now();
        }
    }
}
