/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::{
    BaseContext, BaseContextoid, Context, Contextoid, ContextoidType, ExtendableContextuableGraph,
    RelationKind, Root,
};

// Helper to create a default context for tests.
fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

// Helper to create a test contextoid.
fn get_contextoid(id: u64) -> BaseContextoid {
    Contextoid::new(id, ContextoidType::Root(Root::new(id)))
}

// =================================================================================
// Context Management Tests
// =================================================================================

#[test]
fn test_extra_ctx_add_new() {
    let mut context = get_context();
    assert_eq!(context.extra_ctx_get_current_id(), 0);

    // Add a new context, but don't set it as default
    let new_id_1 = context.extra_ctx_add_new(10, false);
    assert_eq!(new_id_1, 1);
    assert!(context.extra_ctx_check_exists(1));
    // Current ID should not have changed
    assert_eq!(context.extra_ctx_get_current_id(), 0);

    // Add another context and set it as default
    let new_id_2 = context.extra_ctx_add_new(10, true);
    assert_eq!(new_id_2, 2);
    assert!(context.extra_ctx_check_exists(2));
    // Current ID should now be the new one
    assert_eq!(context.extra_ctx_get_current_id(), 2);
}

#[test]
fn test_extra_ctx_add_new_with_id() {
    let mut context = get_context();
    let id = 99;
    let capacity = 10;

    // Add with default=false
    let res = context.extra_ctx_add_new_with_id(id, capacity, false);
    assert!(res.is_ok());
    assert!(context.extra_ctx_check_exists(id));
    assert_eq!(
        context.extra_ctx_get_current_id(),
        0,
        "Current ID should not change when default is false"
    );

    // Add with default=true
    let id2 = 100;
    let res2 = context.extra_ctx_add_new_with_id(id2, capacity, true);
    assert!(res2.is_ok());
    assert!(context.extra_ctx_check_exists(id2));
    assert_eq!(
        context.extra_ctx_get_current_id(),
        id2,
        "Current ID should be set when default is true"
    );
}

#[test]
fn test_extra_ctx_add_new_with_id_err() {
    let mut context = get_context();
    let id = 1;
    let capacity = 10;
    let default = true;
    let res = context.extra_ctx_add_new_with_id(id, capacity, default);
    assert!(res.is_ok());

    // Attempt to add again with the same ID
    let res = context.extra_ctx_add_new_with_id(id, capacity, default);
    assert!(
        res.is_err(),
        "Should fail when adding a context with a duplicate ID"
    );
}

#[test]
fn test_extra_ctx_check_exists() {
    let mut context = get_context();
    assert!(
        !context.extra_ctx_check_exists(1),
        "Should not exist before creation"
    );
    context.extra_ctx_add_new(10, false);
    assert!(
        context.extra_ctx_check_exists(1),
        "Should exist after creation"
    );
    assert!(
        !context.extra_ctx_check_exists(99),
        "Non-existent ID should return false"
    );
}

#[test]
fn test_extra_ctx_set_and_get_current_id() {
    let mut context = get_context();
    let id = 42;
    context.extra_ctx_add_new_with_id(id, 10, false).unwrap();

    // Set current ID
    let res = context.extra_ctx_set_current_id(id);
    assert!(res.is_ok());
    assert_eq!(context.extra_ctx_get_current_id(), id);

    // Try to set a non-existent ID
    let res_err = context.extra_ctx_set_current_id(99);
    assert!(res_err.is_err());
}

#[test]
fn test_extra_ctx_unset_current_id() {
    let mut context = get_context();
    let id = 42;
    context.extra_ctx_add_new_with_id(id, 10, true).unwrap();
    assert_eq!(context.extra_ctx_get_current_id(), id);

    // Unset the current ID
    let res = context.extra_ctx_unset_current_id();
    assert!(res.is_ok());
    assert_eq!(context.extra_ctx_get_current_id(), 0);
}

#[test]
fn test_extra_ctx_unset_current_id_err() {
    let mut context = get_context();
    // Should fail because no context is set (current_id is 0)
    let res = context.extra_ctx_unset_current_id();
    assert!(res.is_err());
}

// =================================================================================
// Node Operation Tests
// =================================================================================

#[test]
fn test_extra_ctx_node_ops_happy_path() {
    let mut context = get_context();
    let ctx_id = 1;
    context.extra_ctx_add_new_with_id(ctx_id, 10, true).unwrap();

    // Add node
    let contextoid = get_contextoid(101);
    let node_idx_res = context.extra_ctx_add_node(contextoid.clone());
    assert!(node_idx_res.is_ok());
    let node_idx = node_idx_res.unwrap();

    // Contains node
    assert!(context.extra_ctx_contains_node(node_idx));
    assert!(!context.extra_ctx_contains_node(999));

    // Get node
    let fetched_node_res = context.extra_ctx_get_node(node_idx);
    assert!(fetched_node_res.is_ok());
    assert_eq!(*fetched_node_res.unwrap(), contextoid);

    // Remove node
    let remove_res = context.extra_ctx_remove_node(node_idx);
    assert!(remove_res.is_ok());
    assert!(!context.extra_ctx_contains_node(node_idx));
}

