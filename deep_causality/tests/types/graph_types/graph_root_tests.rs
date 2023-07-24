// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Identifiable, Root};

#[test]
fn test_new() {
    let r = Root::new(1);
    assert_eq!(1, r.id());
}

#[test]
fn test_id() {
    let r = Root::new(1);
    assert_eq!(1, r.id());
}

#[test]
fn test_to_string() {
    let r = Root::new(1);
    assert_eq!(1, r.id());

    let exp = format!("Root ID: {}", r.id());
    let act = r.to_string();

    assert_eq!(exp, act);
}