/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ordinal assignment, and the property it exists to provide.
//!
//! The decisive test is [`two_separately_built_identical_graphs_draw_alike`]: it fails if a node
//! address ever reaches the generator, which is the one mistake this design is built to avoid and
//! the one that no single-process run would reveal.

use deep_causality_uncertain::UncertainBool;
use deep_causality_uncertain::{LeafOrdinals, SampleSession, Uncertain};

const SEED: u64 = 0x5EED_2026;

/// `normal(10, 1) + uniform(0, 1)` — two drawing leaves, so an ordinal scheme has more than one
/// slot to get wrong.
fn two_leaf_graph() -> Uncertain<f64> {
    Uncertain::<f64>::normal(10.0, 1.0) + Uncertain::<f64>::uniform(0.0, 1.0)
}

#[test]
fn each_drawing_leaf_takes_one_ordinal() {
    let ordinals = LeafOrdinals::new(&two_leaf_graph());

    assert_eq!(ordinals.len(), 2);
    assert!(!ordinals.is_empty());
}

#[test]
fn ordinals_are_dense_from_zero() {
    let graph = Uncertain::<f64>::normal(0.0, 1.0)
        + Uncertain::<f64>::uniform(0.0, 1.0)
        + Uncertain::<f64>::normal(5.0, 2.0);
    let ordinals = LeafOrdinals::new(&graph);

    assert_eq!(ordinals.assigned(), vec![0, 1, 2]);
}

#[test]
fn a_point_leaf_takes_no_ordinal() {
    // It consumes no entropy, so giving it a slot would leave a gap in the addresses of the
    // leaves that do draw.
    let ordinals = LeafOrdinals::new(&Uncertain::<f64>::point(42.0));

    assert!(ordinals.is_empty());
    assert_eq!(ordinals.len(), 0);
}

#[test]
fn a_graph_of_point_values_draws_nothing() {
    let graph = Uncertain::<f64>::point(1.0) + Uncertain::<f64>::point(2.0);

    assert!(LeafOrdinals::new(&graph).is_empty());
}

#[test]
fn a_leaf_reached_twice_takes_one_ordinal() {
    // `x + x` is one leaf reached along two paths. Two ordinals would make it two independent
    // draws, and the sum would no longer be twice the value.
    let x = Uncertain::<f64>::normal(10.0, 1.0);
    let doubled = x.clone() + x;

    assert_eq!(LeafOrdinals::new(&doubled).len(), 1);
}

#[test]
fn a_shared_leaf_is_drawn_once_per_sample() {
    let x = Uncertain::<f64>::normal(10.0, 1.0);
    let doubled = x.clone() + x.clone();
    let session = SampleSession::seeded(SEED);

    for index in 0..16 {
        let once = x.sample_at(&session, index).unwrap();
        let twice = doubled.sample_at(&session, index).unwrap();
        assert_eq!(twice, once * 2.0, "at index {index}");
    }
}

#[test]
fn two_separately_built_identical_graphs_draw_alike() {
    // The test that pins the whole design. Two graphs built by the same calls occupy different
    // addresses, so if an address reached the generator these would disagree — and they would
    // disagree differently on every run, which no amount of re-running a single graph would show.
    let left = two_leaf_graph();
    let right = two_leaf_graph();
    let session = SampleSession::seeded(SEED);

    for index in 0..64 {
        assert_eq!(
            left.sample_at(&session, index).unwrap(),
            right.sample_at(&session, index).unwrap(),
            "two identical graphs disagreed at index {index}"
        );
    }
}

#[test]
fn the_same_index_means_the_same_draw_for_graphs_sharing_a_leaf() {
    // Stronger than the cache it replaces, which was keyed by root and so made two graphs over one
    // leaf disagree about that leaf entirely.
    let shared = Uncertain::<f64>::normal(10.0, 1.0);
    let plus_one = shared.clone() + Uncertain::<f64>::point(1.0);
    let session = SampleSession::seeded(SEED);

    for index in 0..32 {
        let alone = shared.sample_at(&session, index).unwrap();
        let within = plus_one.sample_at(&session, index).unwrap();
        assert_eq!(within, alone + 1.0, "at index {index}");
    }
}

