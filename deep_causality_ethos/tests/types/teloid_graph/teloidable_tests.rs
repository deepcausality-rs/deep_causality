/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A teloid pair carries one relation. `TeloidGraph::relation` reads it directly; the
//! `pair_relation` oracle below observes it a second way, through conflict resolution on an
//! `EffectEthos` built from the graph, and the two calibration tests show that the oracle
//! agrees with the direct read on a lone edge of each kind.

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
///
/// A pair with no edge also leaves both norms standing and yields [1, 2], so this oracle
/// cannot tell `Inherits` from no edge. The tests that pin what a pair carries use
/// `TeloidGraph::relation`; this oracle only shows that resolution agrees with that read.
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
fn test_relation_is_none_on_a_pair_with_no_edge() {
    let graph = two_teloids();
    assert_eq!(graph.relation(0, 1), None);
}

#[test]
fn test_relation_reads_inherits() {
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
}

#[test]
fn test_relation_reads_defeats() {
    let mut graph = two_teloids();
    graph.add_defeasance_edge(0, 1).unwrap();
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Defeats));
}

#[test]
fn test_relation_is_none_in_the_direction_without_an_edge() {
    // Only 0 -> 1 exists; (1, 0) is a different ordered pair and carries nothing.
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
    assert_eq!(graph.relation(1, 0), None);
}

#[test]
fn test_relation_is_none_for_an_absent_source_node() {
    // Index 5 holds no teloid, so there are no edges to search.
    let graph = two_teloids();
    assert_eq!(graph.relation(5, 1), None);
}

#[test]
fn test_a_lone_inheritance_edge_reads_as_inherits() {
    // Calibrates the oracle against the direct read: one Inherits edge, norm 2 stands
    // beside norm 1.
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
    assert_eq!(pair_relation(graph), TeloidRelation::Inherits);
}

#[test]
fn test_a_lone_defeasance_edge_reads_as_defeats() {
    // Calibrates the oracle against the direct read: one Defeats edge, norm 1 outranks and
    // removes norm 2.
    let mut graph = two_teloids();
    graph.add_defeasance_edge(0, 1).unwrap();
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Defeats));
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
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
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
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Defeats));
}

#[test]
fn test_re_adding_inheritance_is_accepted_and_counts_one_edge() {
    let mut graph = two_teloids();

    assert_eq!(graph.add_inheritance_edge(0, 1), Ok(()));
    assert_eq!(graph.add_inheritance_edge(0, 1), Ok(()));
    // One ordered pair, one relation: one edge.
    assert_eq!(graph.number_edges(), 1);
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
}

#[test]
fn test_re_adding_defeasance_is_accepted_and_counts_one_edge() {
    let mut graph = two_teloids();

    assert_eq!(graph.add_defeasance_edge(0, 1), Ok(()));
    assert_eq!(graph.add_defeasance_edge(0, 1), Ok(()));
    assert_eq!(graph.number_edges(), 1);
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Defeats));
}

#[test]
fn test_the_opposite_direction_is_a_different_pair() {
    let mut graph = two_teloids();

    assert_eq!(graph.add_inheritance_edge(0, 1), Ok(()));
    assert_eq!(graph.add_defeasance_edge(1, 0), Ok(()));
    // (0, 1) and (1, 0) are two ordered pairs: two edges, each with its own relation.
    assert_eq!(graph.number_edges(), 2);
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
    assert_eq!(graph.relation(1, 0), Some(TeloidRelation::Defeats));
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
    assert_eq!(graph.relation(2, 1), Some(TeloidRelation::Defeats));
}

#[test]
fn test_a_frozen_graph_refuses_the_other_relation_as_frozen_not_as_a_conflict() {
    // The pair already carries Inherits, so an unfrozen graph would answer
    // ConflictingRelation; frozen, the freeze is checked first and wins.
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();
    graph.freeze();

    assert_eq!(
        graph.add_defeasance_edge(0, 1),
        Err(DeonticError::GraphIsFrozen(
            ultragraph::GraphError::GraphIsFrozen
        ))
    );
    assert_eq!(graph.relation(0, 1), Some(TeloidRelation::Inherits));
}

#[test]
fn test_a_frozen_graph_refuses_the_same_relation_as_frozen() {
    // Re-adding the carried relation is accepted on an unfrozen graph; frozen, it is refused.
    let mut graph = two_teloids();
    graph.add_inheritance_edge(0, 1).unwrap();
    graph.freeze();

    assert_eq!(
        graph.add_inheritance_edge(0, 1),
        Err(DeonticError::GraphIsFrozen(
            ultragraph::GraphError::GraphIsFrozen
        ))
    );
}
