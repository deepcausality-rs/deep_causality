/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::*;

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    let capacity = 10; // adjust as needed
    Context::with_capacity(id, name, capacity)
}

#[test]
fn test_new() {
    let id = 1;
    let context = get_context();
    assert_eq!(context.id(), id);
}

#[test]
fn test_extra_ctx_add_new() {
    let id = 1;

    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let res = context.extra_ctx_add_new(capacity, default);
    assert_eq!(res, 1);
}

#[test]
fn test_extra_ctx_check_exists() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);
}

#[test]
fn test_extra_ctx_get_current_id() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);
}

#[test]
fn test_extra_ctx_set_current_id() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let first_ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(first_ctx_id, 1);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, first_ctx_id);

    let second_ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(second_ctx_id, 2);

    // When default is set to true, the current extra context is set to the second context
    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, second_ctx_id);

    let capacity = 10;
    let default = false;
    let third_ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(third_ctx_id, 3);

    // When default is set to false, the current extra context remains at its previous value,
    // i.e. the second context.
    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, second_ctx_id);

    // Set current extra context to the third context
    let res = context.extra_ctx_set_current_id(third_ctx_id);
    assert!(res.is_ok());

    // The current extra context is now the third context
    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, third_ctx_id);
}

#[test]
fn test_extra_ctx_unset_current_id() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 10;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let res = context.extra_ctx_unset_current_id();
    assert!(res.is_ok());

    // Zero is the default value for the extra context if nothing else is set
    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, 0);
}

#[test]
fn test_extra_ctx_set_current_id_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    // Try to set the current context to an invalid context id
    let random_ctx_id = 42;
    let res = context.extra_ctx_set_current_id(random_ctx_id);
    assert!(res.is_err());
}

#[test]
fn test_extra_ctx_add_node() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(contextoid);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 0);
}

#[test]
fn test_extra_ctx_contains_node() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(contextoid);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 0);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);
}

#[test]
fn test_extra_ctx_contains_node_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let first_ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(first_ctx_id, 1);

    let random_node_id = 42;
    let exists = context.extra_ctx_contains_node(random_node_id);
    assert!(!exists);
}

#[test]
fn test_extra_ctx_get_node() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(contextoid);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 0);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let node = context.extra_ctx_get_node(node_id);
    assert!(node.is_ok());

    let contextoid = node.unwrap();
    assert_eq!(contextoid.id(), 1);
}

#[test]
fn test_extra_ctx_get_node_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let random_node_id = 42;
    let node = context.extra_ctx_get_node(random_node_id);
    assert!(node.is_err());
}

#[test]
fn test_extra_ctx_remove_node() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(contextoid);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 0);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let res = context.extra_ctx_remove_node(node_id);
    assert!(res.is_ok());

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(!exists);
}

#[test]
fn test_extra_ctx_remove_node_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let random_node_id = 42;
    let node = context.extra_ctx_remove_node(random_node_id);
    assert!(node.is_err());
}

#[test]
fn test_extra_ctx_add_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(c_1);
    assert!(res.is_ok());

    let root_id = res.unwrap();
    assert_eq!(root_id, 0);

    let exists = context.extra_ctx_contains_node(root_id);
    assert!(exists);

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 1);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let res = context.extra_ctx_add_edge(root_id, node_id, RelationKind::Temporal);
    assert!(res.is_ok());
}

#[test]
fn test_extra_ctx_add_edge_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    // Both nodes do not exist
    let no_id_a = 23;
    let no_id_b = 42;
    let res = context.extra_ctx_add_edge(no_id_a, no_id_b, RelationKind::Temporal);
    assert!(res.is_err());

    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(c_1);
    assert!(res.is_ok());

    let root_id = res.unwrap();
    assert_eq!(root_id, 0);

    let exists = context.extra_ctx_contains_node(root_id);
    assert!(exists);

    // Second node does not exist
    let no_id_b = 42;
    let res = context.extra_ctx_add_edge(root_id, no_id_b, RelationKind::Temporal);
    assert!(res.is_err());

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 1);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    // First nodes do not exist
    let no_id_a = 23;
    let res = context.extra_ctx_add_edge(no_id_a, node_id, RelationKind::Temporal);
    assert!(res.is_err());
}

#[test]
fn test_extra_ctx_contains_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(c_1);
    assert!(res.is_ok());

    let root_id = res.unwrap();
    assert_eq!(root_id, 0);

    let exists = context.extra_ctx_contains_node(root_id);
    assert!(exists);

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 1);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let res = context.extra_ctx_add_edge(root_id, node_id, RelationKind::Temporal);
    assert!(res.is_ok());

    let res = context.extra_ctx_contains_edge(root_id, node_id);
    assert!(res);
}

#[test]
fn test_extra_ctx_contains_edge_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(c_1);
    assert!(res.is_ok());

    let root_id = res.unwrap();
    assert_eq!(root_id, 0);

    let exists = context.extra_ctx_contains_node(root_id);
    assert!(exists);

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 1);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let res = context.extra_ctx_add_edge(root_id, node_id, RelationKind::Temporal);
    assert!(res.is_ok());

    let res = context.extra_ctx_contains_edge(root_id, node_id);
    assert!(res);

    let false_id = 99;
    let res = context.extra_ctx_contains_edge(false_id, node_id);
    assert!(!res);

    let res = context.extra_ctx_contains_edge(root_id, false_id);
    assert!(!res);

    let no_id_a = 23;
    let no_id_b = 42;

    // First node does not exist
    let exists = context.extra_ctx_contains_edge(no_id_a, node_id);
    assert!(!exists);

    // Second node does not exist
    let exists = context.extra_ctx_contains_edge(root_id, no_id_b);
    assert!(!exists);

    // Both nodes do not exist
    let exists = context.extra_ctx_contains_edge(no_id_a, no_id_b);
    assert!(!exists);
}

