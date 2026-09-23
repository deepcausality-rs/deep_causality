/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<NodeRecord>` for `Contextoid`: one arm per `ContextoidType`, a root as an ordinary
//! record, and the phantom arm as `Unrecordable`. Expected records are the inner constructors'
//! literals.
//!
//! Corner cases (rows A to K): C each of the five arms maps to a distinct `NodeRecord` variant,
//! `test_every_arm_round_trips`; the phantom arm, `test_the_phantom_arm_has_no_record`; every
//! other row is the node types' and is in their files.
use core::marker::PhantomData;
use deep_causality_context::{
    Contextoid, ContextoidType, Data, EuclideanTime, NedSpace, Root, SpaceKind, SpaceTimeKind,
    TangentSpacetime, TimeKind, TimeScale,
};
use deep_causality_context_store::{
    DataRecord, NodeRecord, ProjectionError, Recordable, SpaceRecord, TimeRecord,
};

type Node = Contextoid<Data<u64>, SpaceKind<f64>, TimeKind<f64>, SpaceTimeKind<f64>>;

fn one_of_each() -> Vec<Node> {
    vec![
        Contextoid::new(1, ContextoidType::Root(Root::new(1))),
        Contextoid::new(2, ContextoidType::Datoid(Data::new(2, 9))),
        Contextoid::new(
            3,
            ContextoidType::Tempoid(TimeKind::Euclidean(EuclideanTime::new(
                3,
                TimeScale::Second,
                1.5,
            ))),
        ),
        Contextoid::new(
            4,
            ContextoidType::Spaceoid(SpaceKind::Ned(NedSpace::new(4, 1.0, 2.0, 3.0))),
        ),
        Contextoid::new(
            5,
            ContextoidType::SpaceTempoid(SpaceTimeKind::Tangent(TangentSpacetime::new(
                5, 1.0, 2.0, 3.0, 4.0, 1.0, 0.5, 0.25, 0.125,
            ))),
        ),
    ]
}

#[test]
fn test_every_arm_round_trips() {
    let mut names = Vec::new();
    for (id, node) in (1u64..).zip(one_of_each()) {
        let record = node.to_record().unwrap();
        names.push(record.kind_name());
        assert_eq!(Node::from_record(id, record), Ok(node));
    }
    assert_eq!(names, ["Root", "Data", "Time", "Space", "SpaceTime"]);
}

#[test]
fn test_a_root_is_an_ordinary_record() {
    let root: Node = Contextoid::new(7, ContextoidType::Root(Root::new(7)));
    assert_eq!(root.to_record(), Ok(NodeRecord::Root));
    let restored = Node::from_record(7, NodeRecord::Root).unwrap();
    assert_eq!(restored, root);
    assert_eq!(restored.vertex_type().root(), Some(&Root::new(7)));
}

#[test]
fn test_the_inner_refusal_carries_through() {
    assert_eq!(
        Node::from_record(8, NodeRecord::Data(DataRecord::Number(1.0))),
        Err(ProjectionError::WrongPayload(8, "Count", "Number"))
    );
    assert_eq!(
        Node::from_record(
            8,
            NodeRecord::Space(SpaceRecord::Geo {
                lat: 1.0,
                lon: 2.0,
                alt: 3.0,
                datum: Default::default()
            })
        )
        .map(|n| n.vertex_type().spaceoid().is_some()),
        Ok(true)
    );
    assert_eq!(
        Node::from_record(8, NodeRecord::Time(TimeRecord::Entropic { tick: 1 }))
            .map(|n| n.vertex_type().tempoid().is_some()),
        Ok(true)
    );
}

#[test]
fn test_the_phantom_arm_has_no_record() {
    let phantom: Node = Contextoid::new(6, ContextoidType::_Marker(PhantomData));
    assert_eq!(
        phantom.to_record(),
        Err(ProjectionError::Unrecordable(6, "Marker"))
    );
}
