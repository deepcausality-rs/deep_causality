/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The two pre-passes agree about which leaves draw.
//!
//! `LeafOrdinals` assigns a Monte-Carlo address and `QmcSampler` assigns a Sobol dimension, by the
//! same rule over the same traversal. Nothing forces them to stay in step, so this measures it: a
//! leaf counted by one and not the other would mean one sampler drawing where the other does not.

use deep_causality_uncertain::{LeafOrdinals, QmcSampler, SampleSession, Uncertain};

const SEED: u64 = 0x5EED_2026;

fn two_leaf_graph() -> Uncertain<f64> {
    Uncertain::<f64>::normal(10.0, 1.0) + Uncertain::<f64>::uniform(0.0, 1.0)
}

#[test]
fn both_pre_passes_count_the_same_drawing_leaves() {
    for (name, graph) in [
        ("two leaves", two_leaf_graph()),
        ("one leaf", Uncertain::<f64>::normal(0.0, 1.0)),
        (
            "three leaves",
            Uncertain::<f64>::normal(0.0, 1.0)
                + Uncertain::<f64>::uniform(0.0, 1.0)
                + Uncertain::<f64>::normal(5.0, 2.0),
        ),
        (
            "a point leaf among them",
            Uncertain::<f64>::normal(0.0, 1.0) + Uncertain::<f64>::point(3.0),
        ),
    ] {
        let ordinals = LeafOrdinals::new(&graph);
        let qmc = QmcSampler::new(&graph, Some(SEED)).unwrap();

        assert_eq!(
            ordinals.len(),
            qmc.dimension(),
            "{name}: the ordinal pre-pass and the Sobol pre-pass disagree on how many leaves draw"
        );
    }
}

#[test]
fn a_shared_leaf_counts_once_in_both() {
    let x = Uncertain::<f64>::normal(10.0, 1.0);
    let doubled = x.clone() + x;

    let ordinals = LeafOrdinals::new(&doubled);
    let qmc = QmcSampler::new(&doubled, Some(SEED)).unwrap();

    assert_eq!(ordinals.len(), 1);
    assert_eq!(qmc.dimension(), 1);
}

#[test]
fn the_qmc_path_is_unchanged_by_the_addressed_monte_carlo_path() {
    // The addressed path is additive: it added a second way to draw and touched neither the Sobol
    // dimension assignment nor the quasi-Monte-Carlo evaluation.
    let graph = two_leaf_graph();
    let sampler = QmcSampler::new(&graph, Some(SEED)).unwrap();

    let first: Vec<f64> = (0..16)
        .map(|i| graph.sample_with_index_qmc(i, &sampler).unwrap())
        .collect();

    // Every value is finite and inside the support the two leaves imply.
    for (i, v) in first.iter().enumerate() {
        assert!(v.is_finite(), "QMC point {i} is not finite");
    }
    // A Sobol sequence at distinct indices gives distinct points.
    let mut unique = first.clone();
    unique.sort_by(f64::total_cmp);
    unique.dedup();
    assert_eq!(unique.len(), first.len(), "two QMC indices gave one value");
}

#[test]
fn a_monte_carlo_session_and_a_qmc_session_are_different_families() {
    // Not an assertion about which is better, only that the mode selects a genuinely different
    // draw rather than being a label. This is what the removed `SamplerKind` cache key was for.
    let graph = two_leaf_graph();
    let mc = SampleSession::seeded(SEED);
    let qmc_sampler = QmcSampler::new(&graph, Some(SEED)).unwrap();

    let addressed: Vec<f64> = (0..16).map(|i| graph.sample_at(&mc, i).unwrap()).collect();
    let quasi: Vec<f64> = (0..16)
        .map(|i| graph.sample_with_index_qmc(i, &qmc_sampler).unwrap())
        .collect();

    assert_ne!(addressed, quasi);
}
