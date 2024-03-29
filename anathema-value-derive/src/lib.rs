use manyhow::{ensure, manyhow, Result};
use quote_use::quote_use as quote;
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
        # use ::anathema::values::{self, ValueRef, Path, state};
        impl state::State for #name {
            fn state_get(&self, key: &values::Path, node_id: &values::NodeId) -> values::ValueRef<'_> {
                match key {
                    Path::Key(s) => match s.as_str() {
                        #(
                            #field_names => {
                                self.#field_idents.get_value(node_id)
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
                                self.#field_idents.state_get(rhs, node_id)
                            }
                        )*
                            _ => ValueRef::Empty,
                        }
                    }
                    _ => ValueRef::Empty,
                }
            }
        }

        impl<'a> Into<ValueRef<'a>> for &'a #name {
            fn into(self) -> ::anathema::values::ValueRef<'a> {
                ::anathema::values::ValueRef::Map(self)
            }
        }
    })
}
