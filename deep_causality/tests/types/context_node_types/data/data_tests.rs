/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Data, Datable, Identifiable};

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

    let exp = format!("Dataoid: id: {id} data: {data}");
    let act = d.to_string();
    assert_eq!(act, exp);
}
