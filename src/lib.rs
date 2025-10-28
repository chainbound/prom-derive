use proc_macro::TokenStream;
use syn::{ItemStruct, parse_macro_input};

use crate::expand::MetricsAttr;

mod expand;

/// This macro is used to derive the Prometheus metrics for a struct.
#[proc_macro_attribute]
pub fn metrics(attr: TokenStream, item: TokenStream) -> TokenStream {
    // NOTE: We use `proc_macro_attribute` here because we're actually rewriting the struct. Derive macros are additive.
    let mut input = parse_macro_input!(item as ItemStruct);

    let attributes: MetricsAttr = match syn::parse(attr) {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    expand::expand(attributes, &mut input)
        .unwrap_or_else(|err| err.into_compile_error())
        .into()
}
