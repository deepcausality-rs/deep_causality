/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, EuclideanTime,
    Identifiable, RelationKind, Root, TimeScale,
};

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

#[test]
fn test_new() {
    let id = 1;
    let context = get_context();
    assert_eq!(context.id(), id);
}

#[test]
fn test_id() {
    let id = 1;

    let context = get_context();
    assert_eq!(context.id(), id);
}

#[test]
fn test_name() {
    let id = 1;
    let name = "base context".to_string();

    let context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), &name);
}

#[test]
fn test_node_count() {
    let id = 1;
    let name = "base context".to_string();

    let context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);
    let node_count = 0;
    assert_eq!(context.number_of_nodes(), node_count);
}

#[test]
fn test_edge_count() {
    let id = 1;
    let name = "base context".to_string();

    let context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);

    let node_count = 0;
    assert_eq!(context.number_of_nodes(), node_count);
    let edge_count = 0;
    assert_eq!(context.number_of_edges(), edge_count);
}

#[test]
fn test_add_node() {
    let id = 1;
    let name = "base context".to_string();

    let mut context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);
    assert_eq!(context.size(), 0);

    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let idx = context.add_node(contextoid).expect("Failed to add node");

    assert_eq!(idx, 0);
    assert_eq!(context.size(), 1);
}

#[test]
fn test_contains_node() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    context.add_node(contextoid).expect("Failed to add node");

    let idx: usize = 0;

    assert_eq!(context.size(), 1);
    assert!(context.contains_node(idx))
}

#[test]
fn test_get_node() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    context.add_node(contextoid).expect("Failed to add node");
    assert_eq!(context.size(), 1);

    let idx: usize = 0;
    assert!(context.contains_node(idx));

    let contextoid = context.get_node(idx);
    assert!(contextoid.is_some());
}

#[test]
fn test_remove_node() {
    let mut context = get_context();
    assert!(context.is_empty());

    let node_id_to_remove = 1;
    let root = Root::new(node_id_to_remove);
    let contextoid = Contextoid::new(node_id_to_remove, ContextoidType::Root(root));

    // Add the node and verify it's there
    let physical_index = context.add_node(contextoid).expect("Failed to add node");
    assert_eq!(context.number_of_nodes(), 1);
    assert!(context.contains_node(physical_index));

    // Remove the node using its LOGICAL ID, not its physical index
    let result = context.remove_node(node_id_to_remove);
    assert!(result.is_ok(), "Failed to remove node: {:?}", result.err());

    // Verify the node is gone
    assert_eq!(context.number_of_nodes(), 0);
    assert!(!context.contains_node(physical_index));
}

#[test]
fn test_add_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let roodidx = context.add_node(contextoid).expect("Failed to add node");

    assert_eq!(context.size(), 1);
    assert!(context.contains_node(roodidx));

    let contextoid = context.get_node(roodidx);
    assert!(contextoid.is_some());

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12.0f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let contextoid = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let t_idx = context.add_node(contextoid).expect("Failed to add node");

    let res = context.add_edge(roodidx, t_idx, RelationKind::Temporal);
    assert!(res.is_ok());

    assert!(context.contains_edge(roodidx, t_idx));
}

#[test]
fn test_contains_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let roodidx = context.add_node(contextoid).expect("Failed to add node");

    assert_eq!(context.size(), 1);
    assert!(context.contains_node(roodidx));

    let contextoid = context.get_node(roodidx);
    assert!(contextoid.is_some());

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12.0f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let contextoid = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let t_idx = context.add_node(contextoid).expect("Failed to add node");
    let res = context.add_edge(roodidx, t_idx, RelationKind::Temporal);
    assert!(res.is_ok());

    assert!(context.contains_edge(roodidx, t_idx));
}

#[test]
fn test_remove_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid = Contextoid::new(id, ContextoidType::Root(root));
    let roodidx = context.add_node(contextoid).expect("Failed to add node");

    assert_eq!(context.size(), 1);
    assert!(context.contains_node(roodidx));

    let contextoid = context.get_node(roodidx);
    assert!(contextoid.is_some());

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12.0f64;
    let tempoid = EuclideanTime::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let contextoid = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let t_idx = context.add_node(contextoid).expect("Failed to add node");
    let res = context.add_edge(roodidx, t_idx, RelationKind::Temporal);
    assert!(res.is_ok());

    assert!(context.contains_edge(roodidx, t_idx));

    let res = context.remove_edge(roodidx, t_idx);
    assert!(res.is_ok());

    assert!(!context.contains_edge(roodidx, t_idx));
}

#[test]
fn size() {
    let context = get_context();
    assert_eq!(context.size(), 0);
}

#[test]
fn is_empty() {
    let context = get_context();
    assert!(context.is_empty());
}

#[test]
fn test_to_string() {
    let context = get_context();

    let exp = "Context: id: 1, name: base context, node_count: 0, edge_count: 0".to_string();
    let act = context.to_string();
    assert_eq!(exp, act);
}
