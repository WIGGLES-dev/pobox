use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;

pub struct DispatchTrait {
    pub item: ItemTrait,
}

impl DispatchTrait {
    pub fn to_tokens(&self) -> TokenStream {
        quote! {}
    }
}
