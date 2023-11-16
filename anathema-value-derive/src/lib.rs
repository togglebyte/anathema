use proc_macro::TokenStream;
use quote::quote;
use syn::{self, Fields};


#[proc_macro_derive(Statey)]
pub fn state_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_state(&ast)
}

fn impl_state(ast: &syn::DeriveInput) -> TokenStream {
    let syn::Data::Struct(strct) = &ast.data else {
        panic!("a state has to be a struct");
    };
    let name = &ast.ident;

    // let mut fields = vec![];

    let Fields::Named(struct_fields) = &strct.fields else {
        panic!("only named fields");
    };

    for field in &struct_fields.named {
        let ident = match &field.ident {
            Some(ident) => ident,
            None => panic!("named fields only"),
        };

        let ty = match field.ty {
        }
        //
    }

    let gen = quote! {
        impl State for #name {
            fn get(&self, key: &anathema::values::Path) -> Option<std::borrow::Cow<'_, str>> {
                None
            }
        }
    };

    gen.into()
}
