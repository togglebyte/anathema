use anathema_store::smallmap::SmallMap;
use anathema_store::storage::strings::StringId;

use super::const_eval::const_eval;
use super::{Context, Statement, Statements};
use crate::blueprints::{Blueprint, Component, ControlFlow, Else, For, Single};
use crate::error::{Error, Result};
use crate::expressions::{Equality, Expression};
use crate::{ComponentBlueprintId, Primitive};

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
                Statement::For { binding, data } => {
                    if let Some(expr) = self.eval_for(binding, data, ctx)? {
                        output.push(expr);
                    }
                }
                Statement::If(cond) => output.push(self.eval_if(cond, ctx)?),
                Statement::Switch(cond) => output.push(self.eval_switch(cond, ctx)?),
                Statement::Declaration { binding, value } => {
                    let Some(value) = const_eval(value, ctx) else { continue };
                    let binding = ctx.strings.get_unchecked(binding);
                    if binding == "state" {
                        return Err(Error::InvalidStatement(format!("{binding} is a reserved identifier")));
                    }
                    ctx.globals.declare(binding, value);
                }
                Statement::ComponentSlot(slot_id) => {
                    if let Some(bp) = ctx.slots.get(&slot_id).cloned() {
                        output.push(Blueprint::Slot(bp));
                    }
                }
                Statement::ScopeStart
                | Statement::ScopeEnd
                | Statement::LoadAttribute { .. }
                | Statement::AssociatedFunction { .. }
                | Statement::Else(_)
                | Statement::LoadValue(_)
                | Statement::Case(_)
                | Statement::Default => {
                    return Err(Error::InvalidStatement(format!("{statement:?}")));
                }
                Statement::Eof => break,
            }
        }
        Ok(output)
    }

    fn eval_node(&mut self, ident: StringId, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let ident = ctx.strings.get_unchecked(ident);
        let attributes = self.eval_attributes(ctx)?;
        let value = self.statements.take_value().and_then(|v| const_eval(v, ctx));
        let children = self.consume_scope(ctx)?;

        let node = Blueprint::Single(Single {
            ident,
            children,
            attributes,
            value,
        });
        Ok(node)
    }

    fn eval_for(&mut self, binding: StringId, data: Expression, ctx: &mut Context<'_>) -> Result<Option<Blueprint>> {
        let Some(data) = const_eval(data, ctx) else { return Ok(None) };
        let binding = ctx.strings.get_unchecked(binding);
        // add binding to globals so nothing can resolve past the binding outside of the loop
        ctx.globals.declare_local(binding.clone());
        let body = self.consume_scope(ctx)?;
        let node = Blueprint::For(For { binding, data, body });
        Ok(Some(node))
    }

    fn consume_scope(&mut self, ctx: &mut Context<'_>) -> Result<Vec<Blueprint>> {
        let scope = Scope::new(self.statements.take_scope());
        scope.eval(ctx)
    }

    fn eval_attributes(&mut self, ctx: &mut Context<'_>) -> Result<SmallMap<String, Expression>> {
        let mut hm = SmallMap::empty();

        for (key, value) in self.statements.take_attributes() {
            let Some(value) = const_eval(value, ctx) else { continue };
            let key = ctx.strings.get_unchecked(key);
            hm.set(key, value);
        }

        Ok(hm)
    }

    fn eval_if(&mut self, cond: Expression, ctx: &mut Context<'_>) -> Result<Blueprint> {
        // Const eval fail = static false
        let cond = const_eval(cond, ctx).unwrap_or(Expression::Primitive(Primitive::Bool(false)));
        let body = self.consume_scope(ctx)?;
        if body.is_empty() {
            return Err(Error::EmptyBody);
        }

        let mut elses = vec![Else { cond: Some(cond), body }];

        while let Some(cond) = self.statements.next_else() {
            let body = self.consume_scope(ctx)?;
            let cond = cond.and_then(|v| const_eval(v, ctx));

            if body.is_empty() {
                return Err(Error::EmptyBody);
            }

            elses.push(Else { cond, body });
        }
        Ok(Blueprint::ControlFlow(ControlFlow { elses }))
    }

    fn eval_switch(&mut self, cond: Expression, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let switch = const_eval(cond, ctx);
        let mut elses = vec![];

        let mut body = self.statements.take_scope();

        while let Some(case) = body.next_case() {
            let cond = match switch {
                Some(ref switch) => Expression::Equality(switch.clone().into(), case.into(), Equality::Eq),
                None => Expression::Primitive(Primitive::Bool(false)),
            };

            let body = match body.is_next_scope() {
                true => {
                    let scope = Scope::new(body.take_scope());
                    scope.eval(ctx)?
                }
                false => {
                    let scope = Scope::new(body.take_until_case_or_default());
                    scope.eval(ctx)?
                }
            };

            elses.push(Else { cond: Some(cond), body });
        }

        if body.next_default() {
            let body = match body.is_next_scope() {
                true => {
                    let scope = Scope::new(body.take_scope());
                    scope.eval(ctx)?
                }
                false => {
                    let scope = Scope::new(body.take_until_case_or_default());
                    scope.eval(ctx)?
                }
            };

            elses.push(Else { cond: None, body });
        }

        Ok(Blueprint::ControlFlow(ControlFlow { elses }))
    }

    fn eval_component(&mut self, component_id: ComponentBlueprintId, ctx: &mut Context<'_>) -> Result<Blueprint> {
        let parent = ctx.component_parent();

        // Associated functions
        let assoc_functions = self.statements.take_assoc_functions();

        // Attributes
        let attributes = self.eval_attributes(ctx)?;

        // Slots
        let mut slots = SmallMap::empty();
        let mut scope = self.statements.take_scope();

        // If the next statement is NOT a slot id then assume $children
        // and still try to take the scope
        if !scope.is_next_slot() {
            let slot_id = ctx.strings.children();
            let scope = Scope::new(scope);
            let body = scope.eval(ctx)?;
            slots.set(slot_id, body);
        } else {
            // for each slot take the scope and associate it with the slot id
            while let Some(slot_id) = scope.next_slot() {
                let scope = Scope::new(scope.take_scope());
                let body = scope.eval(ctx)?;
                slots.set(slot_id, body);
            }
        }

        let body = ctx.load_component(component_id, slots)?;
        let name_id = ctx.components.name(component_id);
        let name = ctx.strings.get_unchecked(name_id);

        let component = Component {
            name,
            name_id,
            id: component_id,
            body,
            attributes,
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
    use crate::{ToSourceKind, single};

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
        assert_eq!(blueprint, single!(children @ "a", vec![single!("b")]));
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
    fn eval_declaration_should_not_uses_reserved_identifiers() {
        let src = "let state = 1;";

        let mut doc = Document::new(src);
        let response = doc.compile();
        assert_eq!(
            response.err().unwrap().to_string(),
            "invalid statement: state is a reserved keyword"
        );
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
    fn if_else() {
        let src = "
            if 1 == 2
                text
            else
                border
        ";

        let mut doc = Document::new(src);
        let (blueprint, _) = doc.compile().unwrap();
        let Blueprint::ControlFlow(controlflow) = blueprint else { panic!() };
        assert!(matches!(controlflow.elses[0], Else { .. }));
        assert!(!controlflow.elses.is_empty());
    }

    #[test]
    fn eval_switch_case_single() {
        let src = "
            switch 1 == 2
                case true: text 'yay'
                case false: text 'yay'
        ";

        let mut doc = Document::new(src);
        let (blueprint, _) = doc.compile().unwrap();
        let Blueprint::ControlFlow(controlflow) = blueprint else { panic!() };
        assert!(matches!(controlflow.elses[0], Else { .. }));
        assert!(!controlflow.elses.is_empty());
    }

    #[test]
    fn eval_switch_case_multi_line() {
        let src = "
            switch 1 == 2
                case true: 
                    text 'yay'
                case false: 
                    text 'yay'
                    text 'yay'
        ";

        let mut doc = Document::new(src);
        let (blueprint, _) = doc.compile().unwrap();
        let Blueprint::ControlFlow(controlflow) = blueprint else { panic!() };
        assert!(matches!(controlflow.elses[0], Else { .. }));
        assert!(!controlflow.elses.is_empty());
    }

    #[test]
    fn eval_component() {
        let src = "@comp [a: 1]";
        let comp_src = "node a + 2";

        let mut doc = Document::new(src);
        doc.add_component("comp", comp_src.to_template()).unwrap();
        let (blueprint, _) = doc.compile().unwrap();
        assert!(matches!(blueprint, Blueprint::Component(Component { .. })));
    }

    #[test]
    fn eval_component_slots() {
        // TODO: this test is incomplete.
        // It should verify that node '1' and node '2' are in the components
        // blueprint
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
                @comp (a->b) [ a: 1 ]
                @comp (a->b) [ a: 2 ]
        ";

        let mut doc = Document::new(src);
        doc.add_component("comp", "node a".to_template()).unwrap();
        let _ = doc.compile().unwrap();
    }

    #[test]
    fn component_with_assoc_attrs_state() {
        let src = "
            @comp (a->b) [a: 1]
        ";

        let mut doc = Document::new(src);
        doc.add_component("comp", "node a".to_template()).unwrap();
        let _ = doc.compile().unwrap();
    }
}
