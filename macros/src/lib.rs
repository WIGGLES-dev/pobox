#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{Expr, Item, ItemFn, ItemImpl, ItemStruct, Path, parse_macro_input};

mod casing;
