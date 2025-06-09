use proc_macro2::TokenStream;
use quote::quote;
use syn::ImplItemMethod;

/// the default flavor is synchronous &self receivers with a concrete simple return type
pub enum Flavor {
    AsyncDispatch,
    StreamDispatch,
    IterDispatch,
    MutSelf,
}

pub fn flavors_from_impl_item_method(impl_method: ImplItemMethod) -> Vec<Flavor> {
    vec![]
}

impl Flavor {
    pub fn trait_bounds(&self) -> TokenStream {
        quote! {}
    }
}
