// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Context, ContextMatrixGraph, Dataoid, Spaceoid, SpaceTempoid, Tempoid};

fn get_graph() -> ContextMatrixGraph<Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    ContextMatrixGraph::with_capacity(10)
}

#[test]
fn test_new() {
    let id = 1;
    let name = format!("base context");
    let graph = get_graph();

    let context = Context::new(id, name, &graph);
    assert_eq!(context.id(), id);
}

#[test]
fn test_id() {
    let id = 1;
    let name = format!("base context");
    let graph = get_graph();

    let context = Context::new(id, name, &graph);
    assert_eq!(context.id(), id);
}

#[test]
fn test_name() {
    let id = 1;
    let name = format!("base context");
    let graph = get_graph();

    let context = Context::new(id, name.clone(), &graph);
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), &name);
}

#[test]
fn test_node_count() {
    let id = 1;
    let name = format!("base context");
    let graph = get_graph();

    let context = Context::new(id, name.clone(), &graph);
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);
    let node_count = graph.node_count();
    assert_eq!(context.node_count(), node_count);
}

#[test]
fn test_edge_count() {
    let id = 1;
    let name = format!("base context");
    let graph = get_graph();

    let context = Context::new(id, name.clone(), &graph);
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);

    let node_count = graph.node_count();
    assert_eq!(context.node_count(), node_count);
    let edge_count = graph.edge_count();
    assert_eq!(context.edge_count(), edge_count);
}

#[test]
fn test_to_string() {
    let id = 1;
    let name = format!("base context");
    let graph = get_graph();

    let context = Context::new(id, name.clone(), &graph);

    let exp = format!("Context: id: 1, name: base context, node_count: 0, edge_count: 0");
    let act = context.to_string();
    assert_eq!(exp, act);
}
