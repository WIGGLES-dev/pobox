use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Item, ItemFn, ItemImpl, ItemStruct, parse_macro_input};
use util::{strip_impl, wrap_method};

mod borrow_mask;
mod casing;
mod dispatch_impl;
mod dispatch_trait;
mod flavor;
mod signature;
mod util;

#[proc_macro_attribute]
pub fn dispatch(
    attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut out = TokenStream::new();

    let input = tokens.clone();
    let mut item = parse_macro_input!(input as Item);

    match &mut item {
        Item::Mod(_) => {}
        Item::Impl(item) => {
            out.extend(dispatch_impl::DispatchImpl { item: item.clone() }.to_tokens());
            strip_impl(item);
            out.extend(item.into_token_stream());
        }
        Item::Trait(item) => {
            out.extend(dispatch_trait::DispatchTrait { item: item.clone() }.to_tokens());
            out.extend(item.into_token_stream());
        }
        _ => {}
    };

    out.into()
}

#[proc_macro_derive(BorrowMask)]
pub fn derive_borrow_mask(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(tokens as ItemStruct);
    borrow_mask::BorrowMaskDescriptor { item }
        .to_tokens()
        .into()
}

#[proc_macro_derive(State)]
pub fn derive_state(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
}

#[proc_macro_derive(Actor)]
pub fn derive_actor(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
}

#[proc_macro_derive(Message)]
pub fn derive_message(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
}

#[proc_macro_derive(Error)]
pub fn derive_error(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    quote! {}.into()
}