#[test]
fn test_extra_ctx_add_node_err() {
    let mut context = get_context();
    let contextoid = get_contextoid(1);

    // Error: No extra contexts exist at all
    let res = context.extra_ctx_add_node(contextoid.clone());
    assert!(res.is_err());

    // Error: Extra contexts exist, but the current_id is invalid (0)
    context.extra_ctx_add_new(10, false);
    let res2 = context.extra_ctx_add_node(contextoid);
    assert!(res2.is_err());
}

#[test]
fn test_extra_ctx_get_node_err() {
    let mut context = get_context();

    // Error: No extra contexts exist
    assert!(context.extra_ctx_get_node(0).is_err());

    context.extra_ctx_add_new_with_id(1, 10, true).unwrap();
    // Error: Extra context exists, but node index is invalid
    assert!(context.extra_ctx_get_node(0).is_err());
}

#[test]
fn test_extra_ctx_remove_node_err() {
    let mut context = get_context();

    // Error: No extra contexts exist
    assert!(context.extra_ctx_remove_node(0).is_err());

    context.extra_ctx_add_new_with_id(1, 10, true).unwrap();
    // Error: Extra context exists, but node index is invalid
    assert!(context.extra_ctx_remove_node(0).is_err());
}

// =================================================================================
// Edge Operation Tests
// =================================================================================

#[test]
fn test_extra_ctx_edge_ops_happy_path() {
    let mut context = get_context();
    context.extra_ctx_add_new_with_id(1, 10, true).unwrap();

    let node_a_idx = context.extra_ctx_add_node(get_contextoid(1)).unwrap();
    let node_b_idx = context.extra_ctx_add_node(get_contextoid(2)).unwrap();

    // Add edge
    let add_res = context.extra_ctx_add_edge(node_a_idx, node_b_idx, RelationKind::Datial);
    assert!(add_res.is_ok());

    // Contains edge
    assert!(context.extra_ctx_contains_edge(node_a_idx, node_b_idx));
    assert!(!context.extra_ctx_contains_edge(node_b_idx, node_a_idx)); // Directed graph

    // Remove edge
    let remove_res = context.extra_ctx_remove_edge(node_a_idx, node_b_idx);
    assert!(remove_res.is_ok());
    assert!(!context.extra_ctx_contains_edge(node_a_idx, node_b_idx));
}

#[test]
fn test_extra_ctx_add_edge_err() {
    let mut context = get_context();

    // Error: No extra contexts exist
    let res = context.extra_ctx_add_edge(0, 1, RelationKind::Datial);
    assert!(res.is_err());

    context.extra_ctx_add_new_with_id(1, 10, true).unwrap();
    // Error: Extra context exists, but node indices are invalid
    let res2 = context.extra_ctx_add_edge(0, 1, RelationKind::Datial);
    assert!(res2.is_err());
}

// =================================================================================
// Graph Property Tests
// =================================================================================

#[test]
fn test_extra_ctx_graph_properties() {
    let mut context = get_context();
    context.extra_ctx_add_new_with_id(1, 10, true).unwrap();

    // Check properties on empty graph
    assert!(context.extra_ctx_is_empty().unwrap());
    assert_eq!(context.extra_ctx_size().unwrap(), 0);
    assert_eq!(context.extra_ctx_node_count().unwrap(), 0);
    assert_eq!(context.extra_ctx_edge_count().unwrap(), 0);

    // Add nodes and edges
    let node_a = context.extra_ctx_add_node(get_contextoid(1)).unwrap();
    let node_b = context.extra_ctx_add_node(get_contextoid(2)).unwrap();
    context
        .extra_ctx_add_edge(node_a, node_b, RelationKind::Datial)
        .unwrap();

    // Check properties on non-empty graph
    assert!(!context.extra_ctx_is_empty().unwrap());
    assert_eq!(context.extra_ctx_size().unwrap(), 2); // 2 nodes 
    assert_eq!(context.extra_ctx_node_count().unwrap(), 2);
    assert_eq!(context.extra_ctx_edge_count().unwrap(), 1);
}

#[test]
fn test_extra_ctx_graph_properties_err() {
    let context = get_context();

    // All property methods should fail if no extra context is active
    assert!(context.extra_ctx_is_empty().is_err());
    assert!(context.extra_ctx_size().is_err());
    assert!(context.extra_ctx_node_count().is_err());
    assert!(context.extra_ctx_edge_count().is_err());
}
