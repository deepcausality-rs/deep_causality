/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context_store::RelationKind;
use ultragraph_traits::EdgeKey;

// Compiles only when the key type is the relation itself.
fn relation_key<T: EdgeKey<Key = RelationKind>>(payload: &T) -> RelationKind {
    payload.edge_key()
}

#[test]
fn test_the_key_is_the_relation() {
    assert_eq!(relation_key(&RelationKind::Datial), RelationKind::Datial);
    assert_eq!(
        relation_key(&RelationKind::Temporal),
        RelationKind::Temporal
    );
    assert_eq!(relation_key(&RelationKind::Spatial), RelationKind::Spatial);
    assert_eq!(
        relation_key(&RelationKind::SpaceTemporal),
        RelationKind::SpaceTemporal
    );
}

#[test]
fn test_distinct_relations_have_distinct_keys() {
    let kinds = [
        RelationKind::Datial,
        RelationKind::Temporal,
        RelationKind::Spatial,
        RelationKind::SpaceTemporal,
    ];
    for (i, a) in kinds.iter().enumerate() {
        for (j, b) in kinds.iter().enumerate() {
            assert_eq!(a.edge_key() == b.edge_key(), i == j, "{a} vs {b}");
        }
    }
}
