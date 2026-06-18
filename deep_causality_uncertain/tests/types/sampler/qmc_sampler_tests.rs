/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the Quasi-Monte-Carlo sampler and its opt-in batch estimators.
//!
//! Coverage note: `evaluate_node` carries defensive arms that no public `Uncertain` builder can
//! reach, so they are intentionally left uncovered rather than tested through contrived inputs:
//!   * the `PureOp` / `FmapOp` / `ApplyOp` / `BindOp` node variants — no `Uncertain` constructor
//!     produces them (only `from_root_node` could, and `BindOp` rejection is unit-tested in-crate);
//!   * the `UnsupportedTypeError` arms of every op (arithmetic / comparison / logical / function /
//!     negation / conditional) — the typed builders guarantee matching `SampledValue` variants, so
//!     a mismatch is unconstructible;
//!   * the `DoubleFloat` arms of `f64`-only ops (`ComparisonOp`, `FunctionOpF64`, `FunctionOpBool`)
//!     — `greater_than` / `map` / `map_to_bool` exist only on `Uncertain<f64>`, so their operand is
//!     never `Float106`;
//!   * the `NOR` and wrong-arity `LogicalOp` arms — the `&` / `|` / `!` / `^` operators emit only
//!     `And` / `Or` / `Not` / `XOR` at the correct arity.

use deep_causality_num::Float106;
use deep_causality_uncertain::{QmcSampler, Uncertain, UncertainError, seed_sampler};
use rusty_fork::rusty_fork_test;

fn f106(x: f64) -> Float106 {
    Float106::from_f64(x)
}

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

#[test]
fn test_over_dimension_tree_is_rejected() {
    // MAX_SOBOL_DIM is 16; a sum of 17 independent normals needs 17 stochastic dimensions.
    let mut u = Uncertain::normal(0.0, 1.0);
    for _ in 0..16 {
        u = u + Uncertain::normal(0.0, 1.0);
    }
    let err = QmcSampler::new(&u, None).unwrap_err();
    assert!(matches!(err, UncertainError::SamplingError(_)));
}

#[test]
fn test_dimension_assignment_walks_every_static_node_kind() {
    // One graph touching the arithmetic, negation, comparison, logical, function, and conditional
    // arms of `assign_dimensions` / `collect_stochastic_leaves`. Both conditional branches share
    // the exact same leaves, so the branch-divergence guard accepts it.
    let n = Uncertain::normal(0.0, 1.0);
    let uni = Uncertain::uniform(0.0, 1.0);
    // A rich numeric branch: negation, arithmetic, an f64 `map` (FunctionOpF64).
    let branch = ((-n.clone()) + uni.clone()).map(|x| x + 1.0);
    // A boolean condition mixing a comparison and a logical `&` over the same leaves.
    let cond = n.clone().greater_than(0.0) & uni.clone().less_than(0.5);
    let u = Uncertain::conditional(cond, branch.clone(), branch);

    let sampler = QmcSampler::new(&u, None).expect("static structure with shared leaves is valid");
    // Two distinct stochastic leaves (the normal and the uniform).
    assert_eq!(sampler.dimension(), 2);
}

#[test]
fn test_f106_dimension_assignment() {
    let u = Uncertain::<Float106>::normal(f106(0.0), f106(1.0))
        + Uncertain::<Float106>::uniform(f106(0.0), f106(1.0));
    let sampler = QmcSampler::new(&u, None).unwrap();
    assert_eq!(sampler.dimension(), 2);
}

#[test]
fn test_branch_leaf_collection_walks_nested_conditional_and_bool_logical() {
    // A numeric branch containing a *nested* conditional whose boolean condition mixes a
    // comparison and a logical `&` over a Bernoulli leaf. `collect_stochastic_leaves` recurses
    // through the conditional / comparison / logical / bool-leaf arms while validating the outer
    // conditional's branches. Both outer branches share the identical subtree, so it is accepted.
    let b = Uncertain::<bool>::bernoulli(0.5);
    let n = Uncertain::normal(0.0, 1.0);
    let inner_cond = n.clone().greater_than(0.0) & b;
    let inner = Uncertain::conditional(inner_cond, n.clone(), n);
    let outer_cond = Uncertain::<bool>::bernoulli(0.5);
    let u = Uncertain::conditional(outer_cond, inner.clone(), inner);

    assert!(QmcSampler::new(&u, None).is_ok());
}

