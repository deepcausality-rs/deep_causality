// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Context, Dataoid, Spaceoid, SpaceTempoid, Tempoid};


fn get_context<'l>() -> Context<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
    let id = 1;
    let name = format!("base context");
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
    let name = format!("base context");

    let context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), &name);
}

#[test]
fn test_node_count() {
    let id = 1;
    let name = format!("base context");

    let context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);
    let node_count = 0;
    assert_eq!(context.node_count(), node_count);
}

#[test]
fn test_edge_count() {
    let id = 1;
    let name = format!("base context");

    let context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);

    let node_count = 0;
    assert_eq!(context.node_count(), node_count);
    let edge_count = 0;
    assert_eq!(context.edge_count(), edge_count);
}

#[test]
fn test_to_string() {
    let context = get_context();

    let exp = format!("Context: id: 1, name: base context, node_count: 0, edge_count: 0");
    let act = context.to_string();
    assert_eq!(exp, act);
}
