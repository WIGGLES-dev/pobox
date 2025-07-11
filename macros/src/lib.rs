#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Item, ItemFn, ItemImpl, ItemStruct, Path, parse_macro_input};
use util::{strip_impl, wrap_method};

use crate::{
    signature::{PoboxAttr, extract_adapter_via_parse2},
    util::strip_trait,
};

mod borrow_mask;
mod casing;
mod dispatch_trait;
mod flavor;
mod signature;
mod util;

#[proc_macro_attribute]
pub fn dispatch(
    attr: proc_macro::TokenStream,
    tokens: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr = parse_macro_input!(attr as PoboxAttr);

    let mut out = TokenStream::new();

    let input = tokens.clone();
    let mut item = parse_macro_input!(input as Item);

    match &mut item {
        Item::Trait(item) => {
            out.extend(dispatch_trait::DispatchTrait { attr, item }.to_tokens());
            strip_trait(item);
            out.extend(item.into_token_stream());
        }
        _ => {
            panic!(
                "dispatch is only supported on traits, define the interface and then implement it"
            )
        }
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
