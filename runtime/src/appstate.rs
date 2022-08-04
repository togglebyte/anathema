use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use display::{Screen, Size};
use templates::Node;
use templates::{NodeCtx, IncludeCache, DataCtx, SubContext, WidgetLookup, WidgetNode};
use widgets::{Constraints, PaintCtx, Pos, Value, WidgetContainer};

use crate::error::{Error, Result};
use crate::Output;

pub use crate::events::{Event, Events, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

#[cfg(feature = "flume")]
pub(crate) type Receiver<T> = flume::Receiver<Event<T>>;

#[cfg(feature = "flume")]
pub type Sender<T> = flume::Sender<Event<T>>;

#[cfg(not(feature = "flume"))]
pub(crate) type Receiver<T> = std::sync::mpsc::Receiver<Event<T>>;

#[cfg(not(feature = "flume"))]
pub type Sender<T> = std::sync::mpsc::Sender<Event<T>>;

pub enum WaitFor {
    Timeout(Duration),
    Input,
}

pub enum Run {
    Continue,
    Quit,
}

fn extra_context(size: Size, metrics: &Metrics) -> Value {
    let mut hm = HashMap::new();
    let size = HashMap::from([
        ("width".to_string(), Value::from(size.width as u64)),
        ("height".to_string(), Value::from(size.height as u64)),
    ]);

    let metrics = HashMap::from([
        ("render".to_string(), Value::from(format!("{:?}", metrics.render_time))),
        ("update".to_string(), Value::from(format!("{:?}", metrics.update_time))),
        ("layout".to_string(), Value::from(format!("{:?}", metrics.layout_time))),
        ("paint".to_string(), Value::from(format!("{:?}", metrics.paint_time))),
        ("frame".to_string(), Value::from(format!("{:?}", metrics.frame_time))),
    ]);

    hm.insert("size".into(), Value::Map(size));
    hm.insert("metrics".into(), Value::Map(metrics));

    Value::Map(hm)
}

pub struct Metrics {
    pub render_time: Duration,
    pub update_time: Duration,
    pub layout_time: Duration,
    pub paint_time: Duration,
    pub frame_time: Duration,
}

// -----------------------------------------------------------------------------
//     - App state -
// -----------------------------------------------------------------------------
/// ```ignore
/// struct Model { counter: usize }
/// let user_model = Model { counter: 0 };
/// let (term_size, output) = Output::stdout().unwrap();
///
/// let root = Constraints::new(term_size.width, term_size.height);
/// ```
pub struct AppState<T: UserModel, O: Output> {
    pub nodes: Vec<WidgetNode>,
    pub old_nodes: Vec<Node>,
    pub user_model: T,
    root: WidgetContainer,
    events: VecDeque<Event<T::Message>>,
    events_src: Events<T::Message>,
    screen: Screen,
    output: O,
    wait_for: WaitFor,
    widget_lookup: WidgetLookup,
    include_cache: IncludeCache,

    tick: Instant,
    metrics: Metrics,
}

impl<T: UserModel, O: Output> AppState<T, O> {
    pub fn new(
        mut user_model: T,
        events_src: Events<T::Message>,
        nodes: Vec<WidgetNode>,
        widget_lookup: WidgetLookup,
        mut output: O,
        wait_for: WaitFor,
    ) -> Result<Self> {
        let size = output.size();
        let mut screen = Screen::new(&mut output, size)?;
        screen.clear_all(&mut output)?;

        let ctx = user_model.data();
        // ctx.insert("context", extra_context(size, &Metrics::new()));
        let sub_context = SubContext::new(ctx);

        let mut include_cache = IncludeCache::default();
        let mut node_ctx = NodeCtx::new(&mut include_cache);
        let mut widget_containers = templates::build_widget_tree(&widget_lookup, &nodes, &sub_context, &mut node_ctx)?;
        if widget_containers.is_empty() {
            return Err(Error::MissingRoot);
        }
        let root = widget_containers.remove(0);
        let old_nodes = templates::to_nodes(&nodes, &sub_context, &mut node_ctx)?;

        let inst = Self {
            nodes,
            old_nodes,
            root,
            user_model,
            widget_lookup,
            events: VecDeque::new(),
            events_src,
            screen,
            output,
            wait_for,
            include_cache,

            tick: Instant::now(),
            metrics: Metrics {
                render_time: Duration::new(0, 0),
                update_time: Duration::new(0, 0),
                layout_time: Duration::new(0, 0),
                paint_time: Duration::new(0, 0),
                frame_time: Duration::new(0, 0),
            },
        };

        Ok(inst)
    }

    fn handle_events(&mut self, blocking: bool) -> Result<Run> {
        while let Some(event) = self.events_src.next_event(blocking) {
            if let Event::Quit = event {
                return Ok(Run::Quit);
            }

            if let Event::Resize(new_size) = event {
                self.root.resize(new_size);

                // 1. `erase` the buffers
                // 2.  render
                // 3.  create new buffers with correct size
                // 4.  layout
                // 5.  render

                self.screen.resize(new_size);
                self.screen.clear_all(&mut self.output)?;
            }

            self.events.push_back(event);

            // If the event loop is entirely driven by events
            // then this can't loop
            if blocking {
                break;
            }
        }

        Ok(Run::Continue)
    }

    pub fn update(&mut self) -> Result<()> {
        let now = Instant::now();

        while let Some(event) = self.events.pop_front() {
            self.user_model.event(event, &mut self.root);
        }

        let ctx = self.user_model.data();
        let diff = ctx.diff();

        if !diff.is_empty() {
            let mut sub_context = SubContext::new(ctx);
            sub_context.insert("context", extra_context(self.screen.size(), &self.metrics));
            let mut node_ctx = NodeCtx::new(&mut self.include_cache);
            let new_nodes = templates::to_nodes(&self.nodes, &sub_context, &mut node_ctx)?;

            if !self.old_nodes.is_empty() {
                let changes = templates::diff::diff(&new_nodes[0], self.old_nodes.remove(0));
                if !changes.is_empty() {
                    changes.apply(&mut self.root, &self.widget_lookup, &new_nodes);
                }
            }

            self.old_nodes = new_nodes;
        }

        self.metrics.update_time = now.elapsed();
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let size = self.screen.size();
        let constraints = Constraints::new(size.width, size.height);

        // Layout
        let now = Instant::now();
        let dt = self.tick.elapsed();
        self.root.animate(dt);
        self.root.layout(constraints, false);
        self.root.position(Pos::ZERO);
        self.metrics.layout_time = now.elapsed();
        self.tick = Instant::now();

        // Paint
        let ctx = PaintCtx::new(&mut self.screen, None);
        let now = Instant::now();
        self.root.paint(ctx);
        self.metrics.paint_time = now.elapsed();

        // ... and render
        let now = Instant::now();
        self.screen.render(&mut self.output)?;
        self.screen.erase();
        self.metrics.render_time = now.elapsed();

        Ok(())
    }

    pub fn wait_for(&mut self) -> Result<Run> {
        let now = Instant::now();

        // Update
        self.update()?;

        // Render
        self.render()?;

        self.metrics.frame_time = now.elapsed();

        let run = match self.wait_for {
            WaitFor::Timeout(duration) => {
                std::thread::sleep(duration);
                let blocking = false;
                self.handle_events(blocking)?
            }
            WaitFor::Input => {
                let blocking = true;
                self.handle_events(blocking)?
            }
        };
        Ok(run)
    }
}

// -----------------------------------------------------------------------------
//     - User model -
// -----------------------------------------------------------------------------
pub trait UserModel {
    type Message: Send + Sync + 'static;

    fn event(&mut self, event: Event<Self::Message>, root: &mut WidgetContainer);

    fn data(&mut self) -> &mut DataCtx;
}

