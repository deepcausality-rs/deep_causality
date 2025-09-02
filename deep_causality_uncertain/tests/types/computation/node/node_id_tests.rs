/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::NodeId;

use rusty_fork::rusty_fork_test;

rusty_fork_test! {

#[test]
fn test_new_node_id() {
    let initial_id = NodeId::new();
    let node_id1 = NodeId::from(0);

    assert_eq!(initial_id, node_id1);
}

#[test]
fn test_default_node_id() {
    let initial_id = NodeId::default();
    let node_id = NodeId::from(0);
    assert_eq!(initial_id, node_id);
}

#[test]
fn test_node_id_debug_trait() {
    let node_id = NodeId::new();
    let debug_string = format!("{:?}", node_id);
    assert!(debug_string.contains(&node_id.to_string()));
    assert!(debug_string.starts_with("NodeId("));
    assert!(debug_string.ends_with(")"));
}

#[test]
fn test_node_id_clone_trait() {
    let node_id1 = NodeId::new();
    let node_id2 = node_id1;
    assert_eq!(node_id1, node_id2);
}

#[test]
fn test_node_id_copy_trait() {
    let node_id1 = NodeId::new();
    let node_id2 = node_id1; // This is a copy, not a move
    assert_eq!(node_id1, node_id2);
}

}
