use crate::{
    attributes::{Constraint, Spanned, parse_attrs},
    errors::{reduce_errors, report_missing_data},
};

use proc_macro2::Span;
use std::collections::HashMap;
use syn::{DataStruct, DeriveInput, Field, Fields, Ident, spanned::Spanned as _};

static FIELD_RENAME: &str = "rename";
static FIELD_IGNORE: &str = "ignore";
static STATE_IGNORE: &str = "state_ignore";

const AVAILABLE_ATTRIBUTES: &[(&str, Constraint)] = &[
    (FIELD_RENAME, Constraint::Exclusive),
    (FIELD_IGNORE, Constraint::Unique),
];

const DEPRECATED_ATTRIBUTES: &[(&str, Constraint)] = &[
    (STATE_IGNORE, Constraint::Unique), //
];

pub fn generate(input: &DeriveInput, data: &DataStruct) -> proc_macro::TokenStream {
    let kind = match collect_fields(
        &data.fields,
        AVAILABLE_ATTRIBUTES, //
        DEPRECATED_ATTRIBUTES,
    ) {
        Ok(kind) => kind,
        Err(err) => return err.into_compile_error().into(),
    };

    match kind {
        GenerateKind::Composite { fields } => generate_composite(&input.ident, fields),
        GenerateKind::List { len } => generate_list(&input.ident, len),
        GenerateKind::Unit => generate_unit(&input.ident),
    }
}

fn generate_unit(name: &Ident) -> proc_macro::TokenStream {
    quote::quote! {
        impl ::anathema::state::State for #name {
            fn type_info(&self) -> ::anathema::state::Type {
                ::anathema::state::Type::Unit
            }
        }

        impl ::anathema::state::TypeId for #name {
            const TYPE: ::anathema::state::Type = ::anathema::state::Type::Unit;
        }
    }
    .into()
}

fn generate_list(name: &Ident, len: usize) -> proc_macro::TokenStream {
    let iter = (0..len).map(|n| {
        let n = syn::Index::from(n);
        quote::quote! {
            #n => Some(self.#n.reference())
        }
    });

    quote::quote! {
        impl ::anathema::state::State for #name {
            fn type_info(&self) -> ::anathema::state::Type {
                ::anathema::state::Type::List
            }
        }

        impl ::anathema::state::TypeId for #name {
            const TYPE: ::anathema::state::Type = ::anathema::state::Type::List;
        }

        impl ::anathema::state::AnyList for #name {
            fn lookup(&self, index: usize) -> Option<::anathema::state::PendingValue> {
                match index {
                    #( #iter, )*
                    _ => None
                }
            }

            fn len(&self) -> usize {
                #len
            }
        }
    }
    .into()
}

fn generate_composite(name: &Ident, fields: Vec<data::Field>) -> proc_macro::TokenStream {
    let field_names = fields
        .iter()
        .map(|data::Field { display_name, .. }| quote::quote! { #display_name });

    let field_idents = fields
        .iter()
        .map(
            |data::Field {
                 name: Spanned { span, value },
                 ..
             }| syn::Ident::new(value, *span),
        )
        .map(|ident| {
            let span = ident.span();
            quote::quote_spanned! {span=> Some(self.#ident.reference())}
        });

    quote::quote! {
        impl ::anathema::state::State for #name {
            fn type_info(&self) -> ::anathema::state::Type {
                ::anathema::state::Type::Composite
            }

            fn as_any_map(&self) -> Option<&dyn ::anathema::state::AnyMap> {
                Some(self)
            }
        }

        impl ::anathema::state::TypeId for #name {
            const TYPE: ::anathema::state::Type = ::anathema::state::Type::Composite;
        }

        impl ::anathema::state::AnyMap for #name {
            fn lookup(&self, key: &str) -> Option<::anathema::state::PendingValue> {
                match key {
                    #(
                        #field_names => {
                            #field_idents
                        },
                    )*
                    _ => None,
                }
            }
        }
    }
    .into()
}

mod data {
    use crate::attributes::Spanned;

    pub struct Field {
        pub name: Spanned<String>,
        pub display_name: String,
    }
}

