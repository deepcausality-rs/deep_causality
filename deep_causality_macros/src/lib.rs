// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use proc_macro::TokenStream;

use getters::expand_getters;

use crate::collections::*;
use crate::constructor::expand_constructor;

mod collections;
mod constructor;
mod getters;

#[proc_macro_derive(Constructor, attributes(new))]
pub fn derive(input: TokenStream) -> TokenStream {
    expand_constructor(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Getters, attributes(getter))]
pub fn getters(input: TokenStream) -> TokenStream {
    expand_getters(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

// The macros below are code generators used to implement type extensions with minimal boilerplate.
// See deep_causality/src/extensions

#[proc_macro]
pub fn make_len(_: TokenStream) -> TokenStream {
    expand_make_len()
}

#[proc_macro]
pub fn make_is_empty(_item: TokenStream) -> TokenStream {
    expand_make_is_empty()
}

#[proc_macro]
pub fn make_get_all_items(_item: TokenStream) -> TokenStream {
    expand_make_get_all_items()
}

#[proc_macro]
pub fn make_get_all_map_items(_item: TokenStream) -> TokenStream {
    expand_make_get_all_map_items()
}

#[proc_macro]
pub fn make_array_to_vec(_item: TokenStream) -> TokenStream {
    expand_make_array_to_vec()
}

#[proc_macro]
pub fn make_map_to_vec(_item: TokenStream) -> TokenStream {
    expand_make_map_to_vec()
}

#[proc_macro]
pub fn make_vec_to_vec(_item: TokenStream) -> TokenStream {
    expand_make_vec_to_vec()
}
