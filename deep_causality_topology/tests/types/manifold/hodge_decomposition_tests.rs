/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Integration tests for `Manifold::hodge_decompose` (Block H2).
//!
//! Covers the four documented failure modes and a pure-exact-1-form smoke test.
//! The full property-test surface (orthogonality identity, two-backend cross-check,
//! varying precision backends) lives in Block H3.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
    TopologyError, TopologyErrorEnum,
};

const TOL: f64 = 1e-8;

/// Build a `Manifold<LatticeComplex<D, f64>, f64>` with a unit-edge cubical
/// Regge geometry attached and arbitrary per-grade data baked in.
fn manifold_with_data<const D: usize>(
    lattice: LatticeComplex<D, f64>,
    data_vec: Vec<f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total = data_vec.len();
    let data = CausalTensor::new(data_vec, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn zero_manifold<const D: usize>(
    lattice: LatticeComplex<D, f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    manifold_with_data(lattice, vec![0.0; total])
}

fn unwrap_hodge_msg(err: &TopologyError) -> &str {
    match &err.0 {
        TopologyErrorEnum::HodgeDecompositionFailed(msg) => msg.as_str(),
        other => panic!("expected HodgeDecompositionFailed, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Input-validation failures
// ---------------------------------------------------------------------------

#[test]
fn hodge_decompose_rejects_grade_above_max_dim() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let m = zero_manifold(lattice);
    let field = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let err = m
        .hodge_decompose(&field, 99)
        .expect_err("grade 99 > max_dim 2 must fail");
    let msg = unwrap_hodge_msg(&err);
    assert!(msg.contains("grade 99"), "msg = {msg}");
    assert!(msg.contains("max_dim 2"), "msg = {msg}");
}

#[test]
fn hodge_decompose_rejects_field_with_wrong_length() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let m = zero_manifold(lattice);
    let expected = m.complex().num_cells(1);
    let wrong_len = expected + 7;
    let field = CausalTensor::new(vec![0.0; wrong_len], vec![wrong_len]).unwrap();
    let err = m
        .hodge_decompose(&field, 1)
        .expect_err("wrong field length must fail");
    let msg = unwrap_hodge_msg(&err);
    assert!(msg.contains(&format!("{}", wrong_len)), "msg = {msg}");
    assert!(msg.contains(&format!("{}", expected)), "msg = {msg}");
}

#[test]
fn hodge_decompose_rejects_manifold_without_metric() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let n1 = lattice.num_cells(1);
    // Build without metric via `from_cubical`.
    let m: Manifold<LatticeComplex<2, f64>, f64> = Manifold::from_cubical(lattice, data, 0);
    let field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let err = m
        .hodge_decompose(&field, 1)
        .expect_err("missing metric must fail");
    let msg = unwrap_hodge_msg(&err);
    assert!(msg.contains("no metric attached"), "msg = {msg}");
}

// ---------------------------------------------------------------------------
// Non-convergence failure
// ---------------------------------------------------------------------------

#[test]
fn hodge_decompose_reports_nonconvergence_under_artificially_low_iteration_cap() {
    // A 4x4 open lattice gives a 1-form CG large enough that iter_cap = 1 cannot
    // possibly converge for a non-trivial right-hand side.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(4);
    let n0 = lattice.num_cells(0);
    let n1 = lattice.num_cells(1);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();

    // Inject a non-trivial 0-form (vertex pattern) so that δω of d(this) is non-zero.
    let mut data_vec = vec![0.0_f64; total];
    for (i, slot) in data_vec.iter_mut().enumerate().take(n0) {
        *slot = ((i as f64) * 0.317).sin();
    }
    let m = manifold_with_data(lattice, data_vec);

    // Build ω = df by calling exterior_derivative on the embedded 0-form.
    let omega = m.exterior_derivative(0);
    assert_eq!(omega.as_slice().len(), n1);

    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-30_f64),
        max_iterations: Some(1),
    };
    let err = m
        .hodge_decompose_opts(&omega, 1, &opts)
        .expect_err("iter cap 1 with tiny tolerance must fail to converge");
    let msg = unwrap_hodge_msg(&err);
    assert!(msg.contains("did not converge"), "msg = {msg}");
    assert!(msg.contains("1 iterations"), "msg = {msg}");
}

// ---------------------------------------------------------------------------
// Smoke tests
// ---------------------------------------------------------------------------

#[test]
fn hodge_decompose_of_zero_field_returns_all_zero_components() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let m = zero_manifold(lattice);
    let n1 = m.complex().num_cells(1);
    let zero_field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let result = m.hodge_decompose(&zero_field, 1).expect("decompose zero");
    assert_eq!(result.grade(), 1);
    assert_eq!(result.exact().as_slice().len(), n1);
    assert_eq!(result.co_exact().as_slice().len(), n1);
    assert_eq!(result.harmonic().as_slice().len(), n1);
    for &x in result.exact().as_slice() {
        assert!(x.abs() < TOL);
    }
    for &x in result.co_exact().as_slice() {
        assert!(x.abs() < TOL);
    }
    for &x in result.harmonic().as_slice() {
        assert!(x.abs() < TOL);
    }
}

