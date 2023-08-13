use anathema_compiler::{Constants, Instruction, StringId};
use anathema_generator::{ControlFlowExpr, Expression, ExpressionValue, TextExpr};
use anathema_values::StoreMut;
use anathema_widget_core::{Value, WidgetContainer, WidgetMeta};

use crate::error::Result;
use crate::Expressions;

static FILE_BUG_REPORT: &str =
    "consts have been modified, this is a bug with Anathema, file a bug report please";

pub(crate) struct Scope<'vm> {
    instructions: Vec<Instruction>,
    consts: &'vm Constants,
}

impl<'vm> Scope<'vm> {
    pub fn new(instructions: Vec<Instruction>, consts: &'vm Constants) -> Self {
        Self {
            instructions,
            consts,
        }
    }

    pub fn exec(&mut self, bucket: &mut StoreMut<'_, Value>) -> Result<Expressions> {
        let mut nodes = vec![];

        if self.instructions.is_empty() {
            return Ok(nodes);
        }

        loop {
            let instruction = self.instructions.remove(0);
            match instruction {
                Instruction::View(id) => {
                    let id = self
                        .consts
                        .lookup_value(id)
                        .cloned()
                        .expect(FILE_BUG_REPORT);
                    // nodes.push(Template::View(id));
                    panic!("need to rethink views")
                }
                Instruction::Node { ident, scope_size } => {
                    nodes.push(self.node(ident, scope_size, bucket)?)
                }
                Instruction::For {
                    binding,
                    data,
                    size,
                } => {
                    let binding = self.consts.lookup_string(binding).expect(FILE_BUG_REPORT);
                    let binding = bucket.insert_path(binding);

                    let collection = self
                        .consts
                        .lookup_value(data)
                        .cloned()
                        .expect(FILE_BUG_REPORT);

                    let body = self.instructions.drain(..size).collect();
                    let body = Scope::new(body, &self.consts).exec(bucket)?;
                    let template = Expression::Loop {
                        binding,
                        collection,
                        body: body.into(),
                    };

                    nodes.push(template);
                }
                Instruction::If { cond, size } => {
                    let cond = self
                        .consts
                        .lookup_value(cond)
                        .cloned()
                        .expect(FILE_BUG_REPORT);

                    let body = self.instructions.drain(..size).collect::<Vec<_>>();
                    let body = Scope::new(body, &self.consts).exec(bucket)?;

                    let mut control_flow = vec![];
                    control_flow.push((ControlFlowExpr::If(cond), body.into()));

                    loop {
                        let Some(&Instruction::Else { cond, size }) = self.instructions.get(0)
                        else {
                            break;
                        };
                        let cond = cond.map(|cond| {
                            self.consts
                                .lookup_value(cond)
                                .cloned()
                                .expect(FILE_BUG_REPORT)
                        });

                        let body = self.instructions.drain(..size).collect();
                        let body = Scope::new(body, &self.consts).exec(bucket)?;

                        control_flow.push((ControlFlowExpr::Else(cond), body.into()));
                    }

                    let template = Expression::ControlFlow(control_flow.into());
                    nodes.push(template);
                }
                Instruction::Else { .. } => {
                    unreachable!("the `Else` instructions are consumed inside the `If` instruction")
                }
                Instruction::LoadAttribute { .. } | Instruction::LoadText(_) => {
                    unreachable!("these instructions are only loaded in the `node` function")
                }
            }

            if self.instructions.is_empty() {
                break;
            }
        }

        Ok(nodes)
    }

    fn node(
        &mut self,
        ident: StringId,
        scope_size: usize,
        bucket: &mut StoreMut<'_, Value>,
    ) -> Result<Expression<WidgetContainer>> {
        let ident = self.consts.lookup_string(ident).expect(FILE_BUG_REPORT);

        let mut attributes = ExpressionValue::empty();
        let mut text = None::<TextExpr>;
        let mut ip = 0;

        loop {
            match self.instructions.get(ip) {
                Some(Instruction::LoadAttribute { key, value }) => {
                    let key = self.consts.lookup_string(*key).expect(FILE_BUG_REPORT);
                    let value = self.consts.lookup_value(*value).expect(FILE_BUG_REPORT);
                    attributes.set(key.to_string(), value.clone());
                }
                Some(Instruction::LoadText(i)) => text = self.consts.lookup_value(*i).cloned(),
                _ => break,
            }
            ip += 1;
        }

        // Remove processed attribute and text instructions
        self.instructions.drain(..ip);

        let scope = self.instructions.drain(..scope_size).collect();
        let children = Scope::new(scope, &self.consts).exec(bucket)?;

        let meta = WidgetMeta {
            ident: ident.to_string(),
            text,
        };

        let node = Expression::Node {
            context: meta,
            attributes,
            children: children.into(),
        };

        Ok(node)
    }
}
