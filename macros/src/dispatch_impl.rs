use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{
    ImplItem, ImplItemMethod, ItemImpl, ReturnType, Type, TypePath, parse_quote, token::Async,
};

use crate::{casing, signature::SignatureAnalysis};

pub struct DispatchImpl {
    pub item: ItemImpl,
}

impl DispatchImpl {
    fn ident(&self) -> Option<&Ident> {
        match &*self.item.self_ty {
            Type::Path(TypePath { path, qself }) => path.get_ident(),
            _ => {
                panic!("dispatch is only supported for simple concrete types");
            }
        }
    }

    fn impl_item_method_ident(method: &ImplItemMethod) -> Ident {
        let ident = &method.sig.ident;
        let ident = Ident::new(&casing::pascal(&ident.to_string()), Span::call_site());
        ident
    }

    fn impl_item_method_dispatch(
        analyis: &SignatureAnalysis,
        dispatch_ident: &Ident,
        ident: &Ident,
        method: &ImplItemMethod,
    ) -> TokenStream {
        let fields: TokenStream = analyis
            .message_data
            .iter()
            .map(|(md)| {
                let ident = md
                    .bind
                    .as_ref()
                    .expect("all message payloads must have an ident");
                let payload = md.message_type();
                quote! {
                    pub #ident: #payload,
                }
            })
            .collect();
        quote! {
            pub struct #ident { #fields }
            impl Into<#dispatch_ident> for #ident {
                fn into(self) -> #dispatch_ident {
                    #dispatch_ident::#ident(self)
                }
            }
            impl pobox::rpc::IntoDispatch for #ident {
                type Dispatch = #dispatch_ident;
                fn into_dispatch(self) -> Self::Dispatch {
                    Into::into(self)
                }
            }
        }
    }

    fn impl_item_method_dispatch_trait(
        root: &Ident,
        ident: &Ident,
        method: &ImplItemMethod,
    ) -> TokenStream {
        quote! {
            unsafe impl pobox::rpc::Dispatch for #ident {
                const ASYNC: bool = false;
                type State = #root;

                fn run(self, state: &Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    todo!()
                }
                fn run_mut(self, state: &mut Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    todo!()
                }
            }
        }
    }

    fn match_branch(
        &self,
        analysis: &SignatureAnalysis,
        root: &Ident,
        variant: &Ident,
        impl_root: &ItemImpl,
        method: &ImplItemMethod,
    ) -> TokenStream {
        let ident = &method.sig.ident;

        let mut destructure_bindings = TokenStream::new();
        let mut call_bindings = TokenStream::new();

        for md in analysis.message_data.iter() {
            if let Some(ident) = md.bind.as_ref() {
                destructure_bindings.extend(quote! { #ident, });
                if md.needs_deref {
                    call_bindings.extend(quote! { &*#ident, });
                } else {
                    call_bindings.extend(quote! { #ident, });
                }
            } else {
                panic!("all message payload params must have an ident")
            }
        }

        let maybe_await = analysis.asyncness.map(|_| quote! {.await});

        quote! {
            #root::#variant(message) => {
                let #variant { #destructure_bindings } = message;
                state.#ident(#call_bindings)#maybe_await;
            }
        }
    }

    fn enum_variant(&self, ident: &Ident, method: &ImplItemMethod) -> TokenStream {
        quote! { #ident(#ident), }
    }

    fn struct_field(&self, fn_name: &Ident, ident: &Ident, method: &ImplItemMethod) -> TokenStream {
        quote! { pub #fn_name: pobox::proxy::ProxyRef<#ident>, }
    }

    fn ext_methods(&self) -> TokenStream {
        let mut methods = TokenStream::new();

        for item in &self.item.items {
            match item {
                ImplItem::Method(method) => {
                    let ident = &method.sig.ident;
                    let ret = match &method.sig.output {
                        ReturnType::Type(_, ty) => ty.to_owned(),
                        _ => parse_quote!(()),
                    };
                    methods.extend(quote! {
                        fn #ident(&mut self) -> impl std::future::Future<Output = #ret> {
                            let t: Option<#ret> = None;
                            async {
                                todo!()
                            }
                        }
                    });
                }
                _ => {}
            }
        }

        methods
    }

    pub fn to_tokens(&self) -> TokenStream {
        let ident = self
            .ident()
            .expect("dispatch is only supported for simple concrete types");

        let mailbox_ident = Ident::new(
            &format!("{}_mailbox", casing::snake(&ident.to_string())),
            Span::call_site(),
        );

        let ext_ident = Ident::new(&format!("{ident}DispatchExt"), Span::call_site());
        let ext_methods = self.ext_methods();

        let dispatch_ident = Ident::new(&format!("{}Dispatch", ident), Span::call_site());
        let sync_dispatch_ident = Ident::new(&format!("{}SyncDispatch", ident), Span::call_site());
        let async_dispatch_ident =
            Ident::new(&format!("{}AsyncDispatch", ident), Span::call_site());

        let fan_ident = Ident::new(&format!("{}Fan", ident), Span::call_site());
        let match_ident = Ident::new(
            &format!("{}_match", casing::snake(&ident.to_string())),
            Span::call_site(),
        );

        let mut dispatch_items = TokenStream::new();
        let mut variants = TokenStream::new();
        let mut fans = TokenStream::new();
        let mut match_branches = TokenStream::new();

        let mut asyncness = None;

        for item in &self.item.items {
            match item {
                ImplItem::Method(method) => {
                    let signature_analysis = SignatureAnalysis::from_signature(&method.sig);
                    asyncness = asyncness.or(signature_analysis.asyncness);

                    let pascal_ident = Self::impl_item_method_ident(method);
                    let variant = self.enum_variant(&pascal_ident, method);
                    let fan = self.struct_field(&method.sig.ident, &pascal_ident, method);
                    let dispatch = Self::impl_item_method_dispatch(
                        &signature_analysis,
                        &dispatch_ident,
                        &pascal_ident,
                        &method,
                    );
                    let dispatch_impl =
                        Self::impl_item_method_dispatch_trait(&ident, &pascal_ident, &method);
                    let branch = self.match_branch(
                        &signature_analysis,
                        &dispatch_ident,
                        &pascal_ident,
                        &self.item,
                        method,
                    );

                    dispatch_items.extend(dispatch);
                    dispatch_items.extend(dispatch_impl);
                    variants.extend(variant);
                    fans.extend(fan);
                    match_branches.extend(branch);
                }
                _ => {
                    panic!(
                        "dispatch is only implemented for impl blocks where all types are methods"
                    );
                }
            }
        }

        let is_async = asyncness.is_some();
        let dispatch_impl = if asyncness.is_some() {
            quote! {
                fn run(self, state: &Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    panic!("cannot synchronously run async actor");
                }
                fn run_mut(self, state: &mut Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    panic!("cannot synchronously run async actor");
                }

                async fn spawn(self, state: &Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    todo!()
                }

                async fn spawn_mut(self, state: &mut Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    #match_ident(state, self).await;
                    Ok(())
                }
            }
        } else {
            quote! {
                fn run(self, state: &Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    todo!()
                }
                fn run_mut(self, state: &mut Self::State) -> Result<(), pobox::rpc::DispatchError<Self>> {
                    #match_ident(state, self);
                    Ok(())
                }
            }
        };

        quote! {
            pub mod #mailbox_ident {
                use super::*;
                #dispatch_items

                pub trait #ext_ident {
                    #ext_methods
                }

                impl #ext_ident for pobox::actor::ActorRef<#dispatch_ident> {
                    #ext_methods
                }

                pub enum #sync_dispatch_ident {}
                pub enum #async_dispatch_ident {}
                pub enum #dispatch_ident {
                    #variants
                }
                // pub struct #fan_ident {
                //    #fans
                // }
                pub #asyncness fn #match_ident(state: &mut #ident, dispatch: #dispatch_ident) {
                    match dispatch {
                        #match_branches
                        _ => {}
                    }
                }

                unsafe impl pobox::rpc::Dispatch for #dispatch_ident {
                    const ASYNC: bool = #is_async;
                    type State = #ident;
                    #dispatch_impl
                }

                unsafe impl pobox::disjoint::Disjoint for #ident {
                    fn try_borrow(&self, state: &mut u64, mask: u64) -> Result<&mut Self, pobox::disjoint::Borrowed> {
                        todo!()
                    }
                }
            }

        }
    }
}
