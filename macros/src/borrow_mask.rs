use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    Expr, FieldPat, Fields, FnArg, ImplItemMethod, Index, ItemStruct, Local, Member, Pat,
    PatStruct, PatType, Path, Signature, Stmt,
};

pub struct BorrowMaskDescriptor {
    pub item: ItemStruct,
}

const DESCRIPTOR_REF_IDENT: &'static str = "__mailbox_descriptor";
const SCOPE_PREFIX: &'static str = "__mailbox_";

impl BorrowMaskDescriptor {
    pub fn struct_descriptor(root: &Ident, fields: &Fields) -> (Ident, TokenStream, TokenStream) {
        let descriptor_ident = Ident::new(
            &format!("{SCOPE_PREFIX}{}", root.to_string()),
            Span::call_site(),
        );
        let mut descriptor_fields = TokenStream::new();
        let mut descriptor_values = TokenStream::new();

        for (i, field) in fields.iter().enumerate() {
            if let Some(ident) = &field.ident {
                let ident = Ident::new(
                    &format!("{SCOPE_PREFIX}{}", ident.to_string()),
                    Span::call_site(),
                );
                descriptor_fields.extend(quote! {
                    #[doc(hidden)]
                    #ident: usize,
                });
                descriptor_values.extend(quote! {
                    #[doc(hidden)]
                    #ident: #i,
                });
            }
        }

        (
            descriptor_ident.clone(),
            quote! {
                pub struct #descriptor_ident {
                    #descriptor_fields
                }
            },
            quote! {
                #descriptor_ident {
                    #descriptor_values
                }
            },
        )
    }

    pub fn access_helpers(fields: &Fields) -> TokenStream {
        let stream = TokenStream::new();

        stream
    }
}

impl BorrowMaskDescriptor {
    pub fn to_tokens(&self) -> TokenStream {
        let ident = &self.item.ident;
        let (descriptor_ident, descriptor, descriptor_value) =
            BorrowMaskDescriptor::struct_descriptor(&self.item.ident, &self.item.fields);
        let access_helpers = BorrowMaskDescriptor::access_helpers(&self.item.fields);
        let descriptor_reference = Ident::new(DESCRIPTOR_REF_IDENT, Span::call_site());

        let num_fields = self.item.fields.len();

        quote! {
            #descriptor
            impl #ident {
                const #descriptor_reference: #descriptor_ident = #descriptor_value;
                #access_helpers
            }

            unsafe impl pobox::borrow::BorrowMask for #ident {
                const BITFIELD_LEN: usize = pobox::borrow::bitfield_len::<#num_fields>();
                const BITMASK_LEN: usize = #num_fields;
                type Bitfield = pobox::borrow::BitfieldState<#num_fields>;
            }
        }
    }
}

pub struct BorrowMask {
    signature: Signature,
}

struct RecursiveMask {
    fields: Vec<Vec<Member>>,
}

impl RecursiveMask {
    /// recursively descend through the pat struct building up a chain of destructuring to forward to bitmask positions
    /// generate by the [BorrowMask] macro.
    fn process_field(&mut self, access_path: Vec<Member>, pat: &Pat) {
        match pat {
            Pat::Struct(pat_struct) => {
                for field in &pat_struct.fields {
                    let mut access_path = access_path.clone();
                    access_path.push(field.member.clone());
                    self.process_field(access_path, &field.pat);
                }
            }
            Pat::Tuple(pat_tuple) => {
                for (i, elem) in pat_tuple.elems.iter().enumerate() {
                    let mut access_path = access_path.clone();
                    access_path.push(Member::Unnamed(Index::from(i)));
                    self.process_field(access_path, elem);
                }
            }
            Pat::TupleStruct(pat_tuple_struct) => {
                for (i, elem) in pat_tuple_struct.pat.elems.iter().enumerate() {
                    let mut access_path = access_path.clone();
                    access_path.push(Member::Unnamed(Index::from(i)));
                    self.process_field(access_path, elem);
                }
            }
            _ => {
                panic!("only struct, tuple and struct tuple destructuring is allowed")
            }
        }
    }

    /// from a root type, build out the recursive mask. this recursive mask will be used to generate some compile time
    /// rust glue code to output a static [u32;N] array for this access pattern.
    pub fn from_root_pat(pat: &Pat) -> Self {
        let mut mask = Self { fields: vec![] };
        mask.process_field(vec![], &pat);
        mask
    }
}

/// given a FnArg create the glue code that create a bitmask based on the destructuring of the FnArg. this is basically the
/// entire point of this crate.
pub fn mask_for_fn_arg(arg: &FnArg) -> TokenStream {
    match arg {
        // always borrow all or borrow all mut
        FnArg::Receiver(_) => {
            panic!(
                "in the spirit of this crate, do not use &self or &mut self receivers for message dispatch"
            );
        }
        FnArg::Typed(pat) => {
            let mask = RecursiveMask::from_root_pat(&*pat.pat);
            match &*pat.pat {
                Pat::Struct(pat_struct) => {}
                Pat::Tuple(pat_tuple) => {}
                Pat::TupleStruct(pat_tuple_struct) => {}
                _ => {
                    panic!(
                        "only destructuring patterns for structs, tuples and tuple structs are supported"
                    )
                }
            }

            quote! {}
        }
    }
}

impl BorrowMask {
    pub fn to_tokens(&self) -> TokenStream {
        if let Some(first) = self.signature.inputs.first() {
            //
        }

        quote! {}
    }
}

fn detect_self_acces_expr(expr: &Expr) {}
fn detect_self_access(stmt: &Stmt) {
    match stmt {
        Stmt::Expr(expr) => detect_self_acces_expr(expr),
        Stmt::Semi(expr, _) => detect_self_acces_expr(expr),

        Stmt::Local(Local {
            init: Some((_, expr)),
            ..
        }) => detect_self_acces_expr(&*expr),
        _ => {}
    }
}
pub fn mask_detail_from_method(method: &ImplItemMethod) {
    for stmt in method.block.stmts.iter() {
        detect_self_access(stmt);
    }
}
