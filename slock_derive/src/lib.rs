extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn;

#[proc_macro_derive(Slockable)]
pub fn slockable(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let name = &ast.ident;
    let slocker_name = syn::Ident::new(&format!("{}Slocker", name), name.span());
    let data = if let syn::Data::Struct(data) = &ast.data {
        data
    } else {
        return quote_spanned! {
            name.span() => compile_error!("Must be placed on a struct")
        }
        .into();
    };
    let gen = quote! {
        struct #slocker_name(Slock<#name>);

        impl #slocker_name {

        }

        impl Slockable for #name {
            type Slocker = #slocker_name;

            fn get_slocker(lock: Slock<#name>) -> #slocker_name {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };

    println!("{}", gen);
    gen.into()
}
