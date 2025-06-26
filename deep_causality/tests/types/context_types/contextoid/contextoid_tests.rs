/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

#[test]
fn test_new() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    assert_eq!(d.id(), id);

    let contextoid: BaseContextoid = Contextoid::new(id, ContextoidType::Datoid(d));
    assert_eq!(contextoid.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    assert_eq!(d.id(), id);

    let contextoid: BaseContextoid = Contextoid::new(id, ContextoidType::Datoid(d));
    assert_eq!(contextoid.id(), id);
}

#[test]
fn test_to_string() {
    let id = 1;
    let data = 42;

    let d = Data::new(id, data);
    assert_eq!(d.id(), id);

    let contextoid: BaseContextoid = Contextoid::new(id, ContextoidType::Datoid(d));

    assert_eq!(contextoid.id(), id);

    let expected = "Contextoid ID: 1 Type: Datoid: Dataoid: id: 1 data: 42".to_string();
    let actual = contextoid.to_string();
    assert_eq!(actual, expected);
}
