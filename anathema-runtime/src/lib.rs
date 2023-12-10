use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use anathema_render::{size, Screen, Size};
use anathema_values::state::State;
use anathema_values::{drain_dirty_nodes, Context};
use anathema_widget_core::contexts::{LayoutCtx, PaintCtx};
use anathema_widget_core::error::Result;
use anathema_widget_core::expressions::Expression;
use anathema_widget_core::layout::Constraints;
use anathema_widget_core::nodes::{make_it_so, Node, NodeKind, Nodes};
use anathema_widget_core::views::{AnyView, RegisteredViews, TabIndex, View, ViewFn, Views};
use anathema_widget_core::{Event, Events, LayoutNodes, Padding, Pos, KeyCode, KeyModifiers};
use anathema_widgets::register_default_widgets;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub struct Runtime<'e> {
    pub enable_meta: bool,
    pub enable_mouse: bool,
    screen: Screen,
    output: Stdout,
    constraints: Constraints,
    nodes: Nodes<'e>,
    events: Events,
    fps: u8,
    // meta: Meta,
}

impl<'e> Drop for Runtime<'e> {
    fn drop(&mut self) {
        let _ = Screen::show_cursor(&mut self.output);
        let _ = disable_raw_mode();
    }
}

impl<'e> Runtime<'e> {
    pub fn new(expressions: &'e [Expression]) -> Result<Self> {
        let nodes = make_it_so(expressions);

        register_default_widgets()?;
        enable_raw_mode()?;

        let mut stdout = stdout();
        Screen::hide_cursor(&mut stdout)?;

        let size: Size = size()?.into();
        let constraints = Constraints::new(Some(size.width), Some(size.height));
        let screen = Screen::new(size);

        let inst = Self {
            output: stdout,
            screen,
            constraints,
            nodes,
            enable_meta: false,
            enable_mouse: false,
            events: Events,
            fps: 30,
            // meta: Meta::new(size),
        };

        Ok(inst)
    }

    fn layout(&mut self) -> Result<()> {
        self.nodes.reset_cache();
        let context = Context::root(&());
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

        let context = Context::root(&());

        for (node_id, change) in dirty_nodes {
            self.nodes.update(node_id.as_slice(), &change, &context);
        }

        // TODO: finish this. Need to figure out a good way to notify that
        //       a values should have a sub removed.
        // for node in removed_nodes() {
        // }
    }

    fn tick_views(&mut self) {
        let views = Views::for_each(|node_id| {
            if let Some(Node { kind: NodeKind::View(view), .. }) = self.nodes.query().get(node_id) {
                view.tick();
            }
        });
    }

    fn global_event(&mut self, event: Event) -> Event {
        // -----------------------------------------------------------------------------
        //   - Ctrl-c to quite -
        //   This should be on by default.
        //   Give it a good name
        // -----------------------------------------------------------------------------
        if let Event::CtrlC = event {
            return Event::Quit;
        }

        // -----------------------------------------------------------------------------
        //   - Handle tabbing between widgets -
        //   TODO: this should be behind a setting on the runtime
        //   Just need to come up with a good name for it.
        //   Should probably be on by default
        // -----------------------------------------------------------------------------
        if let Event::KeyPress(KeyCode::Tab, modifiers, ..) = event {
            if let Some(current) = TabIndex::current() {
                if let Some(Node { kind: NodeKind::View(view), .. }) = self.nodes.query().get(&current) {
                    view.blur();
                }
            }

            if modifiers.contains(KeyModifiers::SHIFT) {
                TabIndex::prev();
            } else {
                TabIndex::next();
            }

            if let Some(current) = TabIndex::current() {
                if let Some(Node { kind: NodeKind::View(view), .. }) = self.nodes.query().get(&current) {
                    view.focus();
                }
            }
        }

        event
    }

    pub fn run(mut self) -> Result<()> {
        if self.enable_mouse {
            Screen::enable_mouse(&mut self.output)?;
        }

        self.screen.clear_all(&mut self.output)?;

        let mut now = Instant::now();
        let sleep_micros = ((1.0 / self.fps as f64) * 1000.0 * 1000.0) as u128;

        'run: loop {
            while let Some(event) = self.events.poll(Duration::from_millis(1)) {
                let event = self.global_event(event);

                // Make sure event handling isn't holding up the rest of the event loop.
                if now.elapsed().as_micros() > sleep_micros {
                    break
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

                        // self.meta.size = size;
                    }
                    Event::Blur => (),  //self.meta.focus = false,
                    Event::Focus => (), //self.meta.focus = true,
                    Event::Quit => break 'run Ok(()),
                    _ => {}
                }

                if let Some(id) = TabIndex::current() {
                    if let Some(Node {
                        kind: NodeKind::View(view),
                        ..
                    }) = self.nodes.query().get(&id)
                    {
                        view.on_event(event);
                    }
                }
            }

            self.changes();

            // *self.meta.count = self.nodes.count();
            let _total = Instant::now();
            self.layout()?;
            // *self.meta.timings.layout = total.elapsed();

            let _now = Instant::now();
            self.position();
            // *self.meta.timings.position = now.elapsed();

            let _now = Instant::now();
            self.paint();
            // *self.meta.timings.paint = now.elapsed();

            let _now = Instant::now();
            self.screen.render(&mut self.output)?;
            // *self.meta.timings.render = now.elapsed();
            // *self.meta.timings.total = total.elapsed();
            self.screen.erase();

            if self.enable_meta {}

            self.tick_views();

            let sleep = sleep_micros.saturating_sub(now.elapsed().as_micros()) as u64;
            if sleep > 0 {
                std::thread::sleep(Duration::from_micros(sleep));
            }
            now = Instant::now();
        }
    }
}
