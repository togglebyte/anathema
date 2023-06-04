use anathema::display::{size, Screen, Size};
use anathema::widgets::{Border, Constraints, NodeId, PaintCtx, Pos, Text, Widget};
use std::io::stdout;

fn main() {
    // Get the screen size
    let size = Size::from(size().unwrap());

    // Stdout is our render target
    let mut stdout = stdout();

    // Create a screen to do the rendering
    let mut screen = Screen::new(&mut stdout, size).unwrap();
    screen.clear_all(&mut stdout).unwrap();

    // Setup widgets
    let mut border = Border::thin(None, None).into_container(NodeId::anon());
    let text = Text::with_text("I would like a hot cup of tea").into_container(NodeId::anon());

    let screen_constraints = Constraints::new(size.width, size.height);

    border.add_child(text);
    border.layout(screen_constraints, false);
    border.position(Pos::ZERO);
    let paint_ctx = PaintCtx::new(&mut screen, None);
    border.paint(paint_ctx);

    // ... and finally render to stdout
    screen.render(&mut stdout).unwrap();

    // Wait two seconds and then restore the terminal
    std::thread::sleep(std::time::Duration::from_secs(2));
    screen.restore(&mut stdout).unwrap();
}
