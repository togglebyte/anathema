use anathema_compiler::{Constants, Instruction, StringId};
use anathema_values::{Attributes, ValueExpr};
use anathema_widget_core::expressions::{
    ControlFlow, ElseExpr, Expression, IfExpr, LoopExpr, SingleNodeExpr, ViewExpr,
};

use crate::{error::Result, ViewTemplates};

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

    pub fn exec(&mut self, views: &mut ViewTemplates) -> Result<Vec<Expression>> {
        let mut nodes = vec![];

        if self.instructions.is_empty() {
            return Ok(nodes);
        }

        loop {
            let instruction = self.instructions.remove(0);
            match instruction {
                Instruction::View(ident) => {
                    nodes.push(self.view(ident, views)?);
                }
                Instruction::Node { ident, scope_size } => {
                    nodes.push(self.node(ident, scope_size, views)?)
                }
                Instruction::For {
                    binding,
                    data,
                    size,
                } => {
                    let binding = self.consts.lookup_string(binding);

                    let collection = self.consts.lookup_value(data).clone();

                    let body = self.instructions.drain(..size).collect();
                    let body = Scope::new(body, &self.consts).exec(views)?;
                    let template = Expression::Loop(LoopExpr {
                        binding: binding.into(),
                        collection,
                        body,
                    });

                    nodes.push(template);
                }
                Instruction::If { cond, size } => {
                    let cond = self.consts.lookup_value(cond);

                    let body = self.instructions.drain(..size).collect::<Vec<_>>();
                    let body = Scope::new(body, &self.consts).exec(views)?;

                    let mut control_flow = ControlFlow {
                        if_expr: IfExpr {
                            cond,
                            expressions: body,
                        },
                        elses: vec![],
                    };

                    loop {
                        let Some(&Instruction::Else { cond, size }) = self.instructions.get(0)
                        else {
                            break;
                        };
                        self.instructions.remove(0);
                        let cond = cond.map(|cond| self.consts.lookup_value(cond));

                        let body = self.instructions.drain(..size).collect();
                        let body = Scope::new(body, &self.consts).exec(views)?;

                        control_flow.elses.push(ElseExpr {
                            cond,
                            expressions: body,
                        });
                    }

                    let template = Expression::ControlFlow(control_flow.into());
                    nodes.push(template);
                }
                Instruction::Else { .. } => {
                    unreachable!("the `Else` instructions are consumed inside the `If` instruction")
                }
                Instruction::LoadAttribute { .. } | Instruction::LoadValue(_) => {
                    unreachable!("these instructions are only executed in the `node` function")
                }
            }

            if self.instructions.is_empty() {
                break;
            }
        }

        Ok(nodes)
    }

    fn node(&mut self, ident: StringId, scope_size: usize, views: &mut ViewTemplates) -> Result<Expression> {
        let ident = self.consts.lookup_string(ident);

        let mut attributes = Attributes::new();
        let mut text = None::<ValueExpr>;
        let mut ip = 0;

        loop {
            match self.instructions.get(ip) {
                Some(Instruction::LoadAttribute { key, value }) => {
                    let key = self.consts.lookup_string(*key);
                    let value = self.consts.lookup_value(*value);
                    attributes.insert(key.to_string(), value.clone());
                }
                Some(Instruction::LoadValue(i)) => {
                    text = Some(self.consts.lookup_value(*i).clone())
                }
                _ => break,
            }
            ip += 1;
        }

        // Remove processed attribute and text instructions
        self.instructions.drain(..ip);

        let scope = self.instructions.drain(..scope_size).collect();
        let children = Scope::new(scope, &self.consts).exec(views)?;

        let node = Expression::Node(SingleNodeExpr {
            ident: ident.to_string(),
            text,
            attributes,
            children,
        });

        Ok(node)
    }

    fn view(&mut self, ident: StringId, views: &mut ViewTemplates) -> Result<Expression> {
        let ident = self.consts.lookup_string(ident).to_owned();

        let state = match self.instructions.get(0) {
            Some(Instruction::LoadValue(i)) => {
                let val = self.consts.lookup_value(*i).clone();
                let _ = self.instructions.remove(0);
                Some(val)
            }
            _ => None,
        };

        let body = views.get(&ident)?;

        let node = Expression::View(ViewExpr {
            id: ident,
            body,
            state,
        });

        Ok(node)
    }
}
