/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Context::snapshot`: every node and edge of the base graph and of each extra, by identifier,
//! in canonical order. Expected records are the constructors' literals; expected order follows
//! from the rule (nodes by identifier, edges by `(from, to)`, extras by identifier), never from
//! insertion order.
//!
//! Corner cases (rows A to K): A an empty context, `test_an_empty_context_snapshots`; B one
//! node, same test; C two contexts with the same content in different insertion orders,
//! `test_a_snapshot_is_canonical`; D identifiers far from indices, 100 and 200 at indices 0 and
//! 1, `test_a_snapshot_names_identifiers_never_indices`; a removed node, whose index is a
//! tombstone the walk steps over, `test_a_removed_node_is_not_recorded`; the absent spacetime in
//! a graph, `test_an_unrecordable_node_stops_the_walk`; every other row n/a.

use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, DiscreteTime, EuclideanSpacetime,
    EuclideanTime, ExtendableContextuableGraph, NedSpace, NoSpaceTime, RelationKind, Root,
    SpaceKind, SpaceTimeKind, TimeKind, TimeScale, UniformContext, UniformContextoid,
};
use deep_causality_context_store::{
    ContextoidRecord, DataRecord, NodeRecord, ProjectionError, RECORD_VERSION, SpaceRecord,
    TimeRecord,
};

fn root(id: u64) -> UniformContextoid {
    Contextoid::new(id, ContextoidType::Root(Root::new(id)))
}

fn count(id: u64, value: u64) -> UniformContextoid {
    Contextoid::new(id, ContextoidType::Datoid(Data::new(id, value)))
}

fn tick(id: u64, tick: u64) -> UniformContextoid {
    Contextoid::new(
        id,
        ContextoidType::Tempoid(TimeKind::Discrete(DiscreteTime::new(
            id,
            TimeScale::Steps,
            tick,
        ))),
    )
}

#[test]
fn test_an_empty_context_snapshots() {
    let ctx: UniformContext = Context::with_capacity(9, "empty", 4);
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot.version(), RECORD_VERSION);
    assert_eq!(snapshot.context().id(), 9);
    assert_eq!(snapshot.context().name(), "empty");
    assert!(
        snapshot.nodes().is_empty() && snapshot.edges().is_empty() && snapshot.extras().is_empty()
    );
    let mut one: UniformContext = Context::with_capacity(9, "one", 4);
    one.add_node(count(3, 5)).unwrap();
    let snapshot = one.snapshot().unwrap();
    assert_eq!(
        snapshot.nodes(),
        &[ContextoidRecord::new(
            3,
            NodeRecord::Data(DataRecord::Count(5))
        )]
    );
}

#[test]
fn test_a_snapshot_is_canonical() {
    let build = |order: [u64; 3]| {
        let mut ctx: UniformContext = Context::with_capacity(1, "c", 8);
        for id in order {
            ctx.add_node(count(id, id * 10)).unwrap();
        }
        let index = |ctx: &UniformContext, id| ctx.get_node_index_by_id(id).unwrap();
        ctx.add_edge(index(&ctx, 30), index(&ctx, 10), RelationKind::Datial)
            .unwrap();
        ctx.add_edge(index(&ctx, 10), index(&ctx, 20), RelationKind::Temporal)
            .unwrap();
        ctx.add_edge(index(&ctx, 10), index(&ctx, 30), RelationKind::Spatial)
            .unwrap();
        ctx
    };
    let forward = build([10, 20, 30]).snapshot().unwrap();
    let backward = build([30, 20, 10]).snapshot().unwrap();
    assert_eq!(forward, backward);
    let ids: Vec<u64> = forward.nodes().iter().map(|n| n.id()).collect();
    assert_eq!(ids, vec![10, 20, 30]);
    let ends: Vec<(u64, u64, RelationKind)> = forward
        .edges()
        .iter()
        .map(|e| (e.from(), e.to(), e.kind()))
        .collect();
    assert_eq!(
        ends,
        vec![
            (10, 20, RelationKind::Temporal),
            (10, 30, RelationKind::Spatial),
            (30, 10, RelationKind::Datial)
        ]
    );
}

#[test]
fn test_a_snapshot_names_identifiers_never_indices() {
    let mut ctx: UniformContext = Context::with_capacity(1, "c", 4);
    let a = ctx.add_node(count(100, 1)).unwrap();
    let b = ctx.add_node(count(200, 2)).unwrap();
    assert_eq!((a, b), (0, 1));
    ctx.add_edge(a, b, RelationKind::SpaceTemporal).unwrap();
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot.edges().len(), 1);
    assert_eq!(
        (snapshot.edges()[0].from(), snapshot.edges()[0].to()),
        (100, 200)
    );
}

#[test]
fn test_extras_are_recorded_with_identifier_and_name() {
    let mut ctx: UniformContext = Context::with_capacity(1, "base", 4);
    ctx.add_node(root(1)).unwrap();
    ctx.extra_ctx_add_new_with_id(41, "terrain", 4, true)
        .unwrap();
    let t = ctx.extra_ctx_add_node(tick(8, 3)).unwrap();
    ctx.extra_ctx_add_new_with_id(40, "weather", 4, true)
        .unwrap();
    let w1 = ctx.extra_ctx_add_node(count(6, 1)).unwrap();
    let w2 = ctx.extra_ctx_add_node(count(7, 2)).unwrap();
    ctx.extra_ctx_add_edge(w2, w1, RelationKind::Datial)
        .unwrap();
    let _ = t;
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot.nodes().len(), 1);
    assert_eq!(snapshot.extras().len(), 2);
    let (weather, terrain) = (&snapshot.extras()[0], &snapshot.extras()[1]);
    assert_eq!((weather.id(), weather.name()), (40, "weather"));
    assert_eq!((terrain.id(), terrain.name()), (41, "terrain"));
    let weather_ids: Vec<u64> = weather.nodes().iter().map(|n| n.id()).collect();
    assert_eq!(weather_ids, vec![6, 7]);
    assert_eq!((weather.edges()[0].from(), weather.edges()[0].to()), (7, 6));
    assert_eq!(
        terrain.nodes(),
        &[ContextoidRecord::new(
            8,
            NodeRecord::Time(TimeRecord::Discrete {
                scale: TimeScale::Steps,
                tick: 3
            })
        )]
    );
}

#[test]
fn test_every_node_kind_is_recorded() {
    let mut ctx: UniformContext = Context::with_capacity(1, "kinds", 8);
    ctx.add_node(root(1)).unwrap();
    ctx.add_node(count(2, 5)).unwrap();
    ctx.add_node(tick(3, 9)).unwrap();
    ctx.add_node(Contextoid::new(
        4,
        ContextoidType::Spaceoid(SpaceKind::Ned(NedSpace::new(4, 1.0, 2.0, 3.0))),
    ))
    .unwrap();
    ctx.add_node(Contextoid::new(
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
    let names: Vec<&str> = ctx
        .snapshot()
        .unwrap()
        .nodes()
        .iter()
        .map(|n| n.node().kind_name())
        .collect();
    assert_eq!(names, ["Root", "Data", "Time", "Space", "SpaceTime"]);
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(
        snapshot.nodes()[3].node(),
        &NodeRecord::Space(SpaceRecord::Ned {
            north: 1.0,
            east: 2.0,
            down: 3.0
        })
    );
}

#[test]
fn test_an_unrecordable_node_stops_the_walk() {
    type Clockwork = Context<Data<f64>, NoSpaceTime<f64>, EuclideanTime<f64>, NoSpaceTime<f64>>;
    let mut ctx: Clockwork = Context::with_capacity(1, "clock", 4);
    ctx.add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .unwrap();
    ctx.add_node(Contextoid::new(
        2,
        ContextoidType::Datoid(Data::new(2, 0.5)),
    ))
    .unwrap();
    ctx.add_node(Contextoid::new(
        3,
        ContextoidType::Tempoid(EuclideanTime::new(3, TimeScale::Second, 1.0)),
    ))
    .unwrap();
    assert_eq!(ctx.snapshot().unwrap().nodes().len(), 3);
    ctx.add_node(Contextoid::new(
        4,
        ContextoidType::Spaceoid(NoSpaceTime::new()),
    ))
    .unwrap();
    assert_eq!(
        ctx.snapshot(),
        Err(ProjectionError::Unrecordable(0, "NoSpaceTime"))
    );
}

#[test]
fn test_a_removed_node_is_not_recorded() {
    let mut ctx: UniformContext = Context::with_capacity(1, "c", 4);
    let a = ctx.add_node(count(10, 1)).unwrap();
    let b = ctx.add_node(count(20, 2)).unwrap();
    let c = ctx.add_node(count(30, 3)).unwrap();
    ctx.add_edge(a, b, RelationKind::Datial).unwrap();
    ctx.add_edge(b, c, RelationKind::Datial).unwrap();
    ctx.add_edge(a, c, RelationKind::Spatial).unwrap();
    ctx.remove_node(20).unwrap();
    let snapshot = ctx.snapshot().unwrap();
    let ids: Vec<u64> = snapshot.nodes().iter().map(|n| n.id()).collect();
    assert_eq!(ids, vec![10, 30]);
    assert_eq!(snapshot.edges().len(), 1);
    assert_eq!(
        (snapshot.edges()[0].from(), snapshot.edges()[0].to()),
        (10, 30)
    );
}

#[test]
fn test_a_parallel_edge_is_refused() {
    // The in-memory graph accepts a second edge between the same two nodes; a store holds one
    // relation per pair, so the snapshot refuses the graph rather than record a form restore
    // refuses.
    let mut ctx: UniformContext = Context::with_capacity(1, "parallel", 2);
    let a = ctx
        .add_node(Contextoid::new(1, ContextoidType::Datoid(Data::new(1, 1))))
        .unwrap();
    let b = ctx
        .add_node(Contextoid::new(2, ContextoidType::Datoid(Data::new(2, 2))))
        .unwrap();
    ctx.add_edge(a, b, RelationKind::Datial).unwrap();
    assert!(ctx.snapshot().is_ok());
    ctx.add_edge(a, b, RelationKind::Spatial).unwrap();
    assert_eq!(
        ctx.snapshot().map(|_| ()),
        Err(ProjectionError::Identity(1, "an edge is carried twice"))
    );
    // The opposite direction is a different relation and is recorded.
    ctx.remove_edge(a, b).unwrap();
    ctx.add_edge(b, a, RelationKind::Spatial).unwrap();
    assert_eq!(ctx.snapshot().unwrap().edges().len(), 2);
}
