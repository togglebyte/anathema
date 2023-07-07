use anathema_compiler::{Constants, Instruction};
use anathema_widget_core::template::{Cond, ControlFlow, Template};
use anathema_widget_core::{Attributes, NodeId, TextPath};

use crate::error::Result;

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

    pub fn exec(&mut self, node_id: NodeId) -> Result<Vec<Template>> {
        let mut nodes = vec![];

        if self.instructions.is_empty() {
            return Ok(nodes);
        }

        let mut next = 0;

        loop {
            let instruction = self.instructions.remove(0);
            match instruction {
                Instruction::Node { ident, scope_size } => {
                    let id = node_id.append(next);
                    next += 1;
                    nodes.push(self.node(ident, id, scope_size)?)
                }
                Instruction::For {
                    binding,
                    data,
                    size,
                } => {
                    let binding = self.consts.lookup_ident(binding).unwrap().to_string(); // TODO: Change this to an error with the binding name: "key {key} does not exist"
                    let data = self.consts.lookup_attrib(data).cloned().unwrap(); // TODO: Same as above
                    let body = self.instructions.drain(..size).collect();
                    let id = node_id.clone();
                    let body = Scope::new(body, &self.consts).exec(id.clone())?;
                    let template = Template::Loop {
                        binding,
                        data,
                        body,
                    };

                    nodes.push(template);
                }
                Instruction::If { cond, size } => {
                    let id = node_id.clone();
                    let cond = self.consts.lookup_attrib(cond).cloned().unwrap(); // TODO: Look at For
                    let body = self.instructions.drain(..size).collect::<Vec<_>>();
                    let body = Scope::new(body, &self.consts).exec(id.clone())?;

                    let mut control_flow = vec![];
                    control_flow.push(ControlFlow {
                        cond: Cond::If(cond),
                        body,
                    });

                    loop {
                        let Some(&Instruction::Else { cond, size }) = self.instructions.get(0)
                        else {
                            break;
                        };
                        let id = node_id.clone();
                        let cond = cond.map(|c| self.consts.lookup_attrib(c).cloned().unwrap()); // TODO: Look at For
                        let body = self.instructions.drain(..size).collect();
                        let body = Scope::new(body, &self.consts).exec(id.clone())?;

                        control_flow.push(ControlFlow {
                            cond: Cond::Else(cond),
                            body,
                        });
                    }

                    let template = Template::ControlFlow(control_flow);
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

    fn node(&mut self, ident: usize, id: NodeId, scope_size: usize) -> Result<Template> {
        let ident = self.consts.lookup_ident(ident).unwrap(); // TODO: Lookup error (see above)

        let mut attributes = Attributes::empty();
        let mut text = None::<TextPath>;
        let mut ip = 0;

        loop {
            match self.instructions.get(ip) {
                Some(Instruction::LoadAttribute { key, value }) => {
                    // TODO: unwrap
                    let key = self.consts.lookup_ident(*key).unwrap();
                    // TODO: unwrap
                    let value = self.consts.lookup_attrib(*value).unwrap();
                    attributes.set(key.to_string(), value.clone());
                }
                Some(Instruction::LoadText(i)) => text = self.consts.lookup_text(*i).cloned(),
                _ => break,
            }
            ip += 1;
        }

        // Remove processed attribute and text instructions
        self.instructions.drain(..ip);

        let scope = self.instructions.drain(..scope_size).collect();
        let children = Scope::new(scope, &self.consts).exec(id.clone())?;

        let node = Template::Node {
            ident: ident.to_string(),
            attributes,
            text,
            children,
        };

        Ok(node)
    }
}
