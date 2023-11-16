use manyhow::{ensure, manyhow, Result};
use quote::quote;
use syn::{self, Fields};

#[manyhow]
#[proc_macro_derive(State)]
pub fn state_derive(strct: syn::ItemStruct) -> Result {
    let name = &strct.ident;

    ensure!(
        let Fields::Named(struct_fields) = &strct.fields,
        strct.fields,
        "only named fields"
    );

    let (field_idents, field_names): (Vec<_>, Vec<_>) = struct_fields
        .named
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .map(|f| (f, f.to_string()))
        .unzip();

    Ok(quote! {
        impl ::anathema::values::state::State for #name {
            fn get(&self, key: &::anathema::values::Path, node_id: ::core::option::Option<&::anathema::values::NodeId>) -> ::anathema::values::ValueRef<'_> {
                use ::anathema::values::{ValueRef, Path};
                use ::anathema::values::state::BlanketGet;
                match key {
                    Path::Key(s) => match s.as_str() {
                        #(
                            #field_names => {
                                (&self.#field_idents).__anathema_get_value(node_id)
                            }
                        )*
                        _ => ValueRef::Empty,
                    }
                    Path::Composite(lhs, rhs) => {
                        let Path::Key(ref key) = &**lhs else {
                            return ValueRef::Empty;
                        };
                        match key.as_str() {
                        #(
                            #field_names => {
                                (&self.#field_idents).__anathema_get(rhs, node_id)
                            }
                        )*
                            _ => ValueRef::Empty,
                        }
                    }
                    _ => ValueRef::Empty,
                }
            }
        }
    })
}
