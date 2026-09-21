/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::utils_test::test_utils::{
    get_base_context, get_context, get_test_context,
};
use deep_causality_context::{ContextKind, ContextuableGraph, Identifiable};

// These three builders are fixtures other suites assert against, so a fixture that returns the
// wrong shape weakens every test that takes it and reports nothing itself. Each test below pins
// the property a caller relies on: the identity, the name, and what is in the graph.

#[test]
fn test_get_context_is_empty() {
    let context = get_context();

    assert_eq!(context.id(), 1);
    assert_eq!(context.name(), "base context");
    assert_eq!(context.size(), 0);
    assert!(context.is_empty());
    assert_eq!(context.number_of_nodes(), 0);
    assert_eq!(context.number_of_edges(), 0);
    // Nothing was added, so no id resolves to an index.
    assert_eq!(context.get_node_index_by_id(1), None);
}

#[test]
fn test_get_base_context_holds_one_root_at_index_zero() {
    let context = get_base_context();

    assert_eq!(context.id(), 1);
    assert_eq!(context.name(), "base context");
    assert_eq!(context.size(), 1);
    assert!(!context.is_empty());

    // The node is a root carrying id 1, sitting at index 0. A fixture that added a datoid, or
    // added the node under another id, satisfies the counts above and fails here.
    assert_eq!(context.get_node_index_by_id(1), Some(0));
    let node = context.get_node(0).expect("the root node is at index 0");
    assert_eq!(node.id(), 1);
    assert_eq!(node.vertex_type().kind(), ContextKind::Root);
    assert_eq!(node.vertex_type().root().expect("a root").id(), 1);
}

#[test]
fn test_get_test_context_carries_its_own_name() {
    let context = get_test_context();

    // The only thing separating this fixture from `get_base_context` is the name, so that is
    // what a caller choosing between them depends on.
    assert_eq!(context.name(), "Test-Context");
    assert_ne!(context.name(), get_base_context().name());

    assert_eq!(context.id(), 1);
    assert_eq!(context.size(), 1);
    assert_eq!(context.get_node_index_by_id(1), Some(0));
    let node = context.get_node(0).expect("the root node is at index 0");
    assert_eq!(node.vertex_type().kind(), ContextKind::Root);
}

#[test]
fn test_each_call_returns_an_independent_context() {
    // The builders return owned contexts, so writing to one must not reach the next. A fixture
    // backed by shared state would show the second node in both.
    let mut first = get_base_context();
    let second = get_base_context();

    // `remove_node` takes the contextoid id, not the graph index; the fixture's root carries 1.
    first.remove_node(1).expect("failed to remove the root");

    assert_eq!(first.size(), 0);
    assert_eq!(second.size(), 1);
}