#[test]
fn test_extra_ctx_contains_edge_when_no_extra_contexts_exist() {
    // This test hits the outer `else` branch.
    let context = get_context();
    // With no extra_contexts map, this should be false.
    assert!(!context.extra_ctx_contains_edge(0, 1));
}

#[test]
fn test_extra_ctx_contains_edge_with_invalid_current_id() {
    // This test hits the inner `else` branch.
    let mut context = get_context();
    // Create an extra context but do NOT set it as the current one.
    context.extra_ctx_add_new_with_id(1, 10, false).unwrap();

    // The current_id is still 0, which is not a valid key in the map.
    assert!(!context.extra_ctx_contains_edge(0, 1));
}

#[test]
fn test_extra_ctx_contains_edge_happy_path_and_no_edge() {
    // This test hits the main logic path.
    let mut context = get_context();
    context.extra_ctx_add_new_with_id(1, 10, true).unwrap();

    let id = 1;
    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let node_a = context.extra_ctx_add_node(c_1).unwrap();

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 42;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let node_b = context.extra_ctx_add_node(c_2).unwrap();

    // Check before adding the edge
    assert!(!context.extra_ctx_contains_edge(node_a, node_b));

    // Add the edge and check again
    context
        .extra_ctx_add_edge(node_a, node_b, RelationKind::Datial)
        .unwrap();
    assert!(context.extra_ctx_contains_edge(node_a, node_b));

    // Check for non-existent edge
    assert!(!context.extra_ctx_contains_edge(node_b, node_a)); // Directed
    assert!(!context.extra_ctx_contains_edge(node_a, 999)); // Non-existent node
}

#[test]
fn test_extra_ctx_remove_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 10;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(c_1);
    assert!(res.is_ok());

    let root_id = res.unwrap();
    assert_eq!(root_id, 0);

    let exists = context.extra_ctx_contains_node(root_id);
    assert!(exists);

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 42;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 1);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let res = context.extra_ctx_node_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 2);

    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 0);

    let res = context.extra_ctx_add_edge(root_id, node_id, RelationKind::Temporal);
    assert!(res.is_ok());

    let exists = context.extra_ctx_contains_edge(root_id, node_id);
    assert!(exists);

    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 1);

    let res = context.extra_ctx_remove_edge(0, 1);
    assert!(res.is_ok());

    let exists = context.extra_ctx_contains_edge(root_id, node_id);
    assert!(!exists);

    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 0);
}

#[test]
fn test_extra_ctx_remove_edge_err() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 10;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let c_1 = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.extra_ctx_add_node(c_1);
    assert!(res.is_ok());

    let root_id = res.unwrap();
    assert_eq!(root_id, 0);

    let exists = context.extra_ctx_contains_node(root_id);
    assert!(exists);

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 42;
    let c_2 = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let res = context.extra_ctx_add_node(c_2);
    assert!(res.is_ok());

    let node_id = res.unwrap();
    assert_eq!(node_id, 1);

    let exists = context.extra_ctx_contains_node(node_id);
    assert!(exists);

    let res = context.extra_ctx_node_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 2);

    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 0);

    let res = context.extra_ctx_add_edge(root_id, node_id, RelationKind::Temporal);
    assert!(res.is_ok());

    let exists = context.extra_ctx_contains_edge(root_id, node_id);
    assert!(exists);

    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 1);

    let no_id_a = 23;
    let no_id_b = 42;

    // First node does not exist
    let res = context.extra_ctx_remove_edge(no_id_a, node_id);
    assert!(res.is_err());

    // Second node does not exist
    let res = context.extra_ctx_remove_edge(root_id, no_id_b);
    assert!(res.is_err());

    // Both nodes do not exist
    let res = context.extra_ctx_remove_edge(no_id_a, no_id_b);
    assert!(res.is_err());

    // Doubled check if the edge is still there i.e all delete attempts failed
    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 1);
}

#[test]
fn test_extra_ctx_size() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let res = context.extra_ctx_size();
    assert!(res.is_ok());

    let size = res.unwrap();
    assert_eq!(size, 0);
}

#[test]
fn test_extra_ctx_is_empty() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let res = context.extra_ctx_size();
    assert!(res.is_ok());

    let size = res.unwrap();
    assert_eq!(size, 0);

    let res = context.extra_ctx_is_empty();
    let is_empty = res.unwrap();

    assert!(is_empty);
}

#[test]
fn test_extra_ctx_node_count() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let res = context.extra_ctx_size();
    assert!(res.is_ok());

    let size = res.unwrap();
    assert_eq!(size, 0);

    let res = context.extra_ctx_node_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 0);
}

#[test]
fn test_extra_ctx_edge_count() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.id(), id);

    let capacity = 100;
    let default = true;

    let ctx_id = context.extra_ctx_add_new(capacity, default);
    assert_eq!(ctx_id, 1);

    let exists = context.extra_ctx_check_exists(ctx_id);
    assert!(exists);

    let current_id = context.extra_ctx_get_current_id();
    assert_eq!(current_id, ctx_id);

    let res = context.extra_ctx_size();
    assert!(res.is_ok());

    let size = res.unwrap();
    assert_eq!(size, 0);

    let res = context.extra_ctx_edge_count();
    let node_count = res.unwrap();
    assert_eq!(node_count, 0);
}
