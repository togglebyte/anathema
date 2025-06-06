use syn::spanned::Spanned as _;
use syn::{DeriveInput, parse_macro_input};

static DERIVE_NAMESPACE: &str = "anathema";

#[proc_macro_derive(State, attributes(anathema, state_ignore))]
pub fn anathema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        syn::Data::Struct(data) => structs::generate(&input, data),

        syn::Data::Enum(data) => syn::Error::new(
            data.enum_token.span(),
            "anathema's State cannot be derived on enums currently",
        )
        .to_compile_error()
        .into(),

        syn::Data::Union(data) => syn::Error::new(
            data.union_token.span(),
            "anathema's State cannot be derived on unions currently",
        )
        .to_compile_error()
        .into(),
    }
}

mod attributes;
mod errors;
mod structs;
