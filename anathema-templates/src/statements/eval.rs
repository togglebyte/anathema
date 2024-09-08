use std::rc::Rc;

use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;

use super::const_eval::const_eval;
use super::{Context, Statement, Statements};
use crate::blueprints::{Blueprint, Component, ControlFlow, Else, For, If, Single};
use crate::error::{Error, Result};
use crate::expressions::Expression;
use crate::WidgetComponentId;

pub(crate) struct Scope {
    statements: Statements,
}

impl Scope {
    pub(crate) fn new(statements: Statements) -> Self {
        Self { statements }
    }

    pub(crate) fn eval(mut self, ctx: &mut Context<'_>) -> Result<Vec<Blueprint>> {
        let mut output = vec![];

        while let Some(statement) = self.statements.next() {
            match statement {
                Statement::Node(ident) => output.push(self.eval_node(ident, ctx)?),
                Statement::Component(component_id) => output.push(self.eval_component(component_id, ctx)?),
                Statement::For { binding, data } => output.push(self.eval_for(binding, data, ctx)?),
                Statement::If(cond) => output.push(self.eval_if(cond, ctx)?),
                Statement::Declaration { binding, value } => {
                    let value = const_eval(value, ctx);
                    let binding = ctx.strings.get_unchecked(binding);
                    ctx.globals.declare(binding, value);
                }
                Statement::ComponentSlot(slot_id) => {
                    if let Some(bp) = ctx.slots.get(&slot_id).cloned() {
                        output.extend(bp);
                    }
                }

                // These statements can't be evaluated on their own,
                // as they are part of other statements
                Statement::ScopeStart
                | Statement::ScopeEnd
                | Statement::LoadAttribute { .. }
                | Statement::AssociatedFunction { .. }
                | Statement::Else(_)
                | Statement::LoadValue(_) => {
                    unreachable!("\"{statement:?}\" found: this is a bug in Anathema. Please open an issue")
                }
                Statement::Eof => break,
            }
        }
        Ok(output)
    }

    fn eval_node(&mut self, ident: StringId, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let ident = ctx.strings.get_unchecked(ident);
        let attributes = self.eval_attributes(ctx)?;
        let value = self.statements.take_value().map(|v| const_eval(v, ctx));
        let children = self.consume_scope(ctx)?;

        let node = Blueprint::Single(Single {
            ident: ident.into(),
            children,
            attributes,
            value,
        });
        Ok(node)
    }

    fn eval_for(&mut self, binding: StringId, data: Expression, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let data = const_eval(data, ctx);
        let binding = ctx.strings.get_unchecked(binding);
        let body = self.consume_scope(ctx)?;
        let node = Blueprint::For(For {
            binding: binding.into(),
            data,
            body,
        });
        Ok(node)
    }

    fn consume_scope(&mut self, ctx: &mut Context<'_>) -> Result<Vec<Blueprint>> {
        let scope = Scope::new(self.statements.take_scope());
        scope.eval(ctx)
    }

    fn eval_attributes(&mut self, ctx: &mut Context<'_>) -> Result<SmallMap<Rc<str>, Expression>> {
        let mut hm = SmallMap::empty();

        for (key, value) in self.statements.take_attributes() {
            let value = const_eval(value, ctx);
            let key = ctx.strings.get_unchecked(key);
            hm.set(key.into(), value);
        }

        Ok(hm)
    }

    fn eval_if(&mut self, cond: Expression, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let cond = const_eval(cond, ctx);
        let body = self.consume_scope(ctx)?;
        if body.is_empty() {
            return Err(Error::EmptyBody);
        }

        let if_node = If { cond, body };
        let mut elses = vec![];
        while let Some(cond) = self.statements.next_else() {
            let cond = cond.map(|v| const_eval(v, ctx));
            let body = self.consume_scope(ctx)?;

            if body.is_empty() {
                return Err(Error::EmptyBody);
            }

            elses.push(Else { cond, body });
        }
        Ok(Blueprint::ControlFlow(ControlFlow { if_node, elses }))
    }

    fn eval_component(&mut self, component_id: WidgetComponentId, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let parent = ctx.component_parent();

        // Associated functions
        let assoc_functions = self.statements.take_assoc_functions();

        // Attributes
        let attributes = self.eval_attributes(ctx)?;

        // State
        let state = self.statements.take_value().map(|v| const_eval(v, ctx));
        let state = match state {
            Some(Expression::Map(map)) => Some(map),
            Some(_) => todo!("Invalid state: state has to be a map or nothing"),
            None => None,
        };

        // Slots
        let mut slots = SmallMap::empty();
        let mut scope = self.statements.take_scope();

        // for each slot take the scope and associate it with the slot id
        while let Some(slot_id) = scope.next_slot() {
            let scope = Scope::new(scope.take_scope());
            let body = scope.eval(ctx)?;
            slots.set(slot_id, body);
        }

        let body = ctx.load_component(component_id, slots)?;

        let component = Component {
            id: component_id,
            body,
            attributes,
            state,
            assoc_functions,
            parent,
        };

        Ok(Blueprint::Component(component))
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::document::Document;
    use crate::{single, ToSourceKind};

    #[test]
    fn eval_node() {
        let mut doc = Document::new("node");
        let (bp, _) = doc.compile().unwrap();
        assert_eq!(bp, single!("node"));
    }

    #[test]
    fn eval_node_with_children() {
        let src = "
        a
            b
        ";
        let mut doc = Document::new(src);
        let (blueprint, _) = doc.compile().unwrap();
        assert_eq!(blueprint, single!("a", vec![single!("b")]));
    }

    #[test]
    fn eval_nested_nodes() {
        let src = "
            node a
                node a
        ";

        let mut doc = Document::new(src);
        let (blueprint, _) = doc.compile().unwrap();
        assert!(matches!(blueprint, Blueprint::Single(Single { value: Some(_), .. })));
    }

    #[test]
    fn eval_for() {
        let src = "
            for a in a
                node
        ";
        let mut doc = Document::new(src);
        let (blueprint, _) = doc.compile().unwrap();
        assert!(matches!(blueprint, Blueprint::For(For { .. })));
    }

    #[test]
    fn eval_component() {
        let src = "@comp {a: 1}";
        let comp_src = "node a + 2";

        let mut doc = Document::new(src);
        doc.add_component("comp", comp_src.to_template()).unwrap();
        let (blueprint, _) = doc.compile().unwrap();
        assert!(matches!(blueprint, Blueprint::Component(Component { .. })));
    }

    #[test]
    fn eval_component_slots() {
        let src = "
            @comp
                $s1
                    node '1'
                $s2
                    node '2'
        ";

        let comp_src = "
            node
                $s1
                $s2
        ";

        let mut doc = Document::new(src);
        doc.add_component("comp", comp_src.to_template()).unwrap();
        let (blueprint, _) = doc.compile().unwrap();
        assert!(matches!(blueprint, Blueprint::Component(Component { .. })));
    }

    #[test]
    fn eval_two_identical_components() {
        let src = "
            vstack
                @comp (a->b) { a: 1 }
                @comp (a->b) { a: 2 }
        ";

        let mut doc = Document::new(src);
        doc.add_component("comp", "node a".to_template()).unwrap();
        let _ = doc.compile().unwrap();
    }

    #[test]
    fn component_with_assoc_attrs_state() {
        let src = "
            @comp (a->b) [a: 1] { a: 1 }
        ";

        let mut doc = Document::new(src);
        doc.add_component("comp", "node a".to_template()).unwrap();
        let _ = doc.compile().unwrap();
    }
}
