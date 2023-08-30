// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::{Constructor, Getters};

// Generate a default constructor and getters for all fields.
#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Data<T>
where
    T: Copy + Default,
{
    #[getter(name = data_id)] // Rename getter methods as you wish
    id: u64,
    data: T,
    filled: bool,
}

#[test]
fn test_derive_struct() {
    let d = Data::new(0, 42, true);

    assert_eq!(*d.data_id(), 0);
    assert_eq!(*d.data(), 42);
    assert!(*d.filled());
}

#[derive(Getters, Constructor)]
struct WebRef<'a> {
    name: &'a str,
    url: &'a str,
    category: Option<&'a str>,
}

#[test]
fn test_ref() {
    let page = WebRef::new("GitHub", "https://github.com/", None);

    assert_eq!(*page.name(), "GitHub");
    assert!(!page.url().is_empty());
    assert!(page.category().is_none());

    let page = WebRef::new(
        "Hacker News",
        "https://news.ycombinator.com//",
        Some("News"),
    );
    assert_eq!(*page.name(), "Hacker News");
    assert!(!page.url().is_empty());
    assert!(page.category().is_some());
}

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum Scale {
    Small,
    Big,
}

#[test]
fn test_derive_enum() {
    let big = Scale::new_big();
    assert_eq!(big, Scale::Big);

    let small = Scale::new_small();
    assert_eq!(small, Scale::Small)
}

#[derive(Constructor)]
pub struct Unnamed(i32);

#[test]
fn test_unnamed_fields() {
    let a = Unnamed::new(98);
    assert_eq!(a.0, 98);
}
