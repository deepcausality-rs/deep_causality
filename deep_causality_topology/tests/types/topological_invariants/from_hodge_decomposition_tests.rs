/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `HodgeDecomposition::topological_invariants` (Block B1a).
//!
//! Covers the property tests that survive the B1 topology/physics split:
//! - **Hodge orthogonality.** `exact_l2² + co_exact_l2² + harmonic_l2² = ‖α + β + h‖²`.
//! - **Betti number consistency.** Reports the manifold's Betti numbers verbatim
//!   in the `[β_0, β_1, β_2, β_3]` slots, zero-padded beyond `max_dim`.
//! - **Reproducibility.** Same input produces bit-identical output.
//! - **Sign-flip invariance.** Flipping the sign of the input scalar 0-form
//!   produces a decomposition whose L2 norms are unchanged (and whose Betti
//!   numbers depend only on the complex, not the field).
//!
//! ## Why no translation-invariance test
//!
//! The note's B1 design lists "Translation invariance" as a property test on
//! `LatticeComplex<3>` with unit-edge geometry. A faithful test requires
//! periodic boundaries (a torus); on an open lattice, translation truncates
//! field values at the boundary and is therefore not invariant. But the
//! matrix-free CG in `Manifold::hodge_decompose` does not project out the
//! harmonic kernel of `Δ_k` for `k > 0`, so on a torus where `β_k > 0` the
//! β-step CG is singular and does not converge (design.md Risk 1). Landing
//! a translation-invariance test on a torus requires upgrading the CG to
//! handle harmonic-kernel projection at all grades, which is out of scope
//! for B1a.
//!
//! Sign-flip invariance is the closest geometric invariant we can test
//! cleanly today: it does not need periodicity, exercises the same algebraic
//! property of the L2 norm, and runs in finite time.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold, TopologicalInvariants,
};

fn unit_manifold_2d_open(side_cells: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(side_cells);
    manifold_with_scalar_field(lattice)
}

fn unit_manifold_3d_open(side_cells: usize) -> Manifold<LatticeComplex<3, f64>, f64> {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(side_cells);
    manifold_with_scalar_field(lattice)
}

