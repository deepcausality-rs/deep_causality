/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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

// VecDeque can't be turned into a vector hence the custom implementation
// https://github.com/rust-lang/rust/issues/23308
// Also, make_contiguous requires self to be mutable, which would violate the API, hence the clone.
// https://doc.rust-lang.org/std/collections/struct.VecDeque.html#method.make_contiguous
pub fn expand_make_vec_deq_to_vec() -> TokenStream {
    "fn to_vec(&self) -> Vec<T>
    {
        let mut v = Vec::with_capacity(self.len());
        let mut deque = self.clone(); // clone to avoid mutating the original

        for item in deque.make_contiguous().iter() {
            v.push(item.clone());
        }

        v
    }"
    .parse()
    .unwrap()
}

pub fn expand_find_from_map() -> TokenStream {
    " fn get_item_by_id(&self, id: IdentificationValue) -> Option<&V>
    {
        self.values().find(|item| item.id() == id)
    }"
    .parse()
    .unwrap()
}

pub fn expand_find_from_iter() -> TokenStream {
    " fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T>
    {
        self.iter().find(|item| item.id() == id)
    }"
    .parse()
    .unwrap()
}
