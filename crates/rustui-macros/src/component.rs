use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Fields, Ident, ItemStruct, Type, parse_quote};

pub fn expand(item: TokenStream) -> syn::Result<TokenStream> {
    let mut structure = syn::parse::<ItemStruct>(item)?;
    let Fields::Named(fields) = &mut structure.fields else {
        return Err(syn::Error::new_spanned(
            &structure,
            "`#[component]` currently requires a struct with named fields",
        ));
    };

    let mut state_fields = Vec::<(Ident, Type, Expr)>::new();
    let mut default_fields = Vec::<(Ident, Type)>::new();

    for field in &mut fields.named {
        let state_attribute = field
            .attrs
            .iter()
            .position(|attribute| attribute.path().is_ident("state"));
        let Some(index) = state_attribute else {
            let name = field.ident.clone().expect("named fields have identifiers");
            default_fields.push((name, field.ty.clone()));
            continue;
        };

        let attribute = field.attrs.remove(index);
        let initial_value = attribute.parse_args::<Expr>()?;
        let name = field.ident.clone().expect("named fields have identifiers");
        state_fields.push((name, field.ty.clone(), initial_value));
    }

    let name = &structure.ident;
    let generics = &structure.generics;
    let state_initializers = state_fields
        .iter()
        .map(|(field, _, initial)| quote!(#field: #initial));
    let default_initializers = default_fields
        .iter()
        .map(|(field, _)| quote!(#field: ::core::default::Default::default()));
    let accessors = state_fields.iter().map(|(field, ty, _)| {
        let setter = format_ident!("set_{field}");
        let updater = format_ident!("update_{field}");
        let binding = format_ident!("{field}_binding");
        let state = format_ident!("{field}_state");
        quote! {
            pub fn #field(&self) -> #ty
            where
                #ty: ::core::clone::Clone,
            {
                self.#field.clone()
            }

            pub fn #setter(&mut self, value: #ty) -> bool
            where
                #ty: ::core::cmp::PartialEq,
            {
                if self.#field == value {
                    return false;
                }
                self.#field = value;
                true
            }

            pub fn #updater<Output>(
                &mut self,
                update: impl ::core::ops::FnOnce(&mut #ty) -> Output,
            ) -> Output {
                update(&mut self.#field)
            }

            pub fn #binding(&self) -> ::rustui::runtime::Binding<#ty>
            where
                Self: 'static,
                #ty: ::core::clone::Clone + 'static,
            {
                ::rustui::runtime::Binding::new(
                    self.#field.clone(),
                    |component: &Self| component.#field.clone(),
                    |component: &mut Self, value| component.#field = value,
                )
            }

            pub fn #state(&self) -> ::rustui::runtime::State<'_, #ty>
            where
                Self: 'static,
                #ty: ::core::clone::Clone + 'static,
            {
                ::rustui::runtime::State::new(&self.#field, self.#binding())
            }
        }
    });

    let mut default_generics = generics.clone();
    for (_, ty) in &default_fields {
        default_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::core::default::Default));
    }
    let (default_impl_generics, default_type_generics, default_where_clause) =
        default_generics.split_for_impl();
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        #structure

        impl #default_impl_generics ::core::default::Default for #name #default_type_generics
            #default_where_clause
        {
            fn default() -> Self {
                Self {
                    #(#state_initializers,)*
                    #(#default_initializers,)*
                }
            }
        }

        impl #impl_generics #name #type_generics #where_clause {
            #(#accessors)*

            pub fn action(
                &self,
                handler: fn(&mut Self),
            ) -> ::rustui::runtime::Action {
                ::rustui::runtime::Action::new(handler)
            }

            pub fn action_if_changed(
                &self,
                handler: fn(&mut Self) -> bool,
            ) -> ::rustui::runtime::Action {
                ::rustui::runtime::Action::new_if_changed(handler)
            }
        }
    }
    .into())
}
