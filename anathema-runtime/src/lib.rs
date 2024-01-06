use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use anathema_render::{size, Screen, Size};
use anathema_values::{drain_dirty_nodes, Context};
use anathema_vm::CompiledTemplates;
use anathema_widget_core::contexts::PaintCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::layout::Constraints;
use anathema_widget_core::nodes::{make_it_so, Nodes};
use anathema_widget_core::views::Views;
use anathema_widget_core::{Event, Events, KeyCode, LayoutNodes, Pos};
use anathema_widgets::register_default_widgets;
use crossterm::terminal::enable_raw_mode;
use tabindex::Direction;

use crate::tabindex::TabIndexing;

#[allow(unused_extern_crates)]
extern crate anathema_values as anathema;

mod meta;
mod tabindex;

/// The runtime handles events, tab indices and configuration of the display
///
/// ```
/// # use anathema_runtime::Runtime;
/// # use anathema_vm::CompiledTemplates;
/// # fn run(templates: &CompiledTemplates) {
/// # let expressions = panic![];
/// let mut runtime = Runtime::new(templates).unwrap();
/// runtime.enable_mouse = true;
/// runtime.enable_alt_screen = false;
/// runtime.fps = 120;
/// runtime.run().unwrap();
/// # }
/// ```
pub struct Runtime<'e> {
    /// Enable meta information such as frame timing, screen size and widget count.
    /// ```text
    /// // Meta information available in the template:
    /// _size.width
    /// _size.height
    /// _timings: Timings
    /// _timings.layout
    /// _timings.position
    /// _timings.paint
    /// _timings.render
    /// _timings.total
    /// _count
    /// ```
    pub enable_meta: bool,
    /// Enable mouse support on terminals that supports it
    pub enable_mouse: bool,
    /// This captures the ctrl+c command and terminates the runtime.
    pub enable_ctrlc: bool,
    /// Enable tab indices.
    /// If this is set to false, the root view will receive all events,
    /// if this is set to true, only views with a given tab index will receive
    /// events.
    ///
    /// The root view will never have a tab index
    pub enable_tabindex: bool,
    /// This will create an alternate screen and render to this screen.
    /// This retains the old content of the terminal and restores it once the
    /// runtime terminates.
    pub enable_alt_screen: bool,
    /// Set the target number of frames to render per second.
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
    /// Create a new runtime.
    pub fn new(templates: &'e CompiledTemplates) -> Result<Self> {
        let expressions = templates.expressions();
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
            meta: meta::Meta::new(size.width, size.height),
            tabindex: TabIndexing::new(),
            enable_ctrlc: true,
            enable_tabindex: false,
        };

        Ok(inst)
    }

    fn layout(&mut self) -> Result<()> {
        self.nodes.reset_cache();
        let context = Context::root(&self.meta);

        let mut nodes = LayoutNodes::new(&mut self.nodes, self.constraints, &context);

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

        let state = &self.meta;
        let context = Context::root(state);

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
                return Event::Stop;
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

    /// Consumes the runtime and loops until
    /// either the runtime receives an error or the `Quit` event is triggered.
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
            if let Some(next) = self.tabindex.current_node() {
                self.nodes.with_view(next, |view| view.focus());
            }
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

                        *self.meta._size.width = size.width;
                        *self.meta._size.height = size.height;
                    }
                    Event::Blur => *self.meta._focus = false,
                    Event::Focus => *self.meta._focus = true,
                    Event::Stop => break 'run Ok(()),
                    _ => {}
                }

                let event = if self.enable_tabindex {
                    if let Some(view_id) = self.tabindex.current_node() {
                        self.nodes.with_view(view_id, |view| view.on_event(event))
                    } else {
                        None
                    }
                } else {
                    // TODO: this is a bit sketchy
                    let root = 0.into(); // TODO: this should be a `const`
                    self.nodes.with_view(&root, |view| view.on_event(event))
                };

                if let Some(Event::Stop) = event {
                    break 'run Ok(());
                }
            }

            self.changes();

            *self.meta._count = self.nodes.count();

            // TODO: the meta info should only be updated if `self.enable_meta`
            if self.needs_layout {
                let meta_total = Instant::now();

                self.layout()?;
                *self.meta._timings.layout = format!("{:?}", meta_total.elapsed());

                let now = Instant::now();
                self.position();
                *self.meta._timings.position = format!("{:?}", now.elapsed());

                let now = Instant::now();
                self.paint();
                *self.meta._timings.paint = format!("{:?}", now.elapsed());

                let now = Instant::now();
                self.screen.render(&mut self.output)?;
                *self.meta._timings.render = format!("{:?}", now.elapsed());
                *self.meta._timings.total = format!("{:?}", meta_total.elapsed());
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
