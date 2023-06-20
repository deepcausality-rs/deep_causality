/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
// Procedural Macros https://doc.rust-lang.org/reference/procedural-macros.html

extern crate proc_macro;

use proc_macro::TokenStream;
use std::convert::TryFrom;

use litrs::StringLit;
use quote::quote;

#[proc_macro]
pub fn make_run(_input: TokenStream) -> TokenStream
{
    "if let Err(e) = run::run() {
        eprintln!(\"Error: {}\", e);
        process::exit(1);
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_len(_item: TokenStream) -> TokenStream
{
    "fn len(&self) -> usize {
        self.len()
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_is_empty(_item: TokenStream) -> TokenStream
{
    "fn is_empty(&self) -> bool {
        self.is_empty()
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_get_all_items(_item: TokenStream) -> TokenStream
{
    "fn get_all_items(&self) -> Vec<&T>{
        let mut all: Vec<&T> = Vec::new();
        for item in self {
            all.push(&item)
        }
        all
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_get_all_map_items(_item: TokenStream) -> TokenStream
{
    "fn get_all_items(&self) -> Vec<&V> {
        self.values().collect::<Vec<&V>>()
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_array_to_vec(_item: TokenStream) -> TokenStream
{
    "fn to_vec(&self) -> Vec<T> {
        self.to_vec()
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_map_to_vec(_item: TokenStream) -> TokenStream
{
    "fn to_vec(&self) -> Vec<V> {
        self.values().cloned().collect()
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_vec_to_vec(_item: TokenStream) -> TokenStream
{
    "fn to_vec(&self) -> Vec<T> {
        self.clone()
    }".parse().unwrap()
}

// How do I get the value and type of a Literal in a procedural macro?
// https://stackoverflow.com/questions/61169932/how-do-i-get-the-value-and-type-of-a-literal-in-a-procedural-macro
// https://crates.io/crates/litrs

#[proc_macro]
pub fn make_experiment(input: TokenStream) -> TokenStream
{
    // Parse input.
    let input = input.into_iter().collect::<Vec<_>>();

    // Verify input.
    if input.len() != 1 {
        let msg = format!("expected exactly one input str token, got {}", input.len());
        return quote! { compile_error!(#msg) }.into();
    }

    let string_lit = match StringLit::try_from(&input[0]) {
        // Error if the token is not a string literal
        Err(e) => return e.to_compile_error(),
        Ok(lit) => lit,
    };

    // Extract actual string literal value.
    let descr = string_lit.value();

    // Generate the actual constructor call and inject the string literal value.
    let gen = quote! {
            Experiment::new(
            id,
            stringify!(#descr),
            cm,
            exp::func,
        )
    };

    gen.into()
}
