// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use proc_macro::TokenStream;

pub fn expand_make_len() -> TokenStream {
    "fn len(&self) -> usize
    {
        self.len()
    }"
    .parse()
    .unwrap()
}

pub fn expand_make_is_empty() -> TokenStream {
    "fn is_empty(&self) -> bool{
        self.is_empty()
    }"
    .parse()
    .unwrap()
}

pub fn expand_make_get_all_items() -> TokenStream {
    "fn get_all_items(&self) -> Vec<&T>
    {
        let mut all: Vec<&T> = Vec::new();
        for item in self {
            all.push(&item)
        }
        all
    }"
    .parse()
    .unwrap()
}

pub fn expand_make_get_all_map_items() -> TokenStream {
    "fn get_all_items(&self) -> Vec<&V>
    {
        self.values().collect::<Vec<&V>>()
    }"
    .parse()
    .unwrap()
}

pub fn expand_make_array_to_vec() -> TokenStream {
    "fn to_vec(&self) -> Vec<T>
    {
        self.to_vec()
    }"
    .parse()
    .unwrap()
}

pub fn expand_make_map_to_vec() -> TokenStream {
    "fn to_vec(&self) -> Vec<V>
    {
        self.values().cloned().collect()
    }"
    .parse()
    .unwrap()
}

pub fn expand_make_vec_to_vec() -> TokenStream {
    "fn to_vec(&self) -> Vec<T>
    {
        self.clone()
    }"
    .parse()
    .unwrap()
}
