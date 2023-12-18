use std::cmp::Ordering;
use std::io::{stdout, Stdout};
use std::time::{Duration, Instant};

use anathema_render::{size, Screen, Size};
use anathema_values::{drain_dirty_nodes, Context, NodeId};
use anathema_widget_core::contexts::PaintCtx;
use anathema_widget_core::error::Result;
use anathema_widget_core::expressions::Expression;
use anathema_widget_core::layout::Constraints;
use anathema_widget_core::nodes::{make_it_so, Node, NodeKind, Nodes};
use anathema_widget_core::views::Views;
use anathema_widget_core::{Event, Events, KeyCode, KeyModifiers, LayoutNodes, Padding, Pos};
use anathema_widgets::register_default_widgets;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod meta;

pub struct Runtime<'e> {
    pub enable_meta: bool,
    pub enable_mouse: bool,
    screen: Screen,
    output: Stdout,
    constraints: Constraints,
    nodes: Nodes<'e>,
    events: Events,
    pub fps: u8,
    needs_layout: bool,
    meta: meta::Meta,
    tabindex: Option<u32>,
    current_focus: Option<NodeId>,
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
            needs_layout: true,
            meta: meta::Meta::new(size),
            tabindex: None,
            current_focus: None,
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

        self.needs_layout = true;
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
        Views::for_each(|node_id, _| {
            if let Some(Node {
                kind: NodeKind::View(view),
                ..
            }) = self.nodes.query().get(node_id)
            {
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
        //
        //   This is also a massive hack to try things out...
        // -----------------------------------------------------------------------------
        // -----------------------------------------------------------------------------
        //   - This is an awful hack for now -
        // -----------------------------------------------------------------------------
        if let Event::KeyPress(KeyCode::Tab, modifiers, ..) = event {
            let views = Views::all();
            let mut values = views
                .iter()
                .filter_map(|f| Some((f.key(), f.value?)))
                .collect::<Vec<_>>();
            values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for ((node_id, tabindex)) in &values {
                match self.tabindex {
                    None => {
                        self.tabindex = Some(*tabindex);
                        self.focus_widget((*node_id).clone());
                        return event;
                    }
                    Some(current_tabindex) => {
                        match current_tabindex.cmp(tabindex) {
                            Ordering::Less => {
                                // tab
                                if !modifiers.contains(KeyModifiers::SHIFT) {
                                    self.tabindex = Some(*tabindex);
                                    self.focus_widget((*node_id).clone());
                                    return event;
                                }
                            }
                            Ordering::Equal => {}
                            Ordering::Greater => {
                                // Shift tab
                                if modifiers.contains(KeyModifiers::SHIFT) {
                                    self.tabindex = Some(*tabindex);
                                    self.focus_widget((*node_id).clone());
                                    return event;
                                }
                            }
                        }
                    }
                    Some(_) => continue,
                }
            }

            values.first().map(|(node_id, tabindex)| {
                self.tabindex = Some(*tabindex);
                self.focus_widget((*node_id).clone());
            });
        }

        event
    }

    fn focus_widget(&mut self, node_id: NodeId) {
        if let Some(Node {
            kind: NodeKind::View(view),
            ..
        }) = self.nodes.query().get(&node_id)
        {
            view.focus();

            if let Some(old) = self.current_focus.take() {
                if let Some(Node {
                    kind: NodeKind::View(old_view),
                    ..
                }) = self.nodes.query().get(&old)
                {
                    old_view.blur();
                }
            }

            self.current_focus = Some(node_id);
        }
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

                if let Some(view_id) = self.current_focus.as_ref() {
                    if let Some(Node {
                        kind: NodeKind::View(view),
                        ..
                    }) = self.nodes.query().get(view_id)
                    {
                        view.on_event(event);
                    }
                }

            }

            self.changes();

            self.meta.count = self.nodes.count();
            let total = Instant::now();

            if self.needs_layout {
                self.layout()?;
                self.meta.timings.layout = total.elapsed();

                let _now = Instant::now();
                self.position();
                self.meta.timings.position = now.elapsed();

                let _now = Instant::now();
                self.paint();

                self.meta.timings.paint = now.elapsed();

                let _now = Instant::now();
                self.screen.render(&mut self.output)?;
                self.meta.timings.render = now.elapsed();
                self.meta.timings.total = total.elapsed();
                self.screen.erase();

                self.needs_layout = false;
            }

            self.tick_views();

            let sleep = sleep_micros.saturating_sub(now.elapsed().as_micros()) as u64;
            if sleep > 0 {
                std::thread::sleep(Duration::from_micros(sleep));
            }
            now = Instant::now();
        }
    }
}
