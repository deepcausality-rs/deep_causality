// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Context, Contextoid, Contextuable, Dataoid, ContextoidType, Root, Spaceoid, SpaceTempoid, Tempoid, TimeScale, RelationKind};

fn get_context<'l>() -> Context<'l, Dataoid, Spaceoid, Tempoid, SpaceTempoid>
{
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
fn test_add_contextoid() {
    let id = 1;
    let name = format!("base context");

    let mut context = get_context();
    assert_eq!(context.id(), id);
    assert_eq!(context.name(), name);
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
}

#[test]
fn test_contains_contextoid() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
    assert!(context.contains_contextoid(idx))
}

#[test]
fn test_get_contextoid() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
    assert!(context.contains_contextoid(idx));

    let contextoid = context.get_contextoid(idx);
    assert!(contextoid.is_some());
}


#[test]
fn test_remove_contextoid() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    let idx = context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
    assert!(context.contains_contextoid(idx));

    let contextoid = context.get_contextoid(idx);
    assert!(contextoid.is_some());

    context.remove_contextoid(idx);
    let contextoid = context.get_contextoid(idx);
    assert!(contextoid.is_none());
}

#[test]
fn test_add_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    let roodidx = context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
    assert!(context.contains_contextoid(roodidx));

    let contextoid = context.get_contextoid(roodidx);
    assert!(contextoid.is_some());

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12;
    let tempoid = Tempoid::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let contextoid  = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let t_idx = context.add_contextoid(&contextoid);

    context.add_edge(roodidx, t_idx, RelationKind::Temporal);

    assert!(context.contains_edge(roodidx, t_idx));
}

#[test]
fn test_contains_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    let roodidx = context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
    assert!(context.contains_contextoid(roodidx));

    let contextoid = context.get_contextoid(roodidx);
    assert!(contextoid.is_some());
    
    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12;
    let tempoid = Tempoid::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let contextoid  = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let t_idx = context.add_contextoid(&contextoid);
    context.add_edge(roodidx, t_idx, RelationKind::Temporal);

    assert!(context.contains_edge(roodidx, t_idx));
}

#[test]
fn test_remove_edge() {
    let id = 1;
    let mut context = get_context();
    assert_eq!(context.size(), 0);

    let root = Root::new(id);
    let contextoid  = Contextoid::new(id, ContextoidType::Root(root));
    let roodidx = context.add_contextoid(&contextoid);

    assert_eq!(context.size(), 1);
    assert!(context.contains_contextoid(roodidx));

    let contextoid = context.get_contextoid(roodidx);
    assert!(contextoid.is_some());

    let t_id = 12;
    let t_time_scale = TimeScale::Month;
    let t_time_unit = 12;
    let tempoid = Tempoid::new(t_id, t_time_scale, t_time_unit);

    let id = 2;
    let contextoid  = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    let t_idx = context.add_contextoid(&contextoid);
    context.add_edge(roodidx, t_idx, RelationKind::Temporal);

    assert!(context.contains_edge(roodidx, t_idx));

    context.remove_edge(roodidx, t_idx);
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

    let exp = format!("Context: id: 1, name: base context, node_count: 0, edge_count: 0");
    let act = context.to_string();
    assert_eq!(exp, act);
}
