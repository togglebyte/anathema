use anathema_geometry::Size;
use anathema_widgets::components::events::{Event, KeyEvent};

use crate::TestRuntime;

pub(crate) struct TestCase<'src> {
    setup: Setup<'src>,
    steps: Vec<Step>,
}

impl<'src> TestCase<'src> {
    pub fn new(setup: Setup<'src>, steps: Vec<Step>) -> Self {
        Self { setup, steps }
    }

    pub(crate) fn title(&self) -> &str {
        match self.setup.title {
            Some(desc) => desc,
            None => self.setup.template,
        }
    }

    pub(crate) fn run(self, runtime: &TestRuntime) -> bool {
        for step in self.steps {
            match step {
                Step::Tick => {
                    // runtime.tick();
                }
                Step::KeyPress(key_event) => {
                    let event = Event::Key(key_event);
                    // runtime.handle(event);
                }
                Step::Resize(size) => {
                    let event = Event::Resize(size);
                    // runtime.handle(event);
                }
                Step::Expect(template) => {
                    // compare runtime backend buffer to what the output of this template would be
                    // if template output != runtime.buffer { break false }
                    todo!();
                }
            }
        }

        true
    }
}

pub(crate) struct Setup<'src> {
    pub(crate) title: Option<&'src str>,
    pub(crate) template: &'src str,
    pub(crate) size: Size,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Step {
    Tick,
    KeyPress(KeyEvent),
    Resize(Size),
    Expect(String), // Needs a buffer for comparison
}
