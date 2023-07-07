// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

// Procedural Macros https://doc.rust-lang.org/reference/procedural-macros.html

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro]
pub fn make_run(_input: TokenStream) -> TokenStream
{
    "time(run::run, \"main_run\");

    fn time<T, F: FnOnce() -> T>(f: F, f_name: &str) -> T {
        let start = std::time::Instant::now();
        let res = f();
        println!(\"{} Execution took {:?} \",
            f_name.to_uppercase(),
            start.elapsed()
        );
        res
    }
    ".parse().unwrap()
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
