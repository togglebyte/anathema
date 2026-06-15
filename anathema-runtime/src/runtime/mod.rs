use anathema_compiler::Document;

use crate::widgets::RegisteredWidgets;

pub struct Runtime<Fe> {
    frontend: Fe,
    doc: Document,
    widget_reg: RegisteredWidgets,
}

impl<Fe> Runtime<Fe> {
    pub fn new(doc: Document, frontend: Fe) -> Self {
        Self { frontend, doc, widget_reg: RegisteredWidgets::empty() }
    }

    pub fn run(&mut self) {
        loop {
            // * Layout
            // * Position
            // * Paint
            // * Events
            // * Messages
            // * Deferred events

            std::thread::sleep_ms(20);
        }
    }
}
