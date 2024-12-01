use std::fmt::Write;
use std::ops::ControlFlow;

use anathema_debug::{Debug, DebugWriter};
use anathema_store::tree::visitor::NodeVisitor;
use anathema_store::tree::ValueId;

use crate::expressions::EvalValue;
use crate::nodes::element::Element;
use crate::nodes::loops::{For, Iteration};
use crate::widget::{ValueKey, WidgetTreeView};
use crate::{AttributeStorage, WidgetContainer, WidgetKind, WidgetTree};

pub fn debug_tree<'a, 'b, 'bp>(
    tree: &'a mut WidgetTreeView<'b, 'bp>,
    attribute_storage: &'a AttributeStorage<'bp>,
) -> String {
    Debug::new(String::new())
        .debug("tree", DebugWidgets(tree, attribute_storage))
        .finish()
}

struct EvalValueDebug<'a>(&'a EvalValue<'a>);

impl DebugWriter for EvalValueDebug<'_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        match self.0 {
            EvalValue::Dyn(value_ref) => write!(output, "{} ", usize::from(value_ref.owned_key())),
            EvalValue::Index(value_ref, index_value_ref) => {
                write!(output, "source: ")?;
                Self(value_ref).write(output)?;
                write!(output, "| index: ")?;
                Self(index_value_ref).write(output)
            }
            EvalValue::Empty => write!(output, "<empty>"),
            EvalValue::Static(val) => write!(output, "{val:?}"),
            EvalValue::Pending(pending) => write!(output, "<pending {}>", usize::from(pending.owned_key())),
            // EvalValue::Map(_) => todo!(),
            EvalValue::Map2(_) => todo!(),
            // EvalValue::ExprList(list) => {
            //     write!(output, "[")?;
            //     list.iter().for_each(|val| {
            //         EvalValueDebug(val).write(output).unwrap();
            //         write!(output, ", ").unwrap();
            //     });
            //     write!(output, "]")
            // }
            EvalValue::Negative(_) => todo!(),
            EvalValue::Op(_, _, _) => todo!(),
            EvalValue::Not(_) => todo!(),
            EvalValue::Equality(_, _, _) => todo!(),
            EvalValue::State(_) => todo!(),
            EvalValue::ComponentAttributes(_) => todo!(),
            EvalValue::ExprList(_) => todo!(),
        }
    }
}

struct ElementDebug<'a, 'bp>(&'a Element<'a>, &'a AttributeStorage<'bp>);

impl DebugWriter for ElementDebug<'_, '_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        write!(output, "{} ", self.0.ident)?;

        // Attributes
        let attrs = self.1.get(self.0.container.id);
        let attrib_count = attrs.iter().count();
        if attrib_count > 0 {
            let _ = write!(output, "[");
            for (cntr, (key, value)) in attrs.iter().enumerate() {
                if cntr > 0 {
                    write!(output, ", ")?;
                }
                let ValueKey::Attribute(name) = key else { panic!() };
                write!(output, "{name}: ")?;
                EvalValueDebug(value.inner()).write(output)?;
            }
            let _ = write!(output, "] ");
        }

        // Value
        if let Some(value) = attrs.value() {
            EvalValueDebug(value.inner()).write(output)?;
        }

        Ok(())
    }
}

struct ForDebug<'a>(&'a For<'a>);

impl DebugWriter for ForDebug<'_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        write!(output, "<for")?;
        match self.0.collection() {
            crate::values::Collection::Dyn(value_ref) => {
                write!(output, " {} ", usize::from(value_ref.owned_key()))
            }
            crate::values::Collection::Static(_) => write!(output, " <static> "),
            crate::values::Collection::Static2(_) => write!(output, " <static> "),
            crate::values::Collection::Future => write!(output, " <future> "),
            crate::values::Collection::Index(_, _) => todo!(),
        }?;
        write!(output, ">")
    }
}

struct IterationDebug<'a>(&'a Iteration<'a>);

impl DebugWriter for IterationDebug<'_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        let key = self.0.loop_index.key();
        let value = *self.0.loop_index.to_ref();
        write!(
            output,
            "<iter binding = {}, index = {value}>",
            self.0.binding,
        )
    }
}

struct WidgetDebug<'a, 'bp>(&'a WidgetKind<'a>, &'a AttributeStorage<'bp>);

impl DebugWriter for WidgetDebug<'_, '_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        match self.0 {
            WidgetKind::Element(el) => ElementDebug(el, self.1).write(output),
            WidgetKind::For(forloop) => ForDebug(forloop).write(output),
            WidgetKind::Iteration(iter) => IterationDebug(iter).write(output),
            WidgetKind::ControlFlow(_cf) => write!(output, "<control flow>"),
            WidgetKind::ControlFlowContainer(id) => write!(output, "<control flow container {id}>"),
            // WidgetKind::If(widget) => write!(
            //     output,
            //     "<if cond = {} | show = {}>",
            //     widget.cond.load_bool(),
            //     widget.show
            // ),
            // WidgetKind::Else(widget) => match &widget.cond {
            //     Some(cond) => write!(output, "<else cond = {} | show = {}>", cond.load_bool(), widget.show),
            //     None => write!(output, "<else show = {}>", widget.show),
            // },
            WidgetKind::Component(_) => write!(output, "<component>"),
        }
    }
}

/// Debug print widgets in a tree.
pub struct DebugWidgets<'a, 'b, 'bp>(&'a mut WidgetTreeView<'b, 'bp>, &'a AttributeStorage<'bp>);

impl<'a, 'b, 'bp> DebugWidgets<'a, 'b, 'bp> {
    pub fn new(tree: &'a mut WidgetTreeView<'b, 'bp>, attribute_storage: &'a AttributeStorage<'bp>) -> Self {
        Self(tree, attribute_storage)
    }
}

impl DebugWriter for DebugWidgets<'_, '_, '_> {
    fn write(&mut self, output: &mut impl Write) -> std::fmt::Result {
        let mut visitor = DebugWidgetsVisitor {
            level: 0,
            output,
            attribute_storage: self.1,
        };

        self.0.apply_visitor(&mut visitor);

        Ok(())
    }
}

struct DebugWidgetsVisitor<'a, 'bp, O> {
    level: usize,
    attribute_storage: &'a AttributeStorage<'bp>,
    output: &'a mut O,
}

impl<O: std::fmt::Write> NodeVisitor<WidgetContainer<'_>> for DebugWidgetsVisitor<'_, '_, O> {
    fn visit(&mut self, value: &mut WidgetContainer<'_>, _path: &[u16], _: ValueId) -> ControlFlow<bool> {
        let indent = " ".repeat(self.level * 4);
        write!(self.output, "{indent}").unwrap();
        WidgetDebug(&value.kind, self.attribute_storage)
            .write(self.output)
            .unwrap();
        writeln!(self.output).unwrap();
        ControlFlow::Continue(())
    }

    fn push(&mut self) {
        self.level += 1;
    }

    fn pop(&mut self) {
        self.level -= 1;
    }
}
