// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Dataoid, Identifiable, Contextoid, NodeType, Root, Spaceoid, SpaceTempoid, Tempoid};

#[test]
fn test_new()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, NodeType::Root(root));
    assert_eq!(node.id(), id);
}

#[test]
fn test_id()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, NodeType::Root(root));
    assert_eq!(node.id(), id);
}

#[test]
fn test_vertex_type()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, NodeType::Root(root));
    assert!(node.vertex_type().root().is_some());
    assert!(node.vertex_type().dataoid().is_none());
    assert!(node.vertex_type().tempoid().is_none());
    assert!(node.vertex_type().spaceiod().is_none());
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_to_string()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, NodeType::Root(root));

    let expected = format!("Vertex ID: 1 Type: Root: Root ID: 1");
    let actual = node.to_string();
    assert_eq!(actual, expected);
}