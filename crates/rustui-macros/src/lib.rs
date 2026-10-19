extern crate proc_macro;

mod component;
mod view_builder;

use proc_macro::TokenStream;

/// Adds generated state accessors, default construction, and typed actions.
#[proc_macro_attribute]
pub fn component(attributes: TokenStream, item: TokenStream) -> TokenStream {
    if !attributes.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`component` takes no arguments",
        )
        .to_compile_error()
        .into();
    }

    match component::expand(item) {
        Ok(output) => output,
        Err(error) => error.to_compile_error().into(),
    }
}

/// Builds a vertical stack from constructor expressions and `if`/`else` branches.
/// Branch children are flattened; use explicit `Keyed` views for stable identity.
#[allow(non_snake_case)]
#[proc_macro]
pub fn VStack(input: TokenStream) -> TokenStream {
    match view_builder::expand(input) {
        Ok(output) => output,
        Err(error) => error.to_compile_error().into(),
    }
}

/// Builds a horizontal stack from constructor expressions and `if`/`else` branches.
/// Branch children are flattened; use explicit `Keyed` views for stable identity.
#[allow(non_snake_case)]
#[proc_macro]
pub fn HStack(input: TokenStream) -> TokenStream {
    match view_builder::expand_horizontal(input) {
        Ok(output) => output,
        Err(error) => error.to_compile_error().into(),
    }
}
