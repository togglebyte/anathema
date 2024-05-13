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
        # use ::anathema::state::{self, Value, ValueRef, PendingValue, Path, state, Subscriber, CommonVal};
        # use ::std::any::Any;
        impl state::State for #name {
            fn state_get(&self, path: Path<'_>, sub: Subscriber) -> Option<ValueRef> {
                let Path::Key(key) = path else { return None };
                match key {
                    #(
                        #field_names => {
                            Some(self.#field_idents.value_ref(sub))
                        }
                    )*
                    _ => None,
                }
            }

            fn state_lookup(&self, path: Path<'_>) -> Option<PendingValue> {
                let Path::Key(key) = path else { return None };
                match key {
                    #(
                        #field_names => {
                            Some(self.#field_idents.to_pending())
                        }
                    )*
                    _ => None,
                }
            }

            fn to_common(&self) -> Option<CommonVal<'_>> {
                None
            }
        }
    })
}
