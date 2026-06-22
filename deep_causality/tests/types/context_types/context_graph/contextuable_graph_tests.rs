/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, RelationKind, Root,
};

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

#[test]
fn test_update_node_err() {
    let mut context = get_context();
    let id = 1;
    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.update_node(id, contextoid);
    assert!(res.is_err());
}

#[test]
fn test_add_edge_err() {
    let mut context = get_context();
    let res = context.add_edge(1, 2, RelationKind::Datial);
    assert!(res.is_err());
}

#[test]
fn test_remove_edge_err() {
    let mut context = get_context();
    let res = context.remove_edge(1, 2);
    assert!(res.is_err());
}

#[test]
fn test_remove_node_err() {
    let mut context = get_context();
    let id = 999;
    let res = context.remove_node(id);
    assert!(res.is_err());
}

#[test]
fn test_update_node_changes_id_mapping() {
    // Add a node, then update it with a node carrying a *different* ID. This
    // exercises the `new_node_id != node_id` branch that rewrites the
    // id-to-index map.
    let mut context = get_context();
    let old_id = 1;
    let new_id = 2;

    let idx = context
        .add_node(Contextoid::new(
            old_id,
            ContextoidType::Root(Root::new(old_id)),
        ))
        .expect("add node");

    let res = context.update_node(
        old_id,
        Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id))),
    );
    assert!(res.is_ok());

    // The node now lives under the new id; updating the old id must fail.
    let stale = context.update_node(
        old_id,
        Contextoid::new(old_id, ContextoidType::Root(Root::new(old_id))),
    );
    assert!(stale.is_err());

    // Updating via the new id still resolves to the same index.
    let ok = context.update_node(
        new_id,
        Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id))),
    );
    assert!(ok.is_ok());
    assert!(context.contains_node(idx));
}

#[test]
fn test_add_edge_err_second_index_missing() {
    // First index present, second index missing: exercises the `index b`
    // not-found guard specifically.
    let mut context = get_context();
    let a = context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("add node a");

    let res = context.add_edge(a, 999, RelationKind::Datial);
    assert!(res.is_err());
    assert!(format!("{:?}", res.unwrap_err()).contains("index b"));
}

#[test]
fn test_remove_edge_err_second_index_missing() {
    // First index present, second index missing: exercises the `index b`
    // not-found guard in remove_edge.
    let mut context = get_context();
    let a = context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("add node a");

    let res = context.remove_edge(a, 999);
    assert!(res.is_err());
    assert!(format!("{:?}", res.unwrap_err()).contains("index b"));
}
