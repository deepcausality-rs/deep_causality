// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Data, Datable, Identifiable};

#[test]
fn test_new() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);

    assert_eq!(d.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    assert_eq!(d.id(), id);
}

#[test]
fn test_data() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    assert_eq!(d.id(), id);
    assert_eq!(d.get_data(), data);
}

#[test]
fn test_to_string() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    assert_eq!(d.id(), id);
    assert_eq!(d.id(), id);
    assert_eq!(d.get_data(), data);

    let exp = format!("Dataoid: id: {} data: {}", id, data);
    let act = d.to_string();
    assert_eq!(act, exp);
}