#[test]
fn hodge_decompose_of_pure_exact_1form_has_dominant_exact_component() {
    // Construct ω = df on an open 2D lattice via exterior_derivative(0).
    // A pure-exact 1-form on a trivially-topological lattice (Betti_1 = 0) must
    // decompose as α ≈ ω, β ≈ 0, h ≈ 0. We assert the qualitative inequality
    // ‖α‖ >> ‖β‖ and ‖α‖ >> ‖h‖. Exact analytic equality is checked in H3.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(4);
    let n0 = lattice.num_cells(0);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();

    let mut data_vec = vec![0.0_f64; total];
    for (i, slot) in data_vec.iter_mut().enumerate().take(n0) {
        *slot = (i as f64) * 1.5 - 3.0;
    }
    let m = manifold_with_data(lattice, data_vec);

    let omega = m.exterior_derivative(0);
    let omega_norm_sq: f64 = omega.as_slice().iter().map(|x| x * x).sum();
    assert!(omega_norm_sq > 0.0, "ω must be non-trivial for this test");

    let result = m.hodge_decompose(&omega, 1).expect("decompose ω = df");

    let alpha_norm_sq: f64 = result.exact().as_slice().iter().map(|x| x * x).sum();
    let beta_norm_sq: f64 = result.co_exact().as_slice().iter().map(|x| x * x).sum();
    let h_norm_sq: f64 = result.harmonic().as_slice().iter().map(|x| x * x).sum();

    assert!(
        alpha_norm_sq > beta_norm_sq,
        "alpha {} should dominate beta {} for pure-exact input",
        alpha_norm_sq,
        beta_norm_sq
    );
    assert!(
        alpha_norm_sq > h_norm_sq,
        "alpha {} should dominate harmonic {} for pure-exact input",
        alpha_norm_sq,
        h_norm_sq
    );
}

#[test]
fn hodge_decompose_returns_components_with_grade_k_length() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let m = zero_manifold(lattice);
    for k in 0..=2 {
        let n_k = m.complex().num_cells(k);
        let field = CausalTensor::new(vec![0.0; n_k], vec![n_k]).unwrap();
        let result = m.hodge_decompose(&field, k).unwrap_or_else(|e| {
            panic!("grade {} decompose failed: {:?}", k, e);
        });
        assert_eq!(result.grade(), k);
        assert_eq!(result.exact().as_slice().len(), n_k);
        assert_eq!(result.co_exact().as_slice().len(), n_k);
        assert_eq!(result.harmonic().as_slice().len(), n_k);
    }
}

#[test]
fn hodge_decompose_grade_zero_handles_alpha_branch_skip() {
    // At k = 0, the α step is skipped (no grade -1 forms). Just verify it runs
    // and returns zero α plus a well-formed β / h.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let n0 = lattice.num_cells(0);
    let m = zero_manifold(lattice);
    let field = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    let result = m.hodge_decompose(&field, 0).expect("k=0 decompose");
    assert_eq!(result.grade(), 0);
    for &x in result.exact().as_slice() {
        assert_eq!(x, 0.0);
    }
}

#[test]
fn hodge_decompose_grade_max_dim_handles_beta_branch_skip() {
    // At k = max_dim, the β step is skipped. Just verify it runs and returns
    // zero β plus a well-formed α / h.
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let n2 = lattice.num_cells(2);
    let m = zero_manifold(lattice);
    let field = CausalTensor::new(vec![0.0; n2], vec![n2]).unwrap();
    let result = m.hodge_decompose(&field, 2).expect("k=max_dim decompose");
    assert_eq!(result.grade(), 2);
    for &x in result.co_exact().as_slice() {
        assert_eq!(x, 0.0);
    }
}

#[test]
fn hodge_decompose_rejects_zero_tolerance() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let m = zero_manifold(lattice);
    let n1 = m.complex().num_cells(1);
    let field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let opts = HodgeDecomposeOptions {
        tolerance: Some(0.0_f64),
        max_iterations: Some(10),
    };
    let err = m
        .hodge_decompose_opts(&field, 1, &opts)
        .expect_err("zero tolerance must be rejected");
    let msg = unwrap_hodge_msg(&err);
    assert!(
        msg.contains("tolerance must be strictly positive"),
        "msg = {msg}"
    );
}

#[test]
fn hodge_decompose_rejects_negative_tolerance() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(2);
    let m = zero_manifold(lattice);
    let n1 = m.complex().num_cells(1);
    let field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let opts = HodgeDecomposeOptions {
        tolerance: Some(-1e-6_f64),
        max_iterations: Some(10),
    };
    let err = m
        .hodge_decompose_opts(&field, 1, &opts)
        .expect_err("negative tolerance must be rejected");
    let msg = unwrap_hodge_msg(&err);
    assert!(
        msg.contains("tolerance must be strictly positive"),
        "msg = {msg}"
    );
}

#[test]
fn hodge_decompose_options_default_yields_none_overrides() {
    let opts: HodgeDecomposeOptions<f64> = HodgeDecomposeOptions::default();
    assert!(opts.tolerance.is_none());
    assert!(opts.max_iterations.is_none());
}

#[test]
fn hodge_decompose_options_explicit_override_succeeds_with_loose_tolerance() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let m = zero_manifold(lattice);
    let n1 = m.complex().num_cells(1);
    let zero_field = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-3_f64),
        max_iterations: Some(50),
    };
    let result = m
        .hodge_decompose_opts(&zero_field, 1, &opts)
        .expect("zero field with loose tolerance");
    for &x in result.exact().as_slice() {
        assert!(x.abs() < TOL);
    }
}
