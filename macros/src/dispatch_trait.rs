use crate::{
    casing::{pascal, snake},
    signature::{self, PoboxAttr, SignatureAnalysis},
};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{ExprMatch, GenericParam, Ident, ItemTrait, TraitItem, Type, TypeImplTrait, TypeParam};

pub struct DispatchTrait<'a> {
    pub attr: PoboxAttr,
    pub item: &'a mut ItemTrait,
}

impl<'a> DispatchTrait<'a> {
    pub fn to_tokens(&mut self) -> TokenStream {
        let ident = &self.item.ident;
        let trait_name = self.item.ident.to_string();
        let trait_snake_name = snake(&trait_name);

        let mailbox_ident = format_ident!("{trait_snake_name}_mailbox");
        let dispatch_ident = format_ident!("{trait_name}Dispatch");
        let match_ident = format_ident!("{trait_snake_name}_match");
        let ext_ident = format_ident!("{trait_name}Ext");
        let mut ext_methods = TokenStream::new();
        let mut ext_impl = TokenStream::new();

        let wasm_bindgen_ident = format_ident!("{trait_name}WasmBindgen");

        let mut trait_params = self.item.generics.params.to_token_stream();
        if !trait_params.is_empty() {
            trait_params.extend(quote! {,});
        }

        let (impl_generics, ty_generics, where_clause) = self.item.generics.split_for_impl();
        let mut phantom = TokenStream::new();
        if trait_params.is_empty() {
            phantom.extend(quote! {()});
        } else {
            phantom.extend(trait_params.clone());
        }

        let mut dispatch_structs = TokenStream::new();
        let mut dispatch_derives = TokenStream::new();
        let mut dispatch_variants = TokenStream::new();

        let mut reply_items = TokenStream::new();

        let mut match_branches = TokenStream::new();
        let mut delegated_variants = TokenStream::new();

        if self.attr.serde.unwrap_or(false) {
            dispatch_derives.extend(quote! { ::serde::Deserialize, ::serde::Serialize });
        }

        for item in &mut self.item.items {
            match item {
                TraitItem::Method(method) => {
                    let signature_analysis = SignatureAnalysis::from_trait_method(method);
                    let trait_method_pascal_ident = &signature_analysis.pascal_ident;

                    dispatch_structs.extend(signature_analysis.payload_all(
                        &dispatch_ident,
                        &self.item.generics,
                        if self.attr.serde.unwrap_or(false) {
                            // if serde is enabled, carry over all serde tags to put on the structs
                            Some(
                                method
                                    .attrs
                                    .iter()
                                    .filter(|attr| attr.path.is_ident("serde"))
                                    .cloned()
                                    .collect(),
                            )
                        } else {
                            None
                        },
                    ));

                    let payload_ident = &signature_analysis.pascal_ident;
                    dispatch_variants.extend(quote! {
                        #trait_method_pascal_ident(#payload_ident),
                    });

                    let sig_ident = &method.sig.ident;
                    let inputs = signature_analysis.binding_inputs();
                    let output = &signature_analysis.output;

                    ext_methods.extend(quote! {
                        fn #sig_ident(&self, #inputs);
                    });

                    let bindings: TokenStream = signature_analysis
                        .message_data
                        .iter()
                        .map(|md| {
                            let bind = &md.bind;
                            quote! { #bind, }
                        })
                        .collect();
                    ext_impl.extend(quote! {
                        fn #sig_ident(&self, #inputs) {
                            // self.send(#ident::new(#bindings));
                        }
                    });

                    reply_items.extend(quote! {
                        #sig_ident:
                    });
                }
                TraitItem::Macro(mac) => if mac.mac.path.is_ident("delegate") {},
                TraitItem::Type(ty) => {}
                TraitItem::Const(con) => {}
                _ => {}
            }
        }

        let dispatch_derives = if dispatch_derives.is_empty() {
            quote! { #[derive(#dispatch_derives)] }
        } else {
            quote! {}
        };

        let mut wasm_bindgen_impl = TokenStream::new();
        // there's not way to support generic wasm bindgen traits, so we'll fallback by generating a macro_rules to make concrete
        // types easier to implement
        if self.item.generics.params.empty_or_trailing() {
            wasm_bindgen_impl.extend(quote! {
                // #[::wasm_bindgen::prelude::wasm_bindgen(js_name = #ident)]
                // pub struct #wasm_bindgen_ident {
                //     actor: ::pobox::actor::ActorRef<#dispatch_ident>
                // }
            });
        } else {
            wasm_bindgen_impl.extend(quote! {
                macro_rules! wasm_impl {
                    () => {

                    };
                }
            });
        }

        quote! {
            pub mod #mailbox_ident {
                use super::*;

                #dispatch_structs

                #dispatch_derives
                pub enum SyncDispatch {}

                #dispatch_derives
                pub enum AsyncDispatch {}

                #dispatch_derives
                pub enum Dispatch #ty_generics #where_clause {
                    #dispatch_variants
                    __Phantom(::std::marker::PhantomData<#phantom>)
                }

                #wasm_bindgen_impl

                pub type #dispatch_ident #ty_generics = Dispatch #ty_generics;

                pub trait #ext_ident {
                    #ext_methods
                }

                impl #ty_generics #ext_ident for ::pobox::actor::ActorRef<#dispatch_ident #ty_generics> #where_clause {
                    #ext_impl
                }

                unsafe fn #match_ident
                <#trait_params DispatchImplFor: #ident #ty_generics>
                (
                    dispatch: #dispatch_ident #ty_generics,
                    state: &mut DispatchImplFor
                )
                #where_clause
                {
                    match dispatch {
                        _ => {}
                    }
                }

                pub type ActorRef #ty_generics = ::pobox::actor::ActorRef<Dispatch #ty_generics>;

                unsafe impl <#trait_params DispatchImplFor: #ident #ty_generics> ::pobox::rpc::Dispatch<DispatchImplFor> for #dispatch_ident #ty_generics
                #where_clause
                {
                    fn run(self, state: &mut DispatchImplFor) -> Result<(), ::pobox::rpc::DispatchError<Self>> {
                        todo!()
                    }
                }
            }
        }
    }
}
