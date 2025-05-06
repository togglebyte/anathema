use std::collections::{BTreeMap, HashMap, HashSet};

use proc_macro2::Span;
use syn::{Attribute, LitStr, spanned::Spanned as _};

use crate::errors::{
    reduce_errors, report_empty_value, report_exclusive_failure, report_unique_failure,
    report_unknown_attribute,
};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Constraint {
    Unique,
    Exclusive,
    #[default]
    Loose,
}

#[derive(Debug)]
pub struct ParsedAttr {
    pub key: Span,
    pub value: Option<Spanned<String>>,
}

#[derive(Clone, Debug)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

#[derive(Clone, Debug)]
struct Attr {
    key: Spanned<String>,
    value: Option<Spanned<String>>,
}

fn parse_attr(errors: &mut Vec<syn::Error>, attr: &Attribute) -> Vec<Attr> {
    let attr_list = match attr.meta.require_list() {
        Ok(attr_list) => attr_list,
        Err(err) => {
            if let Ok(path) = attr
                .meta
                .require_path_only()
                .and_then(|p| p.require_ident())
            {
                return vec![Attr {
                    key: Spanned {
                        span: path.span(),
                        value: path.to_string().trim().to_string(),
                    },
                    value: None,
                }];
            }

            errors.push(err);
            return vec![];
        }
    };

    let mut out = vec![];
    let Err(err) = attr_list.parse_nested_meta(|meta| {
        let path = &meta.path;

        let ident = path.require_ident()?;
        let ident_name = ident.to_string();

        enum Kind {
            Value { data: String, span: Span },
            Key { span: Span },
        }

        let kind = meta
            .value()
            .and_then(|c| {
                let value = c.parse::<LitStr>()?;
                Ok(Kind::Value {
                    data: value.value().trim().to_string(),
                    span: value.span(),
                })
            })
            .unwrap_or_else(|_| Kind::Key { span: ident.span() });

        let (data, value_span) = match kind {
            Kind::Value { data, span } => {
                if data.trim().is_empty() {
                    errors.push(report_empty_value(&ident_name, span));
                    return Ok(());
                }
                (Some(data), span)
            }
            Kind::Key { span } => (None, span),
        };

        out.push(Attr {
            key: Spanned {
                span: meta.path.span(),
                value: ident_name,
            },
            value: data.map(|value| Spanned {
                span: value_span,
                value,
            }),
        });

        Ok(())
    }) else {
        return out;
    };

    errors.push(err);
    vec![]
}

pub fn parse_attrs(
    namespace: &str,
    attrs: &[Attribute],
    allowed: &[(&'static str, Constraint)],
    deprecated: &[(&'static str, Constraint)],
) -> Result<HashMap<String, ParsedAttr>, syn::Error> {
    let mapping = allowed.iter().copied().collect::<BTreeMap<_, _>>();

    let mut errors = Vec::new();
    let mut parsed = HashMap::new();

    let mut is_unique = None;
    let mut seen = <HashSet<String>>::new();

    for attr in attrs {
        if !deprecated
            .iter()
            .any(|(name, _)| attr.path().is_ident(name))
            && !attr.path().is_ident(namespace)
        {
            continue;
        }

        for Attr { key, value } in parse_attr(&mut errors, attr) {
            match mapping.get(&*key.value).or_else(|| {
                deprecated
                    .iter()
                    .find_map(|(name, constraint)| (&key.value == name).then_some(constraint))
            }) {
                Some(constraint) => match constraint {
                    Constraint::Unique if is_unique.is_some() => {
                        errors.push(report_unique_failure(&key.value, key.span));
                        continue;
                    }
                    Constraint::Unique => {
                        if !seen.is_empty() {
                            errors.push(report_unique_failure(&key.value, key.span));
                            continue;
                        }

                        is_unique = Some(Attr {
                            key: key.clone(),
                            value: value.clone(),
                        });
                    }

                    Constraint::Exclusive if is_unique.is_none() => {
                        if !seen.insert(key.value.clone()) {
                            errors.push(report_exclusive_failure(&key.value, key.span));
                            continue;
                        }
                    }

                    _ if is_unique.is_some() => {
                        let Attr { key, .. } = is_unique.as_ref().unwrap();
                        errors.push(report_unique_failure(&key.value, key.span));
                        continue;
                    }
                    _ => {}
                },

                None => {
                    errors.push(report_unknown_attribute(
                        mapping.keys(),
                        &key.value,
                        key.span,
                    ));
                    continue;
                }
            }

            let value = ParsedAttr {
                key: key.span,
                value,
            };
            parsed.insert(key.value, value);
        }
    }

    reduce_errors(parsed, errors)
}
