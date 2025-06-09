use proc_macro2::Span;
use syn::{
    Block, FnArg, Ident, Pat, Signature, Token, Type, TypeNever, parse_quote,
    token::{Async, Bang},
};

pub struct InputMessagePayload {
    pub bind: Option<Ident>,
    pub needs_deref: bool,
    payload_type: Option<Type>,
}

impl InputMessagePayload {
    pub fn message_type(&self) -> Type {
        if let Some(t) = &self.payload_type {
            match t {
                Type::Path(_) => t.clone(),
                Type::Reference(reference) => {
                    let ty = &reference.elem;
                    parse_quote!(std::borrow::Cow<'static, #ty>)
                }
                Type::ImplTrait(_) => {
                    parse_quote!(Box<dyn std::any::Any>)
                }
                Type::TraitObject(_) => {
                    parse_quote!(Box<dyn std::any::Any>)
                }
                Type::Paren(_) => t.clone(),
                _ => {
                    panic!("unsupported message type")
                }
            }
        } else {
            Type::Never(TypeNever {
                bang_token: Bang {
                    spans: [Span::call_site()],
                },
            })
        }
    }
}

pub struct SignatureAnalysis {
    pub asyncness: Option<Async>,
    pub message_data: Vec<InputMessagePayload>,
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
    pub fn from_signature(signature: &Signature) -> Self {
        let asyncness = signature.asyncness.clone();
        let mut message_data = vec![];

        for (i, input) in signature.inputs.iter().enumerate() {
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

        Self {
            asyncness,
            message_data,
        }
    }
}
