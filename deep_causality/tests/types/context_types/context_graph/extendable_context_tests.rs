/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils::get_context;
use deep_causality::*;

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
fn test_extra_ctx_set_default_err() {
    let mut context = get_context();

    // Add an extra context with a non-zero ID (e.g., 1).
    // Crucially, pass `default: false` so that `context.extra_context_id` remains `0`.
    context.extra_ctx_add_new_with_id(1, 10, false).unwrap();

    // Current state:
    // - `context.extra_contexts` is `Some({1: UltraGraph})`
    // - `context.extra_context_id` is `0`

    // The call to `extra_contexts.get_mut(&0)` will now correctly return `None`,
    // because the map does not contain the key `0`. This will trigger the
    // inner `else` branch as intended.
    // The node indices `0` and `1` are arbitrary as the function fails before using them.
    let result = context.extra_ctx_remove_edge(0, 1);

    // ASSERT: Verify that we received the expected error from the correct branch.
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.to_string(),
        "ContextIndexError: Cannot remove edge. Current extra context with ID 0 not found."
    );

    // Attempt to get the size. The check for `extra_contexts` will fail.
    let result = context.extra_ctx_size();

    // ASSERT: Verify the correct error is returned.
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.to_string(),
        "ContextIndexError: Cannot get size. Current extra context with ID 0 not found."
    );

    // ACTION: Attempt to check if empty. The check for `extra_contexts` will fail.
    let result = context.extra_ctx_is_empty();

    // ASSERT: Verify the correct error is returned.
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.to_string(),
        "ContextIndexError: Cannot check if empty. Current extra context with ID 0 not found."
    );

    let result = context.extra_ctx_node_count();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.to_string(),
        "ContextIndexError: Cannot get node count. Current extra context with ID 0 not found."
    );

    let result = context.extra_ctx_edge_count();

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.to_string(),
        "ContextIndexError: Cannot get edge count. Current extra context with ID 0 not found."
    );
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

    // Call the function immediately. The first `if let` will fail.
    // The node index `0` is arbitrary as the function returns before checking it.
    let result = context.extra_ctx_contains_node(0);

    // Assert that the function correctly returns false.
    assert!(!result,);

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
