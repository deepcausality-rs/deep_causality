// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{
    BaseContextoid, Contextoid, ContextoidType, Contextuable, Data, Root, Space, SpaceTime, Time,
    TimeScale,
};

#[test]
fn test_root_some() {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));
    assert!(node.vertex_type().root().is_some());
    //
    assert!(node.vertex_type().dataoid().is_none());
    assert!(node.vertex_type().tempoid().is_none());
    assert!(node.vertex_type().spaceiod().is_none());
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_root_none() {
    let id = 1;
    let data = 42;
    let d = Data::new(id, data);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Datoid(d));
    assert!(node.vertex_type().root().is_none());
}

#[test]
fn test_dataoid_some() {
    let id = 1;
    let data = 42;
    let d = Data::new(id, data);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Datoid(d));
    assert!(node.vertex_type().dataoid().is_some());
    //
    assert!(node.vertex_type().root().is_none());
    assert!(node.vertex_type().tempoid().is_none());
    assert!(node.vertex_type().spaceiod().is_none());
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_dataoid_none() {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));
    assert!(node.vertex_type().dataoid().is_none());
}

#[test]
fn test_tempoid_some() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 1;

    let tempoid = Time::new(id, time_scale, time_unit);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Tempoid(tempoid));
    assert!(node.vertex_type().tempoid().is_some());
    //
    assert!(node.vertex_type().dataoid().is_none());
    assert!(node.vertex_type().root().is_none());
    assert!(node.vertex_type().spaceiod().is_none());
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_tempoid_none() {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));
    assert!(node.vertex_type().tempoid().is_none());
}

#[test]
fn test_spaceiod_some() {
    let id = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = Space::new(id, x, y, z);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Spaceoid(d));
    assert!(node.vertex_type().spaceiod().is_some());
    //
    assert!(node.vertex_type().dataoid().is_none());
    assert!(node.vertex_type().root().is_none());
    assert!(node.vertex_type().tempoid().is_none());
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_spaceiod_none() {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));
    assert!(node.vertex_type().spaceiod().is_none());
}

#[test]
fn test_space_tempoid_some() {
    let id = 1;
    let time_scale = TimeScale::Month;
    let time_unit = 1;
    let x = 7;
    let y = 8;
    let z = 9;

    let d = SpaceTime::new(id, time_scale, time_unit, x, y, z);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::SpaceTempoid(d));
    assert!(node.vertex_type().space_tempoid().is_some());
    //
    assert!(node.vertex_type().dataoid().is_none());
    assert!(node.vertex_type().root().is_none());
    assert!(node.vertex_type().tempoid().is_none());
    assert!(node.vertex_type().spaceiod().is_none());
}

#[test]
fn test_space_tempoid_none() {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));
    assert!(node.vertex_type().space_tempoid().is_none());
}

#[test]
fn test_to_string() {
    let id = 1;
    let root = Root::new(id);
    let node: BaseContextoid = Contextoid::new(id, ContextoidType::Root(root));

    let expected = "Contextoid ID: 1 Type: Root: Root ID: 1".to_string();
    let actual = node.to_string();
    assert_eq!(actual, expected);
}
