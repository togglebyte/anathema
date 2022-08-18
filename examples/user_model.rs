use std::collections::HashMap;

use anathema::display::{events::MouseButton, Color, Style};
use anathema::runtime::{Event, Runtime, Sender, UserModel};
use anathema::templates::DataCtx;
use anathema::widgets::{Canvas, NodeId, Value, WidgetContainer};

static TEMPLATE: &str = r#"
vstack [padding-left: 2, padding-right: 2, padding: 1]:
    hstack:

        // Colours
        // -------
        for [data: {{ colors }}, binding: color]:
            border [id: {{ color.id }}, width: 4, height: 3, background: {{ color.color }}, foreground: {{ color.selected }}]:

        // Selected brush
        // --------------
        border [height: 3]:
            text: "brush: {{ brush }}"

        // Help
        // ----
        spacer:
        text: "Scroll mouse wheel up to cycle brushes"

    // Canvas to draw on
    // -----------------
    border:
        canvas [id: "canvas"]:
"#;

// -----------------------------------------------------------------------------
//     - Color -
//     Represent a color in the color selector
// -----------------------------------------------------------------------------
#[derive(Debug, Clone)]
struct ColorItem {
    color: Color,
    selected: Color,
    id: u64,
}

impl From<ColorItem> for Value {
    fn from(color: ColorItem) -> Value {
        let mut hm = HashMap::new();
        hm.insert("id".to_string(), color.id.into());
        hm.insert("selected".to_string(), color.selected.into());
        hm.insert("color".to_string(), color.color.into());
        Value::Map(hm)
    }
}

// -----------------------------------------------------------------------------
//     - User model -
//     This example exists to show off an implementation of
//     the `UserModel` trait.
//
//     The state is used to store the list of colors, the selected brush and the
//     selected color.
// -----------------------------------------------------------------------------
struct State {
    tx: Sender<()>,
    data: DataCtx,
    selected_color: usize,
    brush: char,
    colors: Vec<ColorItem>,
}

impl State {
    // Create a new instanc of the `State`.
    // Assign a character to the brush and setup the colors
    fn new(tx: Sender<()>) -> Self {
        let brush = '░';
        let colors = vec![
            ColorItem { color: Color::White, selected: Color::White, id: 0 },
            ColorItem { color: Color::Red, selected: Color::Black, id: 1 },
            ColorItem { color: Color::Green, selected: Color::Black, id: 2 },
        ];

        let mut data = DataCtx::empty();
        data.set("brush", brush.to_string());
        data.set("colors", colors.clone());
        Self { tx, data, selected_color: 0, brush, colors }
    }
}

// -----------------------------------------------------------------------------
//     - Implement the `UserModel` -
// -----------------------------------------------------------------------------
impl UserModel for State {
    type Message = ();

    fn event(&mut self, event: Event<Self::Message>, root: &mut WidgetContainer) {
        if event.ctrl_c() {
            let _ = self.tx.send(Event::Quit);
        }

        // Scrolling the mouse wheel changes the brush
        if event.scroll_up().is_some() {
            match self.brush {
                '░' => self.brush = '▒',
                '▒' => self.brush = '▓',
                '▓' => self.brush = '█',
                '█' => self.brush = '░',
                _ => {}
            }
            self.data.set("brush", self.brush.to_string());
        }

        // Select color by finding the widget under the cursor
        if let Some((screen_pos, _btn, _modifiers)) = event.mouse_down() {
            root.at_coords(screen_pos, |w| match w.id() {
                NodeId::Value(val) => match val.to_int() {
                    Some(id) => {
                        self.selected_color = id as usize;
                        self.colors.iter_mut().for_each(|c| {
                            if c.id == id {
                                c.selected = Color::White;
                            } else {
                                c.selected = Color::Black;
                            }
                        });
                        self.data.set("colors", self.colors.clone());
                        false
                    }
                    None => true,
                },
                _ => true,
            });
        }

        // Paint or erase when the mouse is dragged
        if let Some((screen_pos, btn, _modifiers)) = event.mouse_drag() {
            let widget = root.by_id("canvas").unwrap();
            if let Some(pos) = widget.screen_to_local(screen_pos) {
                let canvas = widget.to::<Canvas>();

                match btn {
                    MouseButton::Left => {
                        let mut style = Style::new();
                        let color = &self.colors[self.selected_color].color;
                        style.set_fg(*color);
                        canvas.put(self.brush, style, pos);
                    }
                    MouseButton::Right => canvas.clear(pos),
                    _ => {}
                }
            }
        }
    }

    fn data(&mut self) -> &mut DataCtx {
        &mut self.data
    }
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.output_cfg.enable_mouse = true;
    runtime.output_cfg.alt_screen = false;
    let state = State::new(runtime.sender());

    if let Err(e) = runtime.with_usermodel(TEMPLATE, state) {
        eprintln!("{e}");
    }
}
