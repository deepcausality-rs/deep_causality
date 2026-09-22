/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors; no expectation is computed.
//!
//! Corner cases (rows A to K): A/B n/a; C coinciding payloads across variants n/a (the inner
//! records have distinct types); D to K n/a (no arithmetic; the inner records carry their own
//! rows).

use deep_causality_context_store::{
    DataRecord, NodeRecord, SpaceRecord, SpaceTimeRecord, TimeRecord, TimeScale,
};

fn one_of_each() -> Vec<NodeRecord> {
    vec![
        NodeRecord::Root,
        NodeRecord::Data(DataRecord::Count(1)),
        NodeRecord::Time(TimeRecord::Entropic { tick: 2 }),
        NodeRecord::Space(SpaceRecord::Ecef {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        }),
        NodeRecord::SpaceTime(SpaceTimeRecord::Euclidean {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
            scale: TimeScale::Second,
        }),
    ]
}

#[test]
fn test_five_variants_with_distinct_names() {
    let names: Vec<&str> = one_of_each().iter().map(NodeRecord::kind_name).collect();
    assert_eq!(names, ["Root", "Data", "Time", "Space", "SpaceTime"]);
}

#[test]
fn test_root_is_an_ordinary_record() {
    let root = NodeRecord::Root;
    assert_eq!(root.clone(), NodeRecord::Root);
    assert_ne!(root, NodeRecord::Data(DataRecord::Flag(false)));
    assert!(format!("{root:?}").contains("Root"));
}

#[test]
fn test_clone_equality() {
    for record in one_of_each() {
        assert_eq!(record.clone(), record);
    }
}
