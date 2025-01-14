use manyhow::{ensure, manyhow, Result};
use quote_use::quote_use as quote;
use syn::{self, Data, DeriveInput, Fields};

static STATE_IGNORE: &str = "state_ignore";

#[manyhow]
#[proc_macro_derive(State, attributes(state_ignore))]
pub fn state_derive(input: DeriveInput) -> Result {
    let name = &input.ident;

    ensure!(let Data::Struct(strct) = &input.data, input, "only structs are supported");

    ensure!(
        let Fields::Named(struct_fields) = &strct.fields,
        strct.fields,
        "only named fields"
    );

    let (field_idents, field_names): (Vec<_>, Vec<_>) = struct_fields
        .named
        .iter()
        .filter(|f| {
            // Ignore all `STATE_IGNORE` attributes
            !f.attrs.iter().any(|attr| attr.path().is_ident(STATE_IGNORE))
        })
        .filter_map(|f| f.ident.as_ref())
        .map(|f| (f, f.to_string()))
        .unzip();

    Ok(quote! {
        # use ::anathema::state::{self, AnyMap, Type, Value, ValueRef, PendingValue, Path, state, Subscriber, CommonVal};
        # use ::std::any::Any;
        impl state::State for #name {
            fn type_info(&self) -> Type {
                Type::Composite
            }

            fn as_any_map(&self) -> Option<&dyn AnyMap> {
                Some(self)
            }
        }

        impl state::AnyMap for #name {
            fn lookup(&self, key: std::borrow::Cow<'_, str>) -> Option<PendingValue> {
                match key.as_ref() {
                    #(
                        #field_names => {
                            Some(self.#field_idents.reference())
                        }
                    )*
                    _ => None,
                }
            }
        }
    })
}
