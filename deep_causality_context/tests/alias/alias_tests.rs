/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `SubstrateContext` and `SubstrateContextoid`: the shape a reference-holding store accepts.
//! Expected values are the constructors' literals.
//!
//! Corner cases (rows A to K): B one node of each kind, `test_the_alias_snapshots_without_a_substrate`;
//! every other row n/a.

use deep_causality_context::{
    Contextoid, ContextoidType, ContextuableGraph, Data, DiscreteTime, EuclideanSpacetime,
    NedSpace, SpaceKind, SpaceTimeKind, SubstrateContext, SubstrateContextoid, SubstrateRef,
    TimeKind, TimeScale,
};
use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{ContextStorage, ContextoidId, DataRecord, NodeRecord};

#[test]
fn test_the_alias_snapshots_without_a_substrate() {
    let storage = MemoryStorage::new();
    let n: Vec<ContextoidId> = block_on(storage.reserve(4)).unwrap().collect();
    let mut ctx: SubstrateContext = SubstrateContext::with_capacity(1, "refs", 4);
    let reference = SubstrateRef::new("series".to_string(), "k".to_string());
    let nodes: [SubstrateContextoid; 4] = [
        Contextoid::new(
            n[0],
            ContextoidType::Datoid(Data::new(n[0], reference.clone())),
        ),
        Contextoid::new(
            n[1],
            ContextoidType::Spaceoid(SpaceKind::Ned(NedSpace::new(n[1], 1.0, 2.0, 3.0))),
        ),
        Contextoid::new(
            n[2],
            ContextoidType::Tempoid(TimeKind::Discrete(DiscreteTime::new(
                n[2],
                TimeScale::Steps,
                1,
            ))),
        ),
        Contextoid::new(
            n[3],
            ContextoidType::SpaceTempoid(SpaceTimeKind::Euclidean(EuclideanSpacetime::new(
                n[3],
                1.0,
                2.0,
                3.0,
                4.0,
                TimeScale::Second,
            ))),
        ),
    ];
    for node in nodes {
        ctx.add_node(node).unwrap();
    }
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot.nodes().len(), 4);
    assert_eq!(
        snapshot.nodes()[0].node(),
        &NodeRecord::Data(DataRecord::Reference(reference))
    );
    assert_eq!(block_on(storage.create_node(snapshot.nodes())), Ok(()));
    assert_eq!(
        block_on(storage.lookup(&n))
            .unwrap()
            .iter()
            .filter(|r| r.is_some())
            .count(),
        4
    );
}
