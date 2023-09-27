use proc_macro::TokenStream;
use quote::quote;
// extern crate embed_md_traits;
// use embed_md_traits::Rangeable;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(RangeFn)]
pub fn derive_range_fn(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let expanded = quote! {
        impl Rangeable for #name {
            fn range(&self) -> Range<usize> {
                return self.range.clone()
            }

            fn id(&self) -> String {
                return self.id.clone()
            }
        }
    };
    TokenStream::from(expanded)
}
