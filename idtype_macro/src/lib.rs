use proc_macro::TokenStream;
use quote::quote;
use std::hash::Hash;
use std::{collections::hash_map::DefaultHasher, hash::Hasher};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(IdType)]
pub fn idtype_derive(raw_input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(raw_input as DeriveInput);

    // Hash the name of the crate and the type.
    // This is not a unique combination but it isn't possible to access
    // the name of the current module on stable.
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    let mut hasher = DefaultHasher::new();
    input.ident.hash(&mut hasher);
    crate_name.hash(&mut hasher);
    let hash = hasher.finish();

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::coproduct::type_inequality::IdType
            for #name #ty_generics #where_clause {

            const ID: u64 = #hash;
        }
    }
    .into()
}