enum GenerateKind {
    Composite { fields: Vec<data::Field> },
    List { len: usize },
    Unit,
}

fn has_attributes(fields: &Fields, deprecated_attributes: &[(&str, Constraint)]) -> bool {
    let count = |field: &Field| {
        field
            .attrs
            .iter()
            .filter(|c| {
                deprecated_attributes
                    .iter()
                    .any(|(name, _)| c.meta.path().is_ident(name))
                    || c.meta.path().is_ident(crate::DERIVE_NAMESPACE)
            })
            .count()
    };
    fields.iter().map(count).sum::<usize>() > 0
}

#[derive(Debug)]
struct SpannedField {
    field_name: Spanned<String>,
    rename: Option<Spanned<String>>,
}

fn check_duplicate_renames(values: &[SpannedField]) -> Option<Vec<syn::Error>> {
    let mut seen = <HashMap<&str, Vec<Span>>>::new();

    for value in values {
        let (name, span) = value
            .rename
            .as_ref()
            .map(|rename| (&rename.value, rename.span))
            .unwrap_or((&value.field_name.value, value.field_name.span));
        seen.entry(name).or_default().push(span);
    }

    let duplicate_spans = seen
        .into_iter()
        .filter(|(_, list)| list.len() > 1)
        .collect::<Vec<_>>();

    if duplicate_spans.is_empty() {
        return None;
    }

    let mut errors = vec![];
    for (name, mut dupe) in duplicate_spans {
        let message = format!("duplicate name '{name}' used here");
        let mut error = syn::Error::new(dupe.remove(0), message);
        for span in dupe {
            let message = format!("'{name}' is also used here");
            error.combine(syn::Error::new(span, message))
        }
        errors.push(error);
    }

    Some(errors)
}

fn collect_fields(
    fields: &Fields,
    available_attributes: &[(&'static str, Constraint)],
    deprecated_attributes: &[(&'static str, Constraint)],
) -> Result<GenerateKind, syn::Error> {
    if matches!(fields, Fields::Unnamed(..)) {
        if has_attributes(fields, deprecated_attributes) {
            return Err(syn::Error::new(
                fields.span(),
                "anathema attributes on tuple structs aren't currently supported",
            ));
        }
        return Ok(GenerateKind::List { len: fields.len() });
    }

    if fields.is_empty() {
        return Ok(GenerateKind::Unit);
    }

    let mut out = vec![];
    let mut errors = vec![];
    let mut seen = vec![];

    for field in fields {
        let mut kvs = match parse_attrs(
            crate::DERIVE_NAMESPACE, //
            &field.attrs,
            available_attributes,
            deprecated_attributes,
        ) {
            Ok(kvs) => kvs,
            Err(err) => {
                errors.push(err);
                continue;
            }
        };

        let name = match &field.ident {
            Some(name) => name,
            None => {
                errors.push(syn::Error::new(
                    field.span(),
                    "anathema's attributes cannot be applied to unnamed fields",
                ));
                continue;
            }
        };

        // the duplicate check doesn't need to know about ignored fields
        if kvs.remove(FIELD_IGNORE).is_some() || kvs.remove(STATE_IGNORE).is_some() {
            continue;
        }

        let field_name = Spanned {
            value: name.to_string(),
            span: field.span(),
        };

        let rename = match kvs.remove(FIELD_RENAME) {
            Some(parsed) => {
                #[allow(clippy::let_and_return)] // clippy is totally wrong about this
                let value @ Some(..) = parsed.value else {
                    errors.push(report_missing_data(parsed.key));
                    continue;
                };
                value
            }
            None => None,
        };

        out.push(data::Field {
            name: Spanned {
                span: field.span(),
                value: field_name.value.clone(),
            },
            display_name: rename
                .as_ref()
                .map(|c| c.value.clone())
                .unwrap_or_else(|| field_name.value.clone())
                .trim()
                .to_string(),
        });
        seen.push(SpannedField { field_name, rename });
    }

    errors.extend(check_duplicate_renames(&seen).into_iter().flatten());
    // errors.reverse();

    reduce_errors(GenerateKind::Composite { fields: out }, errors)
}