#[test]
fn test_f106_branch_leaf_collection() {
    // The `Float106` arm of `collect_stochastic_leaves`, reached by validating an f106 conditional.
    let cond = Uncertain::<bool>::bernoulli(0.5);
    let shared = Uncertain::<Float106>::normal(f106(0.0), f106(1.0));
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

// --- evaluate_node arm coverage (each draws through the QMC sampler) ---

#[test]
fn test_qmc_samples_f64_uniform_leaf() {
    let u = Uncertain::uniform(2.0, 4.0);
    let sampler = QmcSampler::new(&u, None).unwrap();
    let v = u.sample_with_index_qmc(0, &sampler).unwrap();
    assert!((2.0..4.0).contains(&v), "uniform draw {v} out of [2,4)");
}

#[test]
fn test_qmc_samples_bool_bernoulli_leaf() {
    let u = Uncertain::<bool>::bernoulli(1.0); // certainly true
    let sampler = QmcSampler::new(&u, None).unwrap();
    assert!(u.sample_with_index_qmc(0, &sampler).unwrap());
}

#[test]
fn test_qmc_samples_f64_negation_and_map_and_function_bool() {
    let base = Uncertain::normal(5.0, 0.0); // a degenerate normal: always its mean
    let neg = -base.clone();
    let neg_sampler = QmcSampler::new(&neg, None).unwrap();
    let nv = neg.sample_with_index_qmc(0, &neg_sampler).unwrap();
    assert!((nv + 5.0).abs() < 1e-9, "negation gave {nv}");

    let mapped = base.clone().map(|x| x * 2.0);
    let mapped_sampler = QmcSampler::new(&mapped, None).unwrap();
    let mv = mapped.sample_with_index_qmc(0, &mapped_sampler).unwrap();
    assert!((mv - 10.0).abs() < 1e-9, "map gave {mv}");

    let to_bool = base.map_to_bool(|x| x > 0.0);
    let tb_sampler = QmcSampler::new(&to_bool, None).unwrap();
    assert!(to_bool.sample_with_index_qmc(0, &tb_sampler).unwrap());
}

#[test]
fn test_qmc_samples_comparison_and_logical_ops() {
    let n = Uncertain::normal(1.0, 0.0); // always 1.0
    let gt = n.greater_than(0.0);
    let s = QmcSampler::new(&gt, None).unwrap();
    assert!(gt.sample_with_index_qmc(0, &s).unwrap());

    let a = Uncertain::<bool>::bernoulli(1.0);
    let b = Uncertain::<bool>::bernoulli(0.0);
    let and = a.clone() & b.clone();
    let or = a.clone() | b.clone();
    let xor = a.clone() ^ b.clone();
    let not = !a;
    for (u, want) in [(and, false), (or, true), (xor, true), (not, false)] {
        let s = QmcSampler::new(&u, None).unwrap();
        assert_eq!(u.sample_with_index_qmc(0, &s).unwrap(), want);
    }
}

#[test]
fn test_qmc_samples_conditional_both_branches() {
    // Shared leaves so the static-structure guard accepts the conditional; sampling across many
    // indices drives the condition both ways, exercising both branch arms of `evaluate_node`.
    let cond = Uncertain::<bool>::bernoulli(0.5);
    let shared = Uncertain::normal(0.0, 1.0);
    let u = Uncertain::conditional(cond, shared.clone(), shared);
    let sampler = QmcSampler::new(&u, None).unwrap();
    for i in 0..64 {
        assert!(u.sample_with_index_qmc(i, &sampler).unwrap().is_finite());
    }
}

#[test]
fn test_qmc_samples_f106_distributions_arithmetic_and_negation() {
    let normal = Uncertain::<Float106>::normal(f106(3.0), f106(0.0)); // always 3.0
    let uniform = Uncertain::<Float106>::uniform(f106(0.0), f106(2.0));

    let sum = normal.clone() + uniform.clone();
    let s = QmcSampler::new(&sum, None).unwrap();
    let v = sum.sample_with_index_qmc(0, &s).unwrap();
    assert!(v.to_f64().is_finite() && v.to_f64() >= 3.0 && v.to_f64() < 5.0);

    let neg = -normal;
    let s2 = QmcSampler::new(&neg, None).unwrap();
    let nv = neg.sample_with_index_qmc(0, &s2).unwrap();
    assert!((nv.to_f64() + 3.0).abs() < 1e-9, "f106 negation gave {}", nv.to_f64());

    let s3 = QmcSampler::new(&uniform, None).unwrap();
    let uv = uniform.sample_with_index_qmc(0, &s3).unwrap();
    assert!((0.0..2.0).contains(&uv.to_f64()));
}

#[test]
fn test_qmc_memoizes_a_shared_leaf() {
    // `n` feeds both operands of the sum, so the second visit to its node id hits the per-sample
    // memoization cache (one draw per shared leaf, matching SequentialSampler semantics).
    let n = Uncertain::normal(2.0, 0.0); // degenerate: always 2.0
    let u = n.clone() + n;
    let sampler = QmcSampler::new(&u, None).unwrap();
    let v = u.sample_with_index_qmc(0, &sampler).unwrap();
    assert!((v - 4.0).abs() < 1e-9, "shared leaf summed to {v}, expected 4.0");
}

#[test]
fn test_qmc_samples_point_leaf_inside_a_tree() {
    // A `point` leaf combined with a stochastic leaf exercises the Point distribution arm.
    let u = Uncertain::<f64>::point(10.0) + Uncertain::normal(0.0, 0.0);
    let sampler = QmcSampler::new(&u, None).unwrap();
    let v = u.sample_with_index_qmc(0, &sampler).unwrap();
    assert!((v - 10.0).abs() < 1e-9);
}

#[test]
fn test_qmc_coordinate_errors_on_a_foreign_leaf() {
    // A sampler built for tree A has no dimension for tree B's distinct stochastic leaf, so
    // sampling B with A's sampler hits the `coordinate` "no assigned dimension" guard.
    let a = Uncertain::normal(0.0, 1.0);
    let b = Uncertain::normal(0.0, 1.0); // a different node id, not in A's dims
    let sampler_a = QmcSampler::new(&a, None).unwrap();
    let err = b.sample_with_index_qmc(0, &sampler_a).unwrap_err();
    assert!(matches!(err, UncertainError::SamplingError(_)));
}

} // rusty_fork_test!
