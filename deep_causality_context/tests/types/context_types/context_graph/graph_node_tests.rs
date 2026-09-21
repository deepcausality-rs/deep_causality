/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{BaseContextoid, Contextoid, ContextoidType, Identifiable, Root};

fn get_test_contextoid() -> BaseContextoid {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));
    node
}

#[test]
fn test_new() {
    let id = 1;
    let node = get_test_contextoid();
    assert_eq!(node.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let node = get_test_contextoid();
    assert_eq!(node.id(), id);
}

#[test]
fn test_vertex_type() {
    let node = get_test_contextoid();

    assert!(node.vertex_type().root().is_some());
    assert!(node.vertex_type().dataoid().is_none());
    assert!(node.vertex_type().tempoid().is_none());
    assert!(node.vertex_type().spaceoid().is_none());
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_to_string() {
    let node = get_test_contextoid();

    let expected = "Contextoid ID: 1 Type: Root: Root ID: 1".to_string();
    let actual = node.to_string();
    assert_eq!(actual, expected);
}
