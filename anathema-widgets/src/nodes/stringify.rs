use std::fmt::Write;
use std::ops::ControlFlow;

use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::ValueId;
use anathema_value_resolver::AttributeStorage;

use super::element::Element;
use super::WidgetContainer;
use crate::WidgetKind;

/// Stringify the tree.
/// Used for debugging
pub struct Stringify<'a, 'bp> {
    level: usize,
    indent: String,
    output: String,
    attribute_storage: &'a AttributeStorage<'bp>,
}

impl<'a, 'bp> Stringify<'a, 'bp> {
    pub fn new(attribute_storage: &'a AttributeStorage<'bp>) -> Self {
        Self {
            level: 0,
            indent: String::new(),
            output: String::new(),
            attribute_storage,
        }
    }

    pub fn finish(self) -> String {
        self.output
    }
}

impl<'a, 'bp> NodeVisitor<WidgetContainer<'_>> for Stringify<'a, 'bp> {
    fn visit(&mut self, value: &mut WidgetContainer<'_>, _path: &[u16], _: ValueId) -> ControlFlow<bool> {
        let _ = write!(&mut self.output, "{}", self.indent);
        match &value.kind {
            WidgetKind::Element(Element { ident, container, .. }) => {
                let _ = write!(&mut self.output, "{ident}");

                let attribs = self.attribute_storage.get(container.id);
                let attrib_count = attribs.iter().count();
                if attrib_count > 0 {
                    // Print attributes
                    let _ = write!(&mut self.output, "[");
                    for (i, (key, val)) in attribs.iter().enumerate() {
                        // Write a comma before the values if this is not the first entry
                        if i > 0 {
                            let _ = write!(&mut self.output, ", ");
                        }
                        let _ = write!(&mut self.output, "{}: {:?}", key.as_str(), val);
                    }
                    let _ = write!(&mut self.output, "]");
                }

                if let Some(val) = attribs.value() {
                    let _ = write!(&mut self.output, " {:?}", val);
                }
            }
            WidgetKind::For(_) => drop(write!(&mut self.output, "<for>")),
            WidgetKind::Iteration(iteration) => {
                let _ = write!(
                    &mut self.output,
                    "<iter binding = {}, index = {}>",
                    iteration.binding,
                    iteration.loop_index.copy_value()
                );
            }
            WidgetKind::ControlFlow(_) => {
                let _ = write!(&mut self.output, "<control flow>");
            }
            WidgetKind::ControlFlowContainer(id) => {
                let _ = write!(&mut self.output, "<control flow container {id}>");
            }
            // WidgetKind::If(if_widget) => {
            //     let _ = write!(&mut self.output, "<if cond = {}>", if_widget.cond.load_bool());
            // }
            // WidgetKind::Else(if_widget) => match &if_widget.cond {
            //     Some(cond) => {
            //         let _ = write!(&mut self.output, "<else cond = {}>", cond.load_bool());
            //     }
            //     None => drop(write!(&mut self.output, "<else>")),
            // },
            WidgetKind::Component(_) => drop(write!(&mut self.output, "<component>")),
            WidgetKind::Slot => todo!(),
        }

        let _ = writeln!(&mut self.output);

        ControlFlow::Continue(())
    }

    fn push(&mut self) {
        self.level += 1;
        self.indent = " ".repeat(self.level * 4);
    }

    fn pop(&mut self) {
        self.level -= 1;
        self.indent = " ".repeat(self.level * 4);
    }
}
