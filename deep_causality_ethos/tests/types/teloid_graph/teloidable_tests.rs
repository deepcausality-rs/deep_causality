/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A teloid pair carries one relation. `TeloidGraph` exposes no edge read, so the relation a
//! pair carries is observed through conflict resolution on an `EffectEthos` built from the graph.

use deep_causality_ethos::utils_test::test_utils_effect_ethos::{
    TestEthos, always_true_predicate, get_dummy_action, get_dummy_context,
};
use deep_causality_ethos::{
    DeonticError, DeonticInferable, TagIndex, Teloid, TeloidGraph, TeloidModal, TeloidRelation,
    TeloidStorable, TeloidStore, Teloidable,
};

/// A graph holding teloid 1 at index 0 and teloid 2 at index 1, with no edge.
fn two_teloids() -> TeloidGraph {
    let mut graph = TeloidGraph::new();
    assert_eq!(graph.add_teloid(1).unwrap(), 0);
    assert_eq!(graph.add_teloid(2).unwrap(), 1);
    graph
}

/// The relation the pair (0, 1) carries, read through conflict resolution.
///
/// Norm 1 (index 0) and norm 2 (index 1) are both active and tagged "drive"; norm 1 has
/// priority 10, norm 2 priority 1, so norm 1 outranks norm 2. Under `Inherits` nothing defeats
/// norm 2, so both stand and the justification is [1, 2]. Under `Defeats` the standing,
/// higher-ranked norm 1 defeats norm 2, so only norm 1 stands and the justification is [1].
fn pair_relation(graph: TeloidGraph) -> TeloidRelation {
    let mut store = TeloidStore::new();
    for (id, priority) in [(1, 10), (2, 1)] {
        store.insert(Teloid::new_deterministic(
            id,
            "drive".to_string(),
            always_true_predicate,
            TeloidModal::Optional(1),
            1,
            1,
            priority,
            vec!["drive"],
            None,
        ));
    }
    let mut index = TagIndex::new();
    index.add("drive", 1);
    index.add("drive", 2);

    let mut ethos = TestEthos::from(store, index, graph);
    ethos.verify_graph().unwrap();
    let verdict = ethos
        .evaluate_action(
            &get_dummy_action("drive", 40.0),
            &get_dummy_context(),
            &["drive"],
        )
        .unwrap();
    let mut justification = verdict.justification().clone();
    justification.sort();
    match justification.as_slice() {
        [1, 2] => TeloidRelation::Inherits,
        [1] => TeloidRelation::Defeats,
        other => panic!("justification {other:?} matches neither relation"),
    }
}

#[test]
fn test_a_lone_inheritance_edge_reads_as_inherits() {
    // Calibrates the oracle: one Inherits edge, norm 2 stands beside norm 1.
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();
    assert_eq!(pair_relation(graph), TeloidRelation::Inherits);
}

#[test]
fn test_a_lone_defeasance_edge_reads_as_defeats() {
    // Calibrates the oracle: one Defeats edge, norm 1 outranks and removes norm 2.
    let mut graph = two_teloids();
    graph.add_defeasance_edge(0, 1).unwrap();
    assert_eq!(pair_relation(graph), TeloidRelation::Defeats);
}

#[test]
fn test_defeasance_after_inheritance_is_refused_and_the_pair_keeps_inherits() {
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();

    assert_eq!(
        graph.add_defeasance_edge(0, 1),
        Err(DeonticError::ConflictingRelation { from: 0, to: 1 })
    );
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_edge(0, 1));
    assert_eq!(pair_relation(graph), TeloidRelation::Inherits);
}

#[test]
fn test_inheritance_after_defeasance_is_refused_and_the_pair_keeps_defeats() {
    let mut graph = two_teloids();
    graph.add_defeasance_edge(0, 1).unwrap();

    assert_eq!(
        graph.add_inheritance_edge(0, 1),
        Err(DeonticError::ConflictingRelation { from: 0, to: 1 })
    );
    assert_eq!(graph.number_edges(), 1);
    assert!(graph.contains_edge(0, 1));
    assert_eq!(pair_relation(graph), TeloidRelation::Defeats);
}

#[test]
fn test_re_adding_inheritance_is_accepted_and_counts_one_edge() {
    let mut graph = two_teloids();

    assert_eq!(graph.add_inheritance_edge(0, 1), Ok(()));
    assert_eq!(graph.add_inheritance_edge(0, 1), Ok(()));
    // One ordered pair, one relation: one edge.
    assert_eq!(graph.number_edges(), 1);
    assert_eq!(pair_relation(graph), TeloidRelation::Inherits);
}

#[test]
fn test_re_adding_defeasance_is_accepted_and_counts_one_edge() {
    let mut graph = two_teloids();

    assert_eq!(graph.add_defeasance_edge(0, 1), Ok(()));
    assert_eq!(graph.add_defeasance_edge(0, 1), Ok(()));
    assert_eq!(graph.number_edges(), 1);
    assert_eq!(pair_relation(graph), TeloidRelation::Defeats);
}

#[test]
fn test_the_opposite_direction_is_a_different_pair() {
    let mut graph = two_teloids();

    assert_eq!(graph.add_inheritance_edge(0, 1), Ok(()));
    assert_eq!(graph.add_defeasance_edge(1, 0), Ok(()));
    // (0, 1) and (1, 0) are two ordered pairs: two edges.
    assert_eq!(graph.number_edges(), 2);
    assert!(graph.contains_edge(0, 1));
    assert!(graph.contains_edge(1, 0));
}

#[test]
fn test_a_conflict_names_the_graph_indices_of_the_pair() {
    // Three teloids at indices 0, 1 and 2; the conflicting pair is (2, 1), so the error names
    // 2 and 1, not the teloid ids 30 and 20.
    let mut graph = TeloidGraph::new();
    assert_eq!(graph.add_teloid(10).unwrap(), 0);
    assert_eq!(graph.add_teloid(20).unwrap(), 1);
    assert_eq!(graph.add_teloid(30).unwrap(), 2);
    graph.add_defeasance_edge(2, 1).unwrap();

    assert_eq!(
        graph.add_inheritance_edge(2, 1),
        Err(DeonticError::ConflictingRelation { from: 2, to: 1 })
    );
    assert_eq!(graph.number_edges(), 1);
}
