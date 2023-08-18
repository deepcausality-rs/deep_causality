// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Contextoid, ContextoidType, Contextuable, Dataoid, Identifiable, Root, Spaceoid, SpaceTempoid, Tempoid};

#[test]
fn test_new()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, ContextoidType::Root(root));
    assert_eq!(node.id(), id);
}

#[test]
fn test_id()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, ContextoidType::Root(root));
    assert_eq!(node.id(), id);
}

#[test]
fn test_vertex_type()
{
    let id = 1;
    let root = Root::new(id);
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, ContextoidType::Root(root));
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
    let node: Contextoid<Dataoid, Spaceoid, Tempoid, SpaceTempoid> = Contextoid::new(id, ContextoidType::Root(root));

    let expected = "Contextoid ID: 1 Type: Root: Root ID: 1".to_string();
    let actual = node.to_string();
    assert_eq!(actual, expected);
}