#[test]
fn a_seeded_session_replays_its_own_draws() {
    let graph = two_leaf_graph();
    let first: Vec<f64> = (0..32)
        .map(|i| graph.sample_at(&SampleSession::seeded(SEED), i).unwrap())
        .collect();
    let again: Vec<f64> = (0..32)
        .map(|i| graph.sample_at(&SampleSession::seeded(SEED), i).unwrap())
        .collect();

    assert_eq!(first, again);
}

#[test]
fn different_seeds_give_different_draws() {
    let graph = two_leaf_graph();
    let left = SampleSession::seeded(1);
    let right = SampleSession::seeded(2);

    let agreements = (0..64)
        .filter(|&i| graph.sample_at(&left, i).unwrap() == graph.sample_at(&right, i).unwrap())
        .count();

    assert_eq!(agreements, 0, "two seeds produced the same draw");
}

#[test]
fn different_indices_give_different_draws() {
    let graph = two_leaf_graph();
    let session = SampleSession::seeded(SEED);
    let drawn: Vec<f64> = (0..64)
        .map(|i| graph.sample_at(&session, i).unwrap())
        .collect();

    let mut unique = drawn.clone();
    unique.sort_by(f64::total_cmp);
    unique.dedup();

    assert_eq!(unique.len(), drawn.len(), "two indices produced one value");
}

#[test]
fn reusing_ordinals_gives_the_same_values_as_assigning_them_each_time() {
    // `sample_at_with` exists only to avoid re-traversing; it must not change a value.
    let graph = two_leaf_graph();
    let session = SampleSession::seeded(SEED);
    let ordinals = LeafOrdinals::new(&graph);

    for index in 0..32 {
        assert_eq!(
            graph.sample_at(&session, index).unwrap(),
            graph.sample_at_with(&session, index, &ordinals).unwrap()
        );
    }
}

#[test]
fn sample_next_walks_the_indices_from_zero() {
    let graph = two_leaf_graph();
    let mut session = SampleSession::seeded(SEED);
    let sequential: Vec<f64> = (0..16)
        .map(|_| graph.sample_next(&mut session).unwrap())
        .collect();

    let fixed = SampleSession::seeded(SEED);
    let by_index: Vec<f64> = (0..16)
        .map(|i| graph.sample_at(&fixed, i).unwrap())
        .collect();

    assert_eq!(sequential, by_index);
    assert_eq!(session.position(), 16);
}

#[test]
fn ordinals_from_a_different_graph_are_refused() {
    // A silent fallback to ambient entropy here would produce plausible values that no seed
    // reproduces, which is the failure this whole change exists to remove.
    let graph = two_leaf_graph();
    let foreign = LeafOrdinals::new(&Uncertain::<f64>::normal(0.0, 1.0));
    let session = SampleSession::seeded(SEED);

    assert!(graph.sample_at_with(&session, 0, &foreign).is_err());
}

#[test]
fn a_boolean_leaf_is_addressed_too() {
    let graph = UncertainBool::<f64>::bernoulli(0.5);
    let session = SampleSession::seeded(SEED);

    assert_eq!(LeafOrdinals::for_bool(&graph).len(), 1);

    let first: Vec<bool> = (0..32)
        .map(|i| graph.sample_at(&session, i).unwrap())
        .collect();
    let again: Vec<bool> = (0..32)
        .map(|i| graph.sample_at(&session, i).unwrap())
        .collect();
    assert_eq!(first, again);
    assert!(
        first.iter().any(|&b| b) && first.iter().any(|&b| !b),
        "a fair coin gave one face"
    );
}
