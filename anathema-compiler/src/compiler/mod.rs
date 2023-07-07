use self::optimizer::Expression;
pub(crate) use self::optimizer::Optimizer;
use super::error::Result;

mod optimizer;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
    If {
        cond: usize,
        size: usize,
    },
    Else {
        cond: Option<usize>,
        size: usize,
    },
    For {
        binding: usize,
        data: usize,
        size: usize,
    },
    Node {
        ident: usize,
        scope_size: usize,
    },
    LoadAttribute {
        key: usize,
        value: usize,
    },
    LoadText(usize),
}

enum Branch {
    If(usize),
    Else(Option<usize>),
}

pub(super) struct Compiler {
    expressions: Vec<Expression>,
    ep: usize,
    output: Vec<Instruction>,
}

impl Compiler {
    pub(super) fn new(expressions: impl IntoIterator<Item = Expression>) -> Self {
        let expressions = expressions.into_iter().collect::<Vec<_>>();
        let inst = Self {
            ep: 0,
            output: Vec::with_capacity(expressions.len()),
            expressions,
        };
        inst
    }

    pub(super) fn compile(mut self) -> Result<Vec<Instruction>> {
        loop {
            self.compile_expression()?;
            if self.ep == self.expressions.len() {
                break;
            }
        }

        Ok(self.output)
    }

    fn compile_expression(&mut self) -> Result<()> {
        if let Some(expr) = self.expressions.get(self.ep) {
            self.ep += 1;
            match expr {
                Expression::Node { ident, scope_size } => self.compile_node(*ident, *scope_size),
                Expression::LoadText(index) => self.compile_text(*index),
                Expression::LoadAttribute { key, value } => self.compile_attribute(*key, *value),
                Expression::If { cond, size } => {
                    self.compile_control_flow(Branch::If(*cond), *size)
                }
                Expression::Else { cond, size } => {
                    self.compile_control_flow(Branch::Else(*cond), *size)
                }
                Expression::For {
                    binding,
                    data,
                    size,
                } => self.compile_for(*binding, *data, *size),
            }?;
        }
        Ok(())
    }

    fn compile_node(&mut self, ident: usize, child_scope_size: usize) -> Result<()> {
        self.output.push(Instruction::Node {
            ident,
            scope_size: child_scope_size,
        });
        Ok(())
    }

    fn compile_text(&mut self, index: usize) -> Result<()> {
        self.output.push(Instruction::LoadText(index));
        Ok(())
    }

    fn compile_attribute(&mut self, key: usize, value: usize) -> Result<()> {
        self.output.push(Instruction::LoadAttribute { key, value });
        Ok(())
    }

    fn compile_inner_scope(&mut self, size: usize) -> Result<()> {
        let expressions = self.expressions.drain(self.ep..self.ep + size);
        let mut body = Compiler::new(expressions).compile()?;
        self.output.append(&mut body);
        Ok(())
    }

    fn compile_control_flow(&mut self, branch: Branch, size: usize) -> Result<()> {
        let instruction_index = self.output.len();
        self.compile_inner_scope(size)?;

        let size = self.output[instruction_index..].len();
        if let Some(Expression::Else { .. }) = self.expressions.get(self.ep) {
            self.compile_expression()?;
        }

        let instruction = match branch {
            Branch::If(cond) => Instruction::If { cond, size },
            Branch::Else(cond) => Instruction::Else { cond, size },
        };

        self.output.insert(instruction_index, instruction);

        Ok(())
    }

    fn compile_for(&mut self, binding: usize, data: usize, size: usize) -> Result<()> {
        let instruction_index = self.output.len();

        // Inner scope = body
        self.compile_inner_scope(size)?;

        let instruction = Instruction::For {
            binding,
            data,
            size,
        };
        self.output.insert(instruction_index, instruction);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn parse(src: &str) -> Vec<Instruction> {
        crate::compile(src).unwrap().0
    }

    #[test]
    fn nested_children() {
        let src = r#"
        vstack
            border
            border
                text
        "#;
        let mut instructions = parse(src);
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 3
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 1
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 2,
                scope_size: 0
            }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn double_ifs() {
        let src = "
        if {{ x }}
            a
        if {{ y }}
            b
        c
        ";

        let mut instructions = parse(src);
        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 1 });
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert_eq!(instructions.remove(0), Instruction::If { cond: 1, size: 1 });
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 2,
                scope_size: 0
            }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn compile_empty() {
        let expressions = vec![];
        let compiler = Compiler::new(expressions);
        let instructions = compiler.compile().unwrap();
        assert!(instructions.is_empty());
    }

    #[test]
    fn compile_if() {
        let expressions = vec![
            Expression::If { cond: 0, size: 1 },
            Expression::Node {
                ident: 0,
                scope_size: 0,
            },
            Expression::Node {
                ident: 1,
                scope_size: 0,
            },
        ];

        let compiler = Compiler::new(expressions);
        let mut instructions = compiler.compile().unwrap();

        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 1 });
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
    }

    #[test]
    fn for_loop() {
        let src = "
        for x in {{ y }}
            a
        ";

        let mut instructions = parse(src);
        assert_eq!(
            instructions.remove(0),
            Instruction::For {
                binding: 0,
                data: 0,
                size: 1
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn nested_for_loop() {
        let src = "
        for x in {{ y }}
            for z in {{ x }}
                a
        ";

        let mut instructions = parse(src);
        assert_eq!(
            instructions.remove(0),
            Instruction::For {
                binding: 0,
                data: 0,
                size: 2
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::For {
                binding: 1,
                data: 1,
                size: 1
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 2,
                scope_size: 0
            }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn if_else_if_else_if() {
        let src = "
        if {{ x }}
            a
        else if {{ x }}
            b
        else
            c
            d
        if {{ x }}
            a
        ";

        let mut instructions = parse(src);
        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 1 });
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Else {
                cond: Some(0),
                size: 1,
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Else {
                cond: None,
                size: 2,
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 2,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 3,
                scope_size: 0
            }
        );
        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 1 });
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn if_if_if_nested() {
        let src = "
        if {{ x }}
            if {{ x }}
                if {{ x }}
                    a
        b
        ";

        let mut instructions = parse(src);
        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 3 });
        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 2 });
        assert_eq!(instructions.remove(0), Instruction::If { cond: 0, size: 1 });
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn load_text() {
        let src = "a 'It is tea time'";
        let mut instructions = parse(src);
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert_eq!(instructions.remove(0), Instruction::LoadText(0));
        assert!(instructions.is_empty());
    }

    #[test]
    fn load_attributes() {
        let src = "a [a: a, a: a]";
        let mut instructions = parse(src);
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::LoadAttribute { key: 0, value: 0 }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::LoadAttribute { key: 0, value: 0 }
        );
        assert!(instructions.is_empty());
    }

    #[test]
    fn load_text_nested_nodes() {
        let src = r#"
        text "hi {{ val }}"
            span [a: b] "bye {{ val }}"
        "#;
        let mut instructions = parse(src);
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 0,
                scope_size: 3
            }
        );
        assert_eq!(instructions.remove(0), Instruction::LoadText(0));
        assert_eq!(
            instructions.remove(0),
            Instruction::Node {
                ident: 1,
                scope_size: 0
            }
        );
        assert_eq!(
            instructions.remove(0),
            Instruction::LoadAttribute { key: 2, value: 0 }
        );
        assert_eq!(instructions.remove(0), Instruction::LoadText(1));
        assert!(instructions.is_empty());
    }
}
