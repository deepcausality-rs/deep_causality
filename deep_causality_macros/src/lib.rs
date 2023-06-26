/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

// Procedural Macros https://doc.rust-lang.org/reference/procedural-macros.html

extern crate proc_macro;

use proc_macro::TokenStream;

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

// macros used in deep_causality/src/types/window_type

#[proc_macro]
pub fn make_first(_item: TokenStream) -> TokenStream
{
    "#[inline(always)]
    fn first(&self) -> Result<T, String> {
        if self.tail != 0 {
            Ok(self.store[self.head])
        } else {
            Err(\"Array is empty. Add some elements to the array first\".to_string())
        }
    }
    ".parse().unwrap()
}

#[proc_macro]
pub fn make_last(_item: TokenStream) -> TokenStream
{
    "#[inline(always)]
    fn last(&self) -> Result<T, String> {
        if self.filled() {
            Ok(self.store[self.tail - 1])
        } else {
            Err(\"Array is not yet filled. Add some elements to the array first\".to_string())
        }
    }
    ".parse().unwrap()
}

#[proc_macro]
pub fn make_tail(_item: TokenStream) -> TokenStream
{
    "#[inline(always)]
    fn tail(&self) -> usize {
        self.tail
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_size(_item: TokenStream) -> TokenStream
{
    "#[inline(always)]
    fn size(&self) -> usize {
        self.size
    }".parse().unwrap()
}

#[proc_macro]
pub fn make_get_slice(_item: TokenStream) -> TokenStream
{
    "#[inline(always)]
    fn get_slice(&self) -> &[T] {
        if self.tail > self.size
        {
            // Adjust offset in case the window is larger than the slice.
            &self.store[self.head + 1..self.tail]
        } else {
            &self.store[self.head..self.tail]
        }
    }".parse().unwrap()
}
