/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context_store::RelationKind;
use std::cmp::Ordering;

// Declaration order, written out rather than derived from the enum, so a reversed or rotated
// order fails here.
const DECLARED: [RelationKind; 4] = [
    RelationKind::Datial,
    RelationKind::Temporal,
    RelationKind::Spatial,
    RelationKind::SpaceTemporal,
];

#[test]
fn test_the_order_is_declaration_order() {
    let mut shuffled = [
        RelationKind::SpaceTemporal,
        RelationKind::Datial,
        RelationKind::Spatial,
        RelationKind::Temporal,
    ];
    shuffled.sort();
    assert_eq!(shuffled, DECLARED);
}

#[test]
fn test_every_pair_compares_by_declaration() {
    for (i, a) in DECLARED.iter().enumerate() {
        for (j, b) in DECLARED.iter().enumerate() {
            assert_eq!(a.cmp(b), i.cmp(&j), "{a} vs {b}");
            assert_eq!(a.partial_cmp(b), Some(i.cmp(&j)), "{a} vs {b}");
        }
    }
}

#[test]
fn test_datial_is_the_least_and_space_temporal_the_greatest() {
    assert_eq!(
        RelationKind::Datial.cmp(&RelationKind::SpaceTemporal),
        Ordering::Less
    );
    assert_eq!(DECLARED.iter().min(), Some(&RelationKind::Datial));
    assert_eq!(DECLARED.iter().max(), Some(&RelationKind::SpaceTemporal));
}
