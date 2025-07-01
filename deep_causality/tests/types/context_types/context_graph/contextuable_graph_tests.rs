/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::prelude::{
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
