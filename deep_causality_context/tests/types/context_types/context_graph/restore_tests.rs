/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Context::restore`: a context rebuilt from a snapshot, and every refusal. Expected values are
//! the constructors' literals; equality is judged by the canonical snapshot, so a dropped node,
//! edge, weight or extra is caught.
//!
//! Corner cases (rows A to K): A an empty snapshot, `test_an_empty_snapshot_restores`; C two
//! restores of one snapshot agree, `test_two_restores_agree`; D the version exactly at
//! `RECORD_VERSION` accepted and one above refused, `test_a_newer_snapshot_is_refused`; F extra
//! identifier 0 refused, `test_an_extra_under_identifier_zero_is_refused`; the dangling edge, the
//! duplicate identifier and the duplicate edge, their own tests; every other row n/a.

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, CurrentDataIndex, Data, DiscreteTime,
    EuclideanSpacetime, ExtendableContextuableGraph, NedSpace, RelationKind, Root, SpaceKind,
    SpaceTimeKind, TimeKind, TimeScale, UniformContext, UniformContextoid,
};
use deep_causality_context_store::{
    ContextRecord, ContextSnapshot, ContextoidRecord, DataRecord, ExtraContextSnapshot, NodeRecord,
    ProjectionError, RECORD_VERSION, RelationRecord,
};

fn root(id: u64) -> UniformContextoid {
    Contextoid::new(id, ContextoidType::Root(Root::new(id)))
}

fn count(id: u64, value: u64) -> UniformContextoid {
    Contextoid::new(id, ContextoidType::Datoid(Data::new(id, value)))
}

/// A root, a data node, a time node, a space node, a spacetime node, edges of all four relation
/// kinds, and two named extras with nodes and an edge.
fn world() -> UniformContext {
    let mut ctx: UniformContext = Context::with_capacity(7, "world", 8);
    let r = ctx.add_node(root(1)).unwrap();
    let d = ctx.add_node(count(2, 5)).unwrap();
    let t = ctx
        .add_node(Contextoid::new(
            3,
            ContextoidType::Tempoid(TimeKind::Discrete(DiscreteTime::new(
                3,
                TimeScale::Steps,
                9,
            ))),
        ))
        .unwrap();
    let s = ctx
        .add_node(Contextoid::new(
            4,
            ContextoidType::Spaceoid(SpaceKind::Ned(NedSpace::new(4, 1.0, 2.0, 3.0))),
        ))
        .unwrap();
    let st = ctx
        .add_node(Contextoid::new(
            5,
            ContextoidType::SpaceTempoid(SpaceTimeKind::Euclidean(EuclideanSpacetime::new(
                5,
                1.0,
                2.0,
                3.0,
                4.0,
                TimeScale::Second,
            ))),
        ))
        .unwrap();
    ctx.add_edge(r, d, RelationKind::Datial).unwrap();
    ctx.add_edge(r, t, RelationKind::Temporal).unwrap();
    ctx.add_edge(s, st, RelationKind::Spatial).unwrap();
    ctx.add_edge(t, st, RelationKind::SpaceTemporal).unwrap();
    ctx.extra_ctx_add_new_with_id(40, "weather", 4, true)
        .unwrap();
    let w1 = ctx.extra_ctx_add_node(count(6, 1)).unwrap();
    let w2 = ctx.extra_ctx_add_node(count(7, 2)).unwrap();
    ctx.extra_ctx_add_edge(w1, w2, RelationKind::Temporal)
        .unwrap();
    ctx.extra_ctx_add_new_with_id(41, "terrain", 4, true)
        .unwrap();
    ctx.extra_ctx_add_node(count(8, 3)).unwrap();
    ctx
}

#[test]
fn test_a_context_restores_equal() {
    let original = world();
    let snapshot = original.snapshot().unwrap();
    let restored = UniformContext::restore(snapshot.clone()).unwrap();
    assert_eq!(restored.snapshot().unwrap(), snapshot);
    assert_eq!(restored.number_of_nodes(), 5);
    assert_eq!(restored.number_of_edges(), 4);
    assert_eq!(restored.name(), "world");
    for (from, to, kind) in [
        (1, 2, RelationKind::Datial),
        (1, 3, RelationKind::Temporal),
        (4, 5, RelationKind::Spatial),
        (3, 5, RelationKind::SpaceTemporal),
    ] {
        let a = restored.get_node_index_by_id(from).unwrap();
        let b = restored.get_node_index_by_id(to).unwrap();
        assert_eq!(restored.get_edge(a, b), Some(&kind));
    }
    assert_eq!(restored.extra_ctx_get_name(40), Some("weather"));
    assert_eq!(restored.extra_ctx_get_name(41), Some("terrain"));
    let mut restored = restored;
    restored.extra_ctx_set_current_id(40).unwrap();
    assert_eq!(restored.extra_ctx_node_count().unwrap(), 2);
    assert_eq!(restored.extra_ctx_edge_count().unwrap(), 1);
    restored.extra_ctx_set_current_id(41).unwrap();
    assert_eq!(restored.extra_ctx_node_count().unwrap(), 1);
}

#[test]
fn test_run_time_state_starts_empty() {
    let mut original = world();
    original.set_current_data_index(2);
    original.extra_ctx_set_current_id(40).unwrap();
    let restored = UniformContext::restore(original.snapshot().unwrap()).unwrap();
    assert_eq!(restored.get_current_data_index(), None);
    assert_eq!(restored.extra_ctx_get_current_id(), 0);
}

#[test]
fn test_a_restored_context_allocates_past_its_extras() {
    let mut restored = UniformContext::restore(world().snapshot().unwrap()).unwrap();
    assert_eq!(restored.extra_ctx_add_new("next", 4, false), 42);
}

#[test]
fn test_two_restores_agree() {
    let snapshot = world().snapshot().unwrap();
    let a = UniformContext::restore(snapshot.clone()).unwrap();
    let b = UniformContext::restore(snapshot).unwrap();
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
    assert_eq!(a.extra_ctx_get_name(40), b.extra_ctx_get_name(40));
    assert_eq!(a.get_node_index_by_id(5), b.get_node_index_by_id(5));
}

#[test]
fn test_an_empty_snapshot_restores() {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(3, "empty".to_string()),
        vec![],
        vec![],
        vec![],
    );
    let restored = UniformContext::restore(snapshot.clone()).unwrap();
    assert!(restored.is_empty());
    assert_eq!(restored.snapshot().unwrap(), snapshot);
}

#[test]
fn test_a_newer_snapshot_is_refused() {
    let context = || ContextRecord::new(3, "v".to_string());
    let current = ContextSnapshot::with_version(RECORD_VERSION, context(), vec![], vec![], vec![]);
    assert!(UniformContext::restore(current).is_ok());
    let newer =
        ContextSnapshot::with_version(RECORD_VERSION + 1, context(), vec![], vec![], vec![]);
    assert_eq!(
        UniformContext::restore(newer).map(|_| ()),
        Err(ProjectionError::Version(RECORD_VERSION + 1, RECORD_VERSION))
    );
}

#[test]
fn test_a_dangling_edge_is_refused() {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![ContextoidRecord::new(1, NodeRecord::Root)],
        vec![RelationRecord::new(1, 99, RelationKind::Datial)],
        vec![],
    );
    assert_eq!(
        UniformContext::restore(snapshot).map(|_| ()),
        Err(ProjectionError::Identity(
            99,
            "an edge names an identifier no node carries"
        ))
    );
    let in_extra = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![],
        vec![],
        vec![ExtraContextSnapshot::new(
            40,
            "w".to_string(),
            vec![ContextoidRecord::new(6, NodeRecord::Root)],
            vec![RelationRecord::new(98, 6, RelationKind::Datial)],
        )],
    );
    assert_eq!(
        UniformContext::restore(in_extra).map(|_| ()),
        Err(ProjectionError::Identity(
            98,
            "an edge names an identifier no node carries"
        ))
    );
}

#[test]
fn test_a_duplicate_identifier_is_refused() {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![
            ContextoidRecord::new(1, NodeRecord::Root),
            ContextoidRecord::new(1, NodeRecord::Data(DataRecord::Count(2))),
        ],
        vec![],
        vec![],
    );
    assert_eq!(
        UniformContext::restore(snapshot).map(|_| ()),
        Err(ProjectionError::Identity(
            1,
            "a node identifier is carried twice"
        ))
    );
    let twice_as_extra = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![],
        vec![],
        vec![
            ExtraContextSnapshot::new(40, "a".to_string(), vec![], vec![]),
            ExtraContextSnapshot::new(40, "b".to_string(), vec![], vec![]),
        ],
    );
    assert_eq!(
        UniformContext::restore(twice_as_extra).map(|_| ()),
        Err(ProjectionError::Identity(
            40,
            "an extra context identifier is carried twice"
        ))
    );
}

#[test]
fn test_an_extra_under_identifier_zero_is_refused() {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![],
        vec![],
        vec![ExtraContextSnapshot::new(
            0,
            "none".to_string(),
            vec![],
            vec![],
        )],
    );
    assert_eq!(
        UniformContext::restore(snapshot).map(|_| ()),
        Err(ProjectionError::Identity(
            0,
            "an extra context identifier is 0"
        ))
    );
}

#[test]
fn test_a_wrong_record_is_refused_at_its_node() {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![ContextoidRecord::new(
            5,
            NodeRecord::Data(DataRecord::Number(1.0)),
        )],
        vec![],
        vec![],
    );
    assert_eq!(
        UniformContext::restore(snapshot).map(|_| ()),
        Err(ProjectionError::WrongPayload(5, "Count", "Number"))
    );
}

#[test]
fn test_a_duplicate_edge_is_refused() {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(3, "d".to_string()),
        vec![
            ContextoidRecord::new(1, NodeRecord::Root),
            ContextoidRecord::new(2, NodeRecord::Root),
        ],
        vec![
            RelationRecord::new(1, 2, RelationKind::Datial),
            RelationRecord::new(1, 2, RelationKind::Spatial),
        ],
        vec![],
    );
    assert_eq!(
        UniformContext::restore(snapshot).map(|_| ()),
        Err(ProjectionError::Identity(1, "an edge is carried twice"))
    );
}
