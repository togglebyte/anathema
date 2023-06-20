use std::io::{stdout, Stdout};
use std::thread;
use std::time::Duration;

use anathema_render::{size, Screen};
use anathema_widgets::error::Result;
use anathema_widgets::view::View;
use anathema_widgets::{Constraints, Lookup};

pub struct Runtime {
    views: Vec<Box<dyn View>>,
    screen: Screen,
    output: Stdout,
    lookup: Lookup,
    constraints: Constraints,
}

impl Runtime {
    pub fn new() -> Result<Self> {
        let mut stdout = stdout();
        let (width, height) = size()?;
        let constraints = Constraints::new(Some(width as usize), Some(height as usize));
        Screen::hide_cursor(&mut stdout)?;
        let screen = Screen::new((width, height));
        let lookup = Lookup::default();

        let inst = Self {
            output: stdout,
            screen,
            views: vec![],
            lookup,
            constraints,
        };

        Ok(inst)
    }

    pub fn load_view(&mut self, view: Box<dyn View>) {
        self.views.push(view);
    }

    fn views(&mut self) -> Result<()> {
        for view in &mut self.views {
            view.update();
            view.render(&self.lookup, self.constraints, &mut self.screen)?;
        }

        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        self.screen.clear_all(&mut self.output)?;

        loop {
            self.views()?;

            self.screen.render(&mut self.output)?;

            thread::sleep(Duration::from_millis(500));
            self.screen.erase();
        }
        Ok(())
    }
}
