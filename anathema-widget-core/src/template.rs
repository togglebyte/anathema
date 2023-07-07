use crate::gen::expressions::Expression;
use crate::gen::store::Store;
use crate::path::TextPath;
use crate::{Attributes, Value};

#[derive(Debug)]
pub enum Cond {
    If(Value),
    Else(Option<Value>),
}

#[derive(Debug)]
pub struct ControlFlow {
    pub cond: Cond,
    pub body: Vec<Template>,
}

// -----------------------------------------------------------------------------
//   - Node -
// -----------------------------------------------------------------------------
/// Describes how to create a widget.
/// The `Template` is used together with [`Values`] and [`Attributes`] to
/// generate widgets.
#[derive(Debug)]
pub enum Template {
    Node {
        ident: String,
        attributes: Attributes,
        text: Option<TextPath>,
        children: Vec<Template>,
    },
    Loop {
        binding: String,
        data: Value,
        body: Vec<Template>,
    },
    ControlFlow(Vec<ControlFlow>),
}

impl Template {
    pub fn to_expression<'tpl: 'parent, 'parent>(
        &'tpl self,
        values: &Store<'parent>,
    ) -> Expression<'tpl, 'parent> {
        match &self {
            Template::Node { .. } => Expression::Node(&self),
            Template::Loop {
                binding,
                data,
                body,
            } => {
                let data = match data {
                    Value::List(slice) => slice.as_slice(),
                    Value::DataBinding(path) => path
                        .lookup_value(values)
                        .and_then(|v| v.to_slice())
                        .unwrap_or(&[]),
                    _ => &[],
                };
                Expression::for_loop(body, binding, data)
            }
            Template::ControlFlow(control_flow) => {
                for branch in control_flow {
                    match &branch.cond {
                        Cond::If(Value::Bool(true))
                        | Cond::Else(None)
                        | Cond::Else(Some(Value::Bool(true))) => {
                            return Expression::Block(&branch.body)
                        }
                        Cond::If(Value::DataBinding(path))
                        | Cond::Else(Some(Value::DataBinding(path))) => {
                            let is_true = path
                                .lookup_value(values)
                                .and_then(|v| v.to_bool())
                                .unwrap_or(false);

                            if is_true {
                                return Expression::Block(&branch.body);
                            }
                        }
                        _ => continue,
                    }
                }

                Expression::Block(&[])
            }
        }
    }
}

pub fn template_text(text: impl Into<TextPath>) -> Template {
    Template::Node {
        ident: "text".into(),
        attributes: Attributes::empty(),
        text: Some(text.into()),
        children: vec![],
    }
}

pub fn template_span(text: impl Into<TextPath>) -> Template {
    Template::Node {
        ident: "span".into(),
        attributes: Attributes::empty(),
        text: Some(text.into()),
        children: vec![],
    }
}

pub fn template(
    ident: impl Into<String>,
    attributes: impl Into<Attributes>,
    children: impl Into<Vec<Template>>,
) -> Template {
    Template::Node {
        ident: ident.into(),
        attributes: attributes.into(),
        text: None,
        children: children.into(),
    }
}

// pub fn template_if(cond: Value, body: Vec<Template>) -> Template {
//     Template {
//         kind: Kind::If(cond, None),
//         children: body,
//     }
// }

// pub fn template_else(cond: Option<Value>, body: Vec<Template>) -> Template {
//     let else_template = Template {
//         kind: Kind::Else,
//         children: body,
//     };
//     Template {
//         kind: Kind::If(Value::Bool(false), Some(Box::new(else_template))),
//         children: vec![],
//     }
// }

pub fn template_for(
    binding: impl Into<String>,
    data: impl Into<Value>,
    body: impl Into<Vec<Template>>,
) -> Template {
    Template::Loop {
        binding: binding.into(),
        body: body.into(),
        data: data.into(),
    }
}
