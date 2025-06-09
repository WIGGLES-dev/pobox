use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Block, FnArg, Ident, ImplItem, ImplItemMethod, ItemImpl, Pat, PatIdent, PatType, Receiver,
    Signature, Token, Type, TypeReference, Visibility, parse_quote,
    token::{And, Mut},
};

fn recursively_replace_ref_in_pat(ref_bindings: &mut Vec<Ident>, pat: &mut Pat) {
    match pat {
        Pat::Struct(pat_struct) => {
            for field in &mut pat_struct.fields {
                match &mut *field.pat {
                    Pat::Ident(PatIdent {
                        by_ref, mutability, ..
                    }) => {
                        *by_ref = None;
                    }
                    pat => {
                        recursively_replace_ref_in_pat(ref_bindings, pat);
                    }
                }
            }
        }
        Pat::Tuple(_) => {}
        Pat::TupleStruct(_) => {}
        _ => {}
    }
}

/// strip ref destructures from &mut Self mutable borrows. while syntactically valid rust, it is a parse error.
/// unfortunately there's not way to reasonable express struct destructuring where some fields are mutably borrowed
/// and others are immutably borrows
pub fn strip_method(method: &mut ImplItemMethod) -> Vec<Ident> {
    let first_arg = &mut method.sig.inputs[0];
    let mut bindings = vec![];
    match first_arg {
        FnArg::Typed(pat) => {
            match &mut *pat.ty {
                Type::Reference(TypeReference { elem, .. }) => {
                    match elem.as_mut() {
                        Type::Path(path) => {
                            if let Some(ident) = path.path.get_ident() {
                                if ident.to_string() == "Self" {
                                    recursively_replace_ref_in_pat(&mut bindings, &mut pat.pat);
                                }
                            }
                        }
                        _ => {}
                    };
                }
                _ => {}
            };
        }
        _ => {}
    };
    bindings
}

fn receiver(is_mut: bool) -> FnArg {
    FnArg::Receiver(Receiver {
        attrs: Vec::new(),
        reference: Some((
            And {
                spans: [Span::call_site()],
            },
            None, // Optional lifetime: Some(Lifetime::new("'a", Span::call_site()))
        )),
        mutability: if is_mut {
            Some(Mut {
                span: Span::call_site(),
            })
        } else {
            None
        },
        self_token: Token![self](Span::call_site()),
    })
}

/// wrap an impl item method that takes &mut Self or &Self rather than &mut self or &self in a signature that does support it
pub fn wrap_method(method: &mut ImplItemMethod) -> Option<ImplItemMethod> {
    let mut replace = method.clone();
    let mut needs_wrap = false;
    if let Some(would_be_receiver) = replace.sig.inputs.first_mut() {
        match would_be_receiver {
            FnArg::Typed(pat_type) => {
                match &mut *pat_type.ty {
                    Type::Reference(reference) => match &mut *reference.elem {
                        Type::Path(path) => {
                            if path.path.get_ident().unwrap().to_string() == "Self" {
                                *would_be_receiver = receiver(reference.mutability.is_some());
                                needs_wrap = true;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                };
            }
            _ => {}
        }
    }

    if needs_wrap {
        method.attrs.push(parse_quote!(#[doc(hidden)]));
        method.vis = Visibility::Inherited;
        method.sig.ident = Ident::new(&format!("__{}", method.sig.ident), Span::call_site());

        let maybe_await = method.sig.asyncness.map(|_| quote! {.await});

        let fn_name = &method.sig.ident;
        let forwarding: TokenStream = get_simple_impl_forwarding_name(&method.sig)
            .map(|(_is_borrow, ident)| {
                quote! { #ident, }
            })
            .collect();
        let new_block: Block = parse_quote! {
            {
                Self::#fn_name(self, #forwarding)#maybe_await
            }
        };
        replace.block = new_block;
        Some(replace)
    } else {
        None
    }
}

pub fn strip_impl(item: &mut ItemImpl) {
    let mut extra_items = vec![];
    for item in &mut item.items {
        match item {
            ImplItem::Method(method) => {
                strip_method(method);
                if let Some(append) = wrap_method(method) {
                    extra_items.push(append);
                }
            }
            _ => {}
        }
    }
    item.items
        .extend(extra_items.drain(0..).map(|item| ImplItem::Method(item)));
}

pub fn get_simple_impl_forwarding_name(sig: &Signature) -> impl Iterator<Item = (bool, Ident)> {
    sig.inputs.iter().skip(1).map(|fnarg| match fnarg {
        FnArg::Typed(PatType { pat, ty, .. }) => match &**pat {
            Pat::Ident(ident) => {
                let is_borrow = match &**ty {
                    Type::Reference(_) => true,
                    _ => false,
                };

                (is_borrow, ident.ident.clone())
            }
            _ => {
                panic!("forwarding does not support destructuring in forwarded args")
            }
        },
        _ => {
            unreachable!()
        }
    })
}
