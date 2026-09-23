/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, EuclideanTime,
    ExtendableContextuableGraph, Identifiable, RelationKind, Root, TimeScale,
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

#[test]
fn test_set_name() {
    let mut context = get_context();
    assert_eq!(context.name(), "base context");

    context.set_name("renamed context".to_string());
    assert_eq!(context.name(), "renamed context");
    assert_eq!(context.id(), 1, "renaming leaves the identity alone");
}

#[test]
fn test_get_node_index_by_id() {
    let mut context = get_context();

    // Contextoid ids are chosen by the caller and need not be dense or ordered, so the map from
    // id to graph index is not the identity. Adding them out of order makes that visible: a
    // lookup that returned the id itself, or the insertion counter, disagrees here.
    let ids = [42u64, 7, 1000];
    for (expected_index, id) in ids.iter().enumerate() {
        let node = Contextoid::new(*id, ContextoidType::Root(Root::new(*id)));
        let index = context.add_node(node).expect("failed to add node");
        assert_eq!(index, expected_index);
    }

    assert_eq!(context.get_node_index_by_id(42), Some(0));
    assert_eq!(context.get_node_index_by_id(7), Some(1));
    assert_eq!(context.get_node_index_by_id(1000), Some(2));

    // An id no contextoid carries resolves to nothing rather than to index zero.
    assert_eq!(context.get_node_index_by_id(0), None);
    assert_eq!(context.get_node_index_by_id(43), None);
}

#[test]
fn test_clone_copies_nodes_edges_and_name() {
    let mut context = get_context();
    context.set_name("original".to_string());

    let a = context
        .add_node(Contextoid::new(11, ContextoidType::Root(Root::new(11))))
        .expect("failed to add node a");
    let b = context
        .add_node(Contextoid::new(
            22,
            ContextoidType::Tempoid(EuclideanTime::new(22, TimeScale::Second, 5.0)),
        ))
        .expect("failed to add node b");
    context
        .add_edge(a, b, RelationKind::Temporal)
        .expect("failed to add edge");

    let cloned = context.clone();

    assert_eq!(cloned.id(), context.id());
    assert_eq!(cloned.name(), "original");
    assert_eq!(cloned.number_of_nodes(), context.number_of_nodes());
    assert_eq!(cloned.number_of_edges(), context.number_of_edges());
    assert_eq!(cloned.get_edge(a, b), Some(&RelationKind::Temporal));
    // The id-to-index map is a separate field from the graph, so a clone that rebuilt the graph
    // and left the map behind still answers the counts above and fails here.
    assert_eq!(cloned.get_node_index_by_id(11), Some(a));
    assert_eq!(cloned.get_node_index_by_id(22), Some(b));
}

#[test]
fn test_clone_is_independent_of_the_original() {
    let mut context = get_context();
    context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("failed to add node");

    let mut cloned = context.clone();
    cloned.set_name("clone".to_string());
    cloned
        .add_node(Contextoid::new(2, ContextoidType::Root(Root::new(2))))
        .expect("failed to add node to the clone");

    // A clone that shared the graph or the map would carry these writes back.
    assert_eq!(context.name(), "base context");
    assert_eq!(context.number_of_nodes(), 1);
    assert_eq!(context.get_node_index_by_id(2), None);

    assert_eq!(cloned.name(), "clone");
    assert_eq!(cloned.number_of_nodes(), 2);
    assert_eq!(cloned.get_node_index_by_id(2), Some(1));
}

#[test]
fn test_clone_carries_the_extra_contexts() {
    let mut context = get_context();
    let extra_id = context.extra_ctx_add_new("extra", 10, true);
    let node = context
        .extra_ctx_add_node(Contextoid::new(9, ContextoidType::Root(Root::new(9))))
        .expect("failed to add node to the extra context");

    let cloned = context.clone();

    // `extra_contexts` and `extra_context_id` are two separate fields. A clone that dropped
    // either of them reports a different answer here.
    assert!(cloned.extra_ctx_check_exists(extra_id));
    assert_eq!(cloned.extra_ctx_get_current_id(), extra_id);
    assert_eq!(cloned.extra_ctx_node_count().unwrap(), 1);
    assert!(cloned.extra_ctx_contains_node(node));
}
