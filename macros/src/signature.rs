use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Attribute, Block, Expr, ExprLit, Field, FnArg, Generics, Ident, ImplItemMethod, ItemStruct,
    LitStr, Pat, PatType, Path, ReturnType, Signature, Token, TraitItemMethod, Type, TypeNever,
    TypePath,
    parse::{Parse, ParseStream, Parser},
    parse_quote,
    punctuated::Punctuated,
    token::{Async, Bang, Comma},
};

use crate::casing::pascal;

pub enum PayloadFlavor {
    Asis,
    Cow { make: TokenStream },
    Box,
}
pub struct InputMessagePayload {
    pub attributes: Vec<Attribute>,
    pub bind: Option<Ident>,
    pub needs_deref: bool,
    pub payload_type: Option<Type>,
}

impl InputMessagePayload {
    pub fn message_type(&self) -> (PayloadFlavor, Type) {
        if let Some(t) = &self.payload_type {
            match t {
                Type::Path(_) => (PayloadFlavor::Asis, t.clone()),
                Type::Reference(reference) => {
                    let ty = &reference.elem;
                    (
                        PayloadFlavor::Cow {
                            make: quote! { ::std::borrow::Cow::<#ty> },
                        },
                        parse_quote!(::std::borrow::Cow<'static, #ty>),
                    )
                }
                Type::ImplTrait(ty_impl) => {
                    let bounds = &ty_impl.bounds;
                    (PayloadFlavor::Box, parse_quote!(Box<dyn #bounds>))
                }
                Type::TraitObject(trait_obj) => (PayloadFlavor::Box, parse_quote!(Box<#trait_obj>)),
                Type::Paren(_) => (PayloadFlavor::Asis, t.clone()),
                _ => {
                    panic!("unsupported message type")
                }
            }
        } else {
            (
                PayloadFlavor::Asis,
                Type::Never(TypeNever {
                    bang_token: Bang {
                        spans: [Span::call_site()],
                    },
                }),
            )
        }
    }
}

pub enum ReturnChannel {
    Single(Type),
    Multiple(Type),
}

pub struct SignatureAnalysis {
    pub pascal_ident: Ident,
    pub message_data: Vec<InputMessagePayload>,
    pub reply_port: Option<Type>,
    pub output: ReturnType,
}

fn get_ty_or_panic(pat: &Pat) -> Type {
    match pat {
        Pat::Reference(pat) => get_ty_or_panic(&pat.pat),
        Pat::Type(pat) => *pat.ty.to_owned(),
        _ => {
            panic!("unsupported signature")
        }
    }
}

impl SignatureAnalysis {
    pub fn wrapped_payload(&self) -> TokenStream {
        let payload_ident = &self.pascal_ident;
        let reply_port = &self.reply_port;
        quote! {
            ::pobox::rpc::Respondable<#payload_ident, #reply_port>
        }
    }

    pub fn payload_all(
        &self,
        root: &Ident,
        generics: &Generics,
        serde: Option<Vec<Attribute>>,
    ) -> TokenStream {
        let payload_struct = self.payload_struct(serde);
        let payload_new = self.payload_new();

        quote! {
            #payload_struct
            #payload_new
        }
    }

    pub fn payload_struct(&self, serde: Option<Vec<Attribute>>) -> TokenStream {
        let payload_ident = &self.pascal_ident;
        let fields: TokenStream = self
            .message_data
            .iter()
            .map(|md| {
                let ident = md
                    .bind
                    .as_ref()
                    .expect("all message payloads must have an ident");
                let (_, payload) = md.message_type();

                let mut out = TokenStream::new();

                if let Some(serde) = serde.as_ref() {
                    let serde_attributes = md
                        .attributes
                        .iter()
                        .filter(|attr| attr.path.is_ident("serde"));
                    for attr in serde_attributes {
                        out.extend(attr.to_token_stream());
                    }
                }

                out.extend(quote! {
                    pub #ident: #payload,
                });

                out
            })
            .collect();

        let mut derives = TokenStream::new();

        if let Some(serde) = serde {
            derives.extend(quote! {
                ::serde::Deserialize, ::serde::Serialize
            });
        }

        let derives = if derives.is_empty() {
            quote! {}
        } else {
            quote! { #[derive(#derives)] }
        };

        quote! {
            #derives
            pub struct #payload_ident { #fields }
        }
    }

    pub fn payload_new(&self) -> TokenStream {
        let payload_ident = &self.pascal_ident;
        let inputs = self.binding_inputs();
        let payload_fields = self.payload_fields();
        quote! {
            impl #payload_ident {
                pub fn new(#inputs) -> Self {
                    Self { #payload_fields }
                }
            }
        }
    }

    pub fn binding_inputs(&self) -> TokenStream {
        self.message_data
            .iter()
            .map(|md| {
                let ident = &md.bind;
                let ty = &md.payload_type;
                quote!(#ident: #ty,)
            })
            .collect()
    }

    pub fn payload_fields(&self) -> TokenStream {
        self.message_data
            .iter()
            .map(|md| {
                let ident = &md.bind;
                let ty = &md.payload_type;
                let (flavor, msg_ty) = &md.message_type();

                match flavor {
                    PayloadFlavor::Asis => {
                        quote!(#ident:#ident,)
                    }
                    PayloadFlavor::Cow { make } => {
                        quote!(#ident: #make::Owned(::std::borrow::ToOwned::to_owned(#ident)),)
                    }
                    PayloadFlavor::Box => {
                        quote!(#ident: Box::new(#ident),)
                    }
                }
            })
            .collect()
    }
    pub fn from_trait_method(method: &mut TraitItemMethod) -> Self {
        Self::from_sig_attrs(&mut method.sig, &mut method.attrs)
    }
    pub fn from_method(method: &mut ImplItemMethod) -> Self {
        Self::from_sig_attrs(&mut method.sig, &mut method.attrs)
    }
    fn from_sig_attrs(signature: &mut Signature, attrs: &mut [Attribute]) -> Self {
        let pobox = attrs
            .iter()
            .find_map(extract_adapter_via_parse2)
            .unwrap_or_default();

        if signature.asyncness.is_some() {
            panic!("async dispatch traits are not supported, spawn a future and use reply ports");
        }

        let mut message_data = vec![];

        for (i, input) in signature.inputs.iter().enumerate() {
            if i == signature.inputs.len() - 1 {
                break;
            }
            match input {
                FnArg::Receiver(_) => {}
                FnArg::Typed(pat_ty) => {
                    match &*pat_ty.pat {
                        // a named ident
                        Pat::Ident(ident) => {
                            let needs_deref = match &*pat_ty.ty {
                                Type::Reference(_) => true,
                                _ => false,
                            };

                            message_data.push(InputMessagePayload {
                                attributes: pat_ty.attrs.clone(),
                                bind: Some(ident.ident.clone()),
                                needs_deref,
                                payload_type: if let Some((_, subpat)) = &ident.subpat {
                                    let ty = get_ty_or_panic(&*subpat);
                                    Some(ty)
                                } else {
                                    Some(*pat_ty.ty.to_owned())
                                },
                            });
                        }
                        // struct dereference
                        Pat::Struct(pat) => {
                            if i != 0 {
                                panic!(
                                    "only the first argument in a method may be destructured the rest are message args"
                                )
                            }
                        }

                        _ => {
                            panic!(
                                "currently only Self destructuring and named fields are supported"
                            )
                        }
                    };
                }
            }
        }

        let user_attr = attrs.iter().find_map(extract_adapter_via_parse2);

        let pascal_name = pascal(&signature.ident.to_string());
        let pascal_ident = Ident::new(&pascal_name, Span::call_site());

        Self {
            pascal_ident,
            message_data,
            reply_port: signature.inputs.last_mut().and_then(|fn_arg| match fn_arg {
                FnArg::Receiver(_) => None,
                FnArg::Typed(ty) => match &mut *ty.ty {
                    Type::ImplTrait(impl_trait) => {
                        //
                        Some(Type::ImplTrait(impl_trait.clone()))
                    }
                    Type::Macro(mac) => {
                        let reply_ty = mac
                            .mac
                            .parse_body::<Type>()
                            .expect("the output declaration must be a valid type");
                        let mac_tokens = &mac.mac.tokens;
                        *ty.ty = syn::parse_quote!(impl ::pobox::rpc::#mac_tokens);
                        Some(reply_ty)
                    }
                    _ => None,
                },
            }),
            output: signature.output.clone(),
        }
    }
}

#[derive(Default)]
pub struct PoboxAttr {
    pub serde: Option<bool>,
}

impl Parse for PoboxAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut serde = None;

        let Ok(ident) = input.parse::<Path>() else {
            return Ok(PoboxAttr { serde });
        };

        loop {
            match input.parse::<Token![=]>() {
                Ok(_) => {
                    let _: Expr = input.parse()?;
                }
                Err(_) => {
                    if ident.is_ident("serde") {
                        serde = Some(true);
                    }
                }
            }
            if input.is_empty() {
                break;
            }

            input.parse::<Token![,]>()?;
        }

        Ok(PoboxAttr { serde })
    }
}

pub fn extract_adapter_via_parse2(attr: &Attribute) -> Option<PoboxAttr> {
    return attr.parse_args().ok();
}
