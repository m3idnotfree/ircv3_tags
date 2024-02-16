use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(irc3_tags)]
pub fn irc3_tags_derive(input: TokenStream) -> TokenStream {
    // TokenStream::new()
    let input = parse_macro_input!(input as DeriveInput);
    // expand_getters(input)
    let DeriveInput { ident, .. } = input;
    let output = quote! {
        impl Irc3TagsParse for #ident{
        }
    };
    output.into()
}

