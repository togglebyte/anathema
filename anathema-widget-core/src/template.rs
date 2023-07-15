use std::borrow::Cow;
use std::sync::Arc;

use crate::gen::expressions::Expression;
use crate::gen::store::Values;
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
    pub body: Arc<[Template]>,
}

// -----------------------------------------------------------------------------
//   - Node -
// -----------------------------------------------------------------------------
/// Describes how to create a widget.
/// The `Template` is used together with [`Values`] and [`Attributes`] to
/// generate widgets.
#[derive(Debug)]
pub enum Template {
    View(Value),
    Node {
        ident: String,
        attributes: Attributes,
        text: Option<TextPath>,
        children: Arc<[Template]>,
    },
    Loop {
        binding: String,
        data: Value,
        body: Arc<[Template]>,
    },
    ControlFlow(Vec<ControlFlow>),
}

impl Template {
    pub fn to_expression<'parent>(&'parent self, values: &Values<'parent>) -> Expression<'_> {
        match &self {
            Template::View(id) => {
                let id = match id {
                    Value::String(s) => Cow::Borrowed(s.as_str()),
                    Value::DataBinding(path) => {
                        let val = path.lookup_value(values).unwrap_or(&Value::Empty);
                        match val {
                            Value::String(s) => Cow::Borrowed(s.as_str()),
                            _ => Cow::Owned(val.to_string()),
                        }
                    }
                    _ => Cow::Owned(id.to_string()),
                };
                Expression::View(id)
            }
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
        children: Arc::new([]),
    }
}

pub fn template_span(text: impl Into<TextPath>) -> Template {
    Template::Node {
        ident: "span".into(),
        attributes: Attributes::empty(),
        text: Some(text.into()),
        children: Arc::new([]),
    }
}

pub fn template(
    ident: impl Into<String>,
    attributes: impl Into<Attributes>,
    children: impl Into<Vec<Template>>,
) -> Template {
    let children = children.into();
    Template::Node {
        ident: ident.into(),
        attributes: attributes.into(),
        text: None,
        children: children.into(),
    }
}

pub fn template_for(
    binding: impl Into<String>,
    data: impl Into<Value>,
    body: impl Into<Vec<Template>>,
) -> Template {
    let body = body.into();
    Template::Loop {
        binding: binding.into(),
        body: body.into(),
        data: data.into(),
    }
}
