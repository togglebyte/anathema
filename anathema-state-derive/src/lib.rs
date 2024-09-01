use manyhow::{bail, manyhow, Result};
use quote_use::quote_use as quote;
use syn::spanned::Spanned;
use syn::{self, Data, DataEnum, DataStruct, DeriveInput, Fields, FieldsNamed, FieldsUnnamed, Ident, Index};

static STATE_IGNORE: &str = "state_ignore";

#[manyhow]
#[proc_macro_derive(State, attributes(state_ignore))]
pub fn state_derive(input: DeriveInput) -> Result {
    let name = &input.ident;

    match input.data {
        Data::Enum(r#enum) => derive_enum(name, r#enum),
        Data::Struct(strct) => derive_struct(name, strct),
        Data::Union(..) => bail!(input, "only structs and enums are supported"),
    }
}

// Takes the state_get and state_lookup for the structs DataType and derives the State for the struct
fn derive_struct(name: &Ident, strct: DataStruct) -> Result {
    let (state_get, state_lookup) = match &strct.fields {
        Fields::Unit => derive_unit_state(),
        Fields::Named(fields) => derive_named_state(fields),
        Fields::Unnamed(fields) => derive_unnamed_state(fields),
    };
    let state_get = state_get?;
    let state_lookup = state_lookup?;

    let struct_fields = match &strct.fields {
        Fields::Unit => quote!(),
        Fields::Named(fields) => {
            let field_names = fields.named.iter().filter_map(|f| f.ident.as_ref()).collect::<Vec<_>>();
            quote!(#(let #field_names = &self.#field_names;)*)
        }
        Fields::Unnamed(fields) => {
            let field_names = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, f)| Ident::new(&format!("_{index}"), f.span()));
            let field_ids = fields.unnamed.iter().enumerate().map(|(index, _)| Index::from(index));

            quote!(#(let #field_names = &self.#field_ids;)*)
        }
    };

    Ok(quote! {
        # use ::anathema::state::{ValueRef, PendingValue, Path, state, Subscriber, CommonVal};
        impl state::State for #name {
            fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
                #struct_fields
                #state_get
            }

            fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
                #struct_fields
                #state_lookup
            }

            fn to_common(&self) -> Option<CommonVal<'_>> {
                None
            }
        }
    })
}

// Derives State for ()
fn derive_unit_state() -> (Result, Result) {
    (Ok(quote!(None)), Ok(quote!(None)))
}

// Derives State for `struct A { field_a: Value<_> }`
fn derive_named_state(fields: &FieldsNamed) -> (Result, Result) {
    let (field_idents, field_names): (Vec<_>, Vec<_>) = fields
        .named
        .iter()
        .filter(|f| {
            // Ignore all `STATE_IGNORE` attributes
            !f.attrs.iter().any(|attr| attr.path().is_ident(STATE_IGNORE))
        })
        .filter_map(|f| f.ident.as_ref())
        .map(|f| (f, f.to_string()))
        .unzip();

    (
        Ok(quote! {
            # use ::anathema::state::Path;
            let Path::Key(key) = path else { return None };
            match key {
                #(
                    #field_names => {
                        Some(#field_idents.value_ref(sub))
                    }
                )*
                _ => None,
            }
        }),
        Ok(quote! {
            # use ::anathema::state::Path;
            let Path::Key(key) = path else { return None };
            match key {
                #(
                    #field_names => {
                        Some(#field_idents.to_pending())
                    }
                )*
                _ => None,
            }
        }),
    )
}

// Derive State for `struct A(Value<_>);`
fn derive_unnamed_state(fields: &FieldsUnnamed) -> (Result, Result) {
    let (field_indices, field_names): (Vec<usize>, Vec<Ident>) = fields
        .unnamed
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            // Ignore all `STATE_IGNORE` attributes
            !f.attrs.iter().any(|attr| attr.path().is_ident(STATE_IGNORE))
        })
        .map(|(index, f)| (index, Ident::new(&format!("_{index}"), f.span())))
        .unzip();

    (
        Ok(quote! {
            # use ::anathema::state::Path;
            let Path::Index(index) = path else { return None };
            match index {
                #(#field_indices => Some(#field_names.value_ref(sub)),)*
                _ => None,
            }
        }),
        Ok(quote! {
            # use ::anathema::state::Path;
            let Path::Index(index) = path else { return None };
            match index {
                #(#field_indices => Some(#field_names.to_pending()),)*
                _ => None,
            }
        }),
    )
}

// Derives State for enums
fn derive_enum(name: &Ident, enumm: DataEnum) -> Result {
    let mut field_state_get = Vec::with_capacity(enumm.variants.len());
    let mut field_state_lookup = Vec::with_capacity(enumm.variants.len());
    let state_to_common = derive_enum_to_common(&enumm)?;

    let fields = enumm.variants.iter().map(|variant| {
        let ident = &variant.ident;
        match &variant.fields {
            Fields::Unit => Ok::<_, manyhow::Error>((quote!(), quote!())),
            Fields::Unnamed(fields) => {
                let (state_get, state_lookup) = derive_unnamed_state(fields);
                let state_get = state_get?;
                let state_lookup = state_lookup?;
                let unnamed_fields = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, field)| Ident::new(&format!("_{index}"), field.span()))
                    .collect::<Vec<_>>();

                Ok((
                    quote!(Self::#ident(#(#unnamed_fields),*) => {#state_get}),
                    quote!(Self::#ident(#(#unnamed_fields),*) => {#state_lookup}),
                ))
            }
            Fields::Named(fields) => {
                let (state_get, state_lookup) = derive_named_state(fields);
                let state_get = state_get?;
                let state_lookup = state_lookup?;
                let named_fields = fields
                    .named
                    .iter()
                    .filter_map(|field| field.ident.as_ref())
                    .collect::<Vec<_>>();

                Ok((
                    quote!(Self::#ident { #(#named_fields),* } => {#state_get}),
                    quote!(Self::#ident { #(#named_fields),* } => {#state_lookup}),
                ))
            }
        }
    });

    for entry in fields {
        let entry = entry?;
        field_state_get.push(entry.0);
        field_state_lookup.push(entry.1);
    }

    Ok(quote! {
        # use ::anathema::state::{ValueRef, PendingValue, Path, state, Subscriber, CommonVal};
        impl state::State for #name {
            fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
                match self {
                    #(#field_state_get)*
                    _ => None,
                }
            }

            fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
                match self {
                    #(#field_state_lookup)*
                    _ => None,
                }
            }

            fn to_common(&self) -> Option<CommonVal<'_>> {
                #state_to_common
            }
        }
    })
}

fn derive_enum_to_common(enumm: &DataEnum) -> Result {
    let variants = enumm.variants.iter().map(|variant| {
        let ident_name = variant.ident.to_string();
        let ident = &variant.ident;
        match variant.fields {
            Fields::Unit => quote!(Self::#ident => #ident_name,),
            Fields::Named(..) => quote!(Self::#ident{..} => #ident_name,),
            Fields::Unnamed(..) => quote!(Self::#ident(..) => #ident_name,),
        }
    });

    Ok(quote! {
        # use ::anathema::state::CommonVal;
        Some(CommonVal::Str(match self {
            #(#variants)*
        }))
    })
}
