/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the Quasi-Monte-Carlo sampler and its opt-in batch estimators.

use deep_causality_uncertain::{QmcSampler, Uncertain, UncertainError, seed_sampler};
use rusty_fork::rusty_fork_test;

// =============================================================================
// Dimension assignment and static-structure validation (no global state)
// =============================================================================

#[test]
fn test_dimension_equals_stochastic_leaf_count() {
    let u = Uncertain::normal(0.0, 1.0) + Uncertain::normal(0.0, 1.0) + Uncertain::normal(0.0, 1.0);
    let sampler = QmcSampler::new(&u, None).unwrap();
    assert_eq!(sampler.dimension(), 3);
}

#[test]
fn test_point_only_tree_has_zero_dimensions() {
    let u = Uncertain::<f64>::point(5.0);
    let sampler = QmcSampler::new(&u, None).unwrap();
    assert_eq!(sampler.dimension(), 0);
    // A point tree samples to its constant regardless of index.
    let v = u.sample_with_index_qmc(0, &sampler).unwrap();
    assert_eq!(v, 5.0);
}

#[test]
fn test_same_index_is_reproducible_distinct_index_differs() {
    let u = Uncertain::normal(0.0, 1.0) + Uncertain::uniform(0.0, 1.0);
    let sampler = QmcSampler::new(&u, None).unwrap();
    let a0 = u.sample_with_index_qmc(4, &sampler).unwrap();
    let a0_again = u.sample_with_index_qmc(4, &sampler).unwrap();
    let a1 = u.sample_with_index_qmc(5, &sampler).unwrap();
    assert_eq!(a0, a0_again, "same index must reproduce the same sample");
    assert_ne!(a0, a1, "distinct indices must give distinct Sobol points");
}

// (BindOp rejection is unit-tested in-crate via `from_root_node`; no `Uncertain` builder produces
// a `BindOp`, so it cannot be constructed through the public `QmcSampler::new(&Uncertain)` here.)

#[test]
fn test_reject_branch_divergent_conditional() {
    let cond = Uncertain::<bool>::bernoulli(0.5);
    let if_true = Uncertain::normal(0.0, 1.0);
    let if_false = Uncertain::normal(5.0, 1.0); // different leaf → divergent
    let u = Uncertain::conditional(cond, if_true, if_false);

    let err = QmcSampler::new(&u, None).unwrap_err();
    assert!(matches!(err, UncertainError::SamplingError(_)));
}

#[test]
fn test_accept_conditional_sharing_leaves() {
    // Both branches reference the same leaf node → identical distribution sets → accepted.
    let cond = Uncertain::<bool>::bernoulli(0.5);
    let shared = Uncertain::normal(0.0, 1.0);
    let u = Uncertain::conditional(cond, shared.clone(), shared);
    assert!(QmcSampler::new(&u, None).is_ok());
}

// =============================================================================
// Batch estimators and cache (process-isolated)
// =============================================================================

rusty_fork_test! {

#[test]
fn test_expected_value_qmc_matches_mean() {
    let u = Uncertain::normal(5.0, 2.0);
    let mean = u.expected_value_qmc(512, 0xABCD).unwrap();
    assert!((mean - 5.0).abs() < 0.2, "QMC mean {mean} not near 5.0");
}

#[test]
fn test_qmc_reproducible_with_same_seed() {
    let u = Uncertain::normal(1.0, 3.0);
    let a = u.expected_value_qmc(256, 42).unwrap();
    let b = u.expected_value_qmc(256, 42).unwrap();
    assert_eq!(a, b);
}

#[test]
fn test_qmc_converges_faster_than_mc() {
    // Uniform(0,1) has true mean 0.5. QMC error is far below MC at equal N.
    let u = Uncertain::uniform(0.0, 1.0);
    const N: usize = 4096;

    seed_sampler(42);
    let mc = u.expected_value(N).unwrap();
    let qmc = u.expected_value_qmc(N, 42).unwrap();

    let mc_err = (mc - 0.5).abs();
    let qmc_err = (qmc - 0.5).abs();
    assert!(
        qmc_err < mc_err,
        "QMC error {qmc_err:e} not below MC error {mc_err:e}"
    );
}

#[test]
fn test_standard_deviation_qmc_is_nonzero() {
    let u = Uncertain::normal(0.0, 1.0);
    let sd = u.standard_deviation_qmc(512, 7).unwrap();
    assert!(sd > 0.0, "QMC standard deviation should be a positive estimate");
    assert!((sd - 1.0).abs() < 0.3, "QMC sd {sd} not near 1.0");
}

#[test]
fn test_estimate_probability_qmc() {
    let u = Uncertain::<bool>::bernoulli(0.3);
    let p = u.estimate_probability_qmc(1024, 99).unwrap();
    assert!((p - 0.3).abs() < 0.03, "QMC probability {p} not near 0.3");
}

#[test]
fn test_mc_and_qmc_caches_do_not_collide() {
    seed_sampler(7);
    let u = Uncertain::normal(0.0, 10.0);
    let sampler = QmcSampler::new(&u, Some(123)).unwrap();

    let mc = u.sample_with_index(3).unwrap();
    let qmc = u.sample_with_index_qmc(3, &sampler).unwrap();

    // Each sampler re-reads its own cached value; neither overwrites the other.
    assert_eq!(mc, u.sample_with_index(3).unwrap());
    assert_eq!(qmc, u.sample_with_index_qmc(3, &sampler).unwrap());
    // The two draws are independent and overwhelmingly distinct.
    assert_ne!(mc, qmc);
}

#[test]
fn test_qmc_batch_rejects_dynamic_tree() {
    let cond = Uncertain::<bool>::bernoulli(0.5);
    let u = Uncertain::conditional(cond, Uncertain::normal(0.0, 1.0), Uncertain::normal(9.0, 1.0));
    assert!(u.expected_value_qmc(64, 1).is_err());
}

} // rusty_fork_test!