fn manifold_with_scalar_field<const D: usize>(
    lattice: LatticeComplex<D, f64>,
) -> Manifold<LatticeComplex<D, f64>, f64> {
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let n0 = lattice.num_cells(0);
    let mut data_vec = vec![0.0_f64; total];
    for (i, slot) in data_vec.iter_mut().enumerate().take(n0) {
        *slot = ((i as f64) * 0.317).sin();
    }
    let data = CausalTensor::new(data_vec, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn extract<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
) -> TopologicalInvariants<f64> {
    let omega = manifold.exterior_derivative(0);
    let decomposition = manifold
        .hodge_decompose(&omega, 1)
        .expect("decompose ω = df");
    decomposition
        .topological_invariants(manifold)
        .expect("extract invariants")
}

fn norm_sq(v: &[f64]) -> f64 {
    v.iter().map(|x| x * x).sum()
}

// ---------------------------------------------------------------------------
// Hodge orthogonality identity
// ---------------------------------------------------------------------------

#[test]
fn orthogonality_identity_holds_on_2d_open_lattice() {
    let m = unit_manifold_2d_open(3);
    let omega = m.exterior_derivative(0);
    let decomposition = m.hodge_decompose(&omega, 1).unwrap();
    let inv = decomposition.topological_invariants(&m).unwrap();

    // Reconstruct ‖α + β + h‖² (which equals ‖ω‖² because that's exactly what
    // the decomposition produces) and compare against the sum of squared
    // component norms reported by the extractor.
    let recon: Vec<f64> = decomposition
        .exact()
        .as_slice()
        .iter()
        .zip(decomposition.co_exact().as_slice())
        .zip(decomposition.harmonic().as_slice())
        .map(|((a, b), h)| a + b + h)
        .collect();
    let recon_norm_sq = norm_sq(&recon);

    let sum_sq = inv.exact_l2_norm().powi(2)
        + inv.co_exact_l2_norm().powi(2)
        + inv.harmonic_l2_norm().powi(2);
    let rel = (sum_sq - recon_norm_sq).abs() / recon_norm_sq.max(1.0);
    assert!(
        rel < 1e-6,
        "rel = {rel}, sum_sq = {sum_sq}, recon = {recon_norm_sq}"
    );
}

#[test]
fn orthogonality_identity_holds_on_3d_open_lattice() {
    let m = unit_manifold_3d_open(2);
    let omega = m.exterior_derivative(0);
    let decomposition = m.hodge_decompose(&omega, 1).unwrap();
    let inv = decomposition.topological_invariants(&m).unwrap();

    let recon: Vec<f64> = decomposition
        .exact()
        .as_slice()
        .iter()
        .zip(decomposition.co_exact().as_slice())
        .zip(decomposition.harmonic().as_slice())
        .map(|((a, b), h)| a + b + h)
        .collect();
    let recon_norm_sq = norm_sq(&recon);

    let sum_sq = inv.exact_l2_norm().powi(2)
        + inv.co_exact_l2_norm().powi(2)
        + inv.harmonic_l2_norm().powi(2);
    let rel = (sum_sq - recon_norm_sq).abs() / recon_norm_sq.max(1.0);
    assert!(rel < 1e-6, "rel = {rel}");
}

#[test]
fn extracted_l2_norms_match_direct_component_norms() {
    let m = unit_manifold_2d_open(3);
    let omega = m.exterior_derivative(0);
    let decomposition = m.hodge_decompose(&omega, 1).unwrap();
    let inv = decomposition.topological_invariants(&m).unwrap();

    let direct_exact = norm_sq(decomposition.exact().as_slice()).sqrt();
    let direct_co = norm_sq(decomposition.co_exact().as_slice()).sqrt();
    let direct_h = norm_sq(decomposition.harmonic().as_slice()).sqrt();

    assert!((inv.exact_l2_norm() - direct_exact).abs() < 1e-15);
    assert!((inv.co_exact_l2_norm() - direct_co).abs() < 1e-15);
    assert!((inv.harmonic_l2_norm() - direct_h).abs() < 1e-15);
}

// ---------------------------------------------------------------------------
// Betti number consistency
// ---------------------------------------------------------------------------

#[test]
fn betti_numbers_match_manifold_for_2d_open_lattice() {
    let m = unit_manifold_2d_open(3);
    let inv = extract(&m);
    assert_eq!(inv.betti_numbers()[0], m.complex().betti_number(0));
    assert_eq!(inv.betti_numbers()[1], m.complex().betti_number(1));
    assert_eq!(inv.betti_numbers()[2], m.complex().betti_number(2));
    // 2D complex has max_dim = 2; the [3] slot is zero-padded.
    assert_eq!(inv.betti_numbers()[3], 0);
}

#[test]
fn betti_numbers_match_manifold_for_3d_open_lattice() {
    let m = unit_manifold_3d_open(2);
    let inv = extract(&m);
    for k in 0..=3 {
        assert_eq!(inv.betti_numbers()[k], m.complex().betti_number(k));
    }
}

#[test]
fn betti_numbers_for_2d_open_lattice_are_contractible_topology() {
    // Open 2D lattice is contractible: β_0 = 1, β_1 = β_2 = β_3 = 0.
    let m = unit_manifold_2d_open(3);
    let inv = extract(&m);
    assert_eq!(inv.betti_numbers(), [1, 0, 0, 0]);
}

#[test]
fn betti_numbers_for_3d_open_lattice_are_contractible_topology() {
    let m = unit_manifold_3d_open(2);
    let inv = extract(&m);
    assert_eq!(inv.betti_numbers(), [1, 0, 0, 0]);
}

#[test]
fn betti_numbers_array_is_zero_padded_beyond_max_dim() {
    let m = unit_manifold_2d_open(3);
    let inv = extract(&m);
    assert_eq!(inv.betti_numbers()[3], 0);
}

// ---------------------------------------------------------------------------
// Reproducibility
// ---------------------------------------------------------------------------

#[test]
fn extraction_is_reproducible_across_calls() {
    let m = unit_manifold_2d_open(3);
    let omega = m.exterior_derivative(0);
    let decomposition = m.hodge_decompose(&omega, 1).unwrap();
    let inv_1 = decomposition.topological_invariants(&m).unwrap();
    let inv_2 = decomposition.topological_invariants(&m).unwrap();
    assert_eq!(inv_1, inv_2);
}

#[test]
fn extraction_at_grade_zero_succeeds() {
    let m = unit_manifold_2d_open(3);
    let n0 = m.complex().num_cells(0);
    let field = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    let decomposition = m.hodge_decompose(&field, 0).unwrap();
    let inv = decomposition.topological_invariants(&m).unwrap();
    assert_eq!(inv.betti_numbers()[0], 1);
}

#[test]
fn l2_norms_are_non_negative_for_arbitrary_input() {
    let m = unit_manifold_2d_open(3);
    let inv = extract(&m);
    assert!(inv.exact_l2_norm() >= 0.0);
    assert!(inv.co_exact_l2_norm() >= 0.0);
    assert!(inv.harmonic_l2_norm() >= 0.0);
}

// ---------------------------------------------------------------------------
// Sign-flip invariance (the in-scope replacement for translation invariance)
// ---------------------------------------------------------------------------

#[test]
fn flipping_input_field_sign_preserves_betti_and_l2_norms() {
    let lattice_a: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    let lattice_b: LatticeComplex<2, f64> = LatticeComplex::square_open(3);

    let total = (0..=2).map(|k| lattice_a.num_cells(k)).sum::<usize>();
    let n0 = lattice_a.num_cells(0);
    let mut data_a = vec![0.0_f64; total];
    let mut data_b = vec![0.0_f64; total];
    for i in 0..n0 {
        let v = ((i as f64) * 0.31).sin();
        data_a[i] = v;
        data_b[i] = -v;
    }
    let m_a = Manifold::from_cubical_with_metric(
        lattice_a,
        CausalTensor::new(data_a, vec![total]).unwrap(),
        CubicalReggeGeometry::<2, f64>::unit(),
        0,
    );
    let m_b = Manifold::from_cubical_with_metric(
        lattice_b,
        CausalTensor::new(data_b, vec![total]).unwrap(),
        CubicalReggeGeometry::<2, f64>::unit(),
        0,
    );

    let inv_a = extract(&m_a);
    let inv_b = extract(&m_b);

    assert_eq!(inv_a.betti_numbers(), inv_b.betti_numbers());

    let diff_exact = (inv_a.exact_l2_norm() - inv_b.exact_l2_norm()).abs();
    let diff_co = (inv_a.co_exact_l2_norm() - inv_b.co_exact_l2_norm()).abs();
    let diff_harm = (inv_a.harmonic_l2_norm() - inv_b.harmonic_l2_norm()).abs();
    assert!(
        diff_exact < 1e-6 && diff_co < 1e-6 && diff_harm < 1e-6,
        "sign-flip invariance violated: Δα={diff_exact}, Δβ={diff_co}, Δh={diff_harm}"
    );
}
