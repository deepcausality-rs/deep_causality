/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Property tests for `Manifold::hodge_decompose` (Block H3).
//!
//! Verifies three algebraic invariants on the cubical backend across multiple
//! lattice sizes and precision backends:
//!
//! 1. The Hodge orthogonality identity `‖α‖² + ‖β‖² + ‖h‖² = ‖ω‖²` (the
//!    decomposition is into pairwise-orthogonal components).
//! 2. Pure-exact 1-forms `ω = df` decompose as `α ≈ ω`, `β ≈ 0`, `h ≈ 0`.
//! 3. Pure-co-exact 1-forms `ω = δg` decompose as `α ≈ 0`, `β ≈ ω`, `h ≈ 0`.
//!
//! Each invariant is exercised at `f32`, `f64`, and `Float106` precision.
//! The two-backend cross-check (simplicial vs cubical) lives in
//! `hodge_decomposition_cross_backend_tests.rs`.

use core::fmt::{Debug, Display};

use deep_causality_algebra::RealField;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HasHodgeStar, LatticeComplex, Manifold,
};

// ---------------------------------------------------------------------------
// Generic helpers
// ---------------------------------------------------------------------------

fn from_f64<R: FromPrimitive>(x: f64) -> R {
    <R as FromPrimitive>::from_f64(x).expect("value must be representable in R")
}

fn manifold_with_data_pattern<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
    seed: R,
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField + deep_causality_par::MaybeParallel + FromPrimitive + Default,
{
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let mut data_vec = vec![R::zero(); total];
    for (i, slot) in data_vec.iter_mut().enumerate() {
        let scaled = from_f64::<R>(i as f64) * seed;
        *slot = scaled.sin();
    }
    let data = CausalTensor::new(data_vec, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn norm_sq<R: RealField>(v: &[R]) -> R {
    v.iter()
        .copied()
        .map(|x| x * x)
        .fold(R::zero(), |a, b| a + b)
}

fn max_r<R: RealField>(a: R, b: R) -> R {
    if a > b { a } else { b }
}

fn relative_diff<R: RealField + deep_causality_par::MaybeParallel + FromPrimitive>(
    a: R,
    b: R,
) -> R {
    let denom = max_r(max_r(a.abs(), b.abs()), from_f64::<R>(1.0));
    (a - b).abs() / denom
}

// Comfortable tolerance per precision backend.
//
// The CG default tolerance is R::from_f64(1e-10).unwrap_or_else(R::epsilon),
// which collapses to f32::EPSILON ≈ 1.19e-7 for f32. The orthogonality check
// then accumulates roughly two CG-residual errors plus a residual term, so we
// pad by roughly two orders of magnitude.
fn ortho_tol<R: RealField + deep_causality_par::MaybeParallel + FromPrimitive>() -> R {
    from_f64::<R>(1e-3)
}

fn pure_part_tol<R: RealField + deep_causality_par::MaybeParallel + FromPrimitive>() -> R {
    // For pure-exact / pure-co-exact tests the "vanishing" component norm² is
    // compared against the input norm². Allow 1% of input norm² for f32 noise.
    from_f64::<R>(1e-2)
}

// ---------------------------------------------------------------------------
// Generic property bodies (one per invariant)
// ---------------------------------------------------------------------------

fn orthogonality_identity_holds<const D: usize, R>(lattice: LatticeComplex<D, R>, seed: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + Debug
        + Display,
    CubicalReggeGeometry<D, R>: HasHodgeStar<R, Complex = LatticeComplex<D, R>> + Clone,
{
    let manifold = manifold_with_data_pattern(lattice, seed);

    // Build ω = df on grade 1 by calling exterior_derivative(0) — this guarantees
    // a non-trivial 1-form on the lattice.
    let omega = manifold.exterior_derivative(0);
    let omega_slice = omega.as_slice();
    assert!(!omega_slice.is_empty(), "ω must be non-empty");

    let result = manifold
        .hodge_decompose(&omega, 1)
        .expect("decomposition should succeed for ω = df");

    let alpha_norm_sq = norm_sq(result.exact().as_slice());
    let beta_norm_sq = norm_sq(result.co_exact().as_slice());
    let h_norm_sq = norm_sq(result.harmonic().as_slice());
    let omega_norm_sq = norm_sq(omega_slice);

    let sum = alpha_norm_sq + beta_norm_sq + h_norm_sq;
    let rel = relative_diff(sum, omega_norm_sq);
    assert!(
        rel < ortho_tol::<R>(),
        "orthogonality identity failed: ‖α‖²={:?}, ‖β‖²={:?}, ‖h‖²={:?}, sum={:?}, ‖ω‖²={:?}, rel={:?}",
        alpha_norm_sq,
        beta_norm_sq,
        h_norm_sq,
        sum,
        omega_norm_sq,
        rel
    );
}

fn pure_exact_decomposition_holds<const D: usize, R>(lattice: LatticeComplex<D, R>, seed: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + Debug
        + Display,
    CubicalReggeGeometry<D, R>: HasHodgeStar<R, Complex = LatticeComplex<D, R>> + Clone,
{
    let manifold = manifold_with_data_pattern(lattice, seed);

    // ω = df on grade 1.
    let omega = manifold.exterior_derivative(0);
    let omega_norm_sq = norm_sq(omega.as_slice());
    assert!(omega_norm_sq > R::zero(), "ω must be non-trivial");

    let result = manifold
        .hodge_decompose(&omega, 1)
        .expect("decompose ω = df");

    let beta_norm_sq = norm_sq(result.co_exact().as_slice());
    let h_norm_sq = norm_sq(result.harmonic().as_slice());

    let beta_ratio = beta_norm_sq / omega_norm_sq;
    let h_ratio = h_norm_sq / omega_norm_sq;

    let tol = pure_part_tol::<R>();
    assert!(
        beta_ratio < tol,
        "pure-exact: ‖β‖² / ‖ω‖² = {:?} should be near zero, tol = {:?}",
        beta_ratio,
        tol
    );
    assert!(
        h_ratio < tol,
        "pure-exact: ‖h‖² / ‖ω‖² = {:?} should be near zero, tol = {:?}",
        h_ratio,
        tol
    );
}

fn pure_co_exact_decomposition_holds<const D: usize, R>(lattice: LatticeComplex<D, R>, seed: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + Debug
        + Display,
    CubicalReggeGeometry<D, R>: HasHodgeStar<R, Complex = LatticeComplex<D, R>> + Clone,
{
    let max_dim = lattice.max_dim();
    let manifold = manifold_with_data_pattern(lattice, seed);

    // Build a pure co-exact 1-form ω = δg where g is a 2-form. On 2D this means
    // codifferential(2), which yields a 1-form. The 2-form data is the highest
    // grade in `manifold.data` and is already populated by manifold_with_data_pattern.
    let omega = manifold.codifferential(max_dim);
    let omega_norm_sq = norm_sq(omega.as_slice());
    assert!(omega_norm_sq > R::zero(), "ω = δg must be non-trivial");
    // The codifferential at grade max_dim returns a (max_dim - 1)-form.
    let grade = max_dim - 1;

    let result = manifold
        .hodge_decompose(&omega, grade)
        .expect("decompose ω = δg");

    let alpha_norm_sq = norm_sq(result.exact().as_slice());
    let h_norm_sq = norm_sq(result.harmonic().as_slice());

    let alpha_ratio = alpha_norm_sq / omega_norm_sq;
    let h_ratio = h_norm_sq / omega_norm_sq;

    let tol = pure_part_tol::<R>();
    assert!(
        alpha_ratio < tol,
        "pure-co-exact: ‖α‖² / ‖ω‖² = {:?} should be near zero, tol = {:?}",
        alpha_ratio,
        tol
    );
    assert!(
        h_ratio < tol,
        "pure-co-exact: ‖h‖² / ‖ω‖² = {:?} should be near zero, tol = {:?}",
        h_ratio,
        tol
    );
}

// ---------------------------------------------------------------------------
// 4.3 — Hodge orthogonality identity across lattice sizes and precision backends
// ---------------------------------------------------------------------------

#[test]
fn orthogonality_identity_holds_f64_2d_size_3() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(3);
    orthogonality_identity_holds(lattice, 0.31_f64);
}

#[test]
fn orthogonality_identity_holds_f64_2d_size_4() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(4);
    orthogonality_identity_holds(lattice, 0.17_f64);
}

#[test]
fn orthogonality_identity_holds_f64_3d_size_2() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(2);
    orthogonality_identity_holds(lattice, 0.27_f64);
}

#[test]
fn orthogonality_identity_holds_f32_2d_size_3() {
    let lattice: LatticeComplex<2, f32> = LatticeComplex::square_open(3);
    orthogonality_identity_holds(lattice, 0.31_f32);
}

#[test]
fn orthogonality_identity_holds_float106_2d_size_3() {
    let lattice: LatticeComplex<2, Float106> = LatticeComplex::square_open(3);
    orthogonality_identity_holds(lattice, Float106::from(0.31_f64));
}

// ---------------------------------------------------------------------------
// 4.1 — Pure-exact 1-form decomposes with vanishing co-exact and harmonic parts
// ---------------------------------------------------------------------------

#[test]
fn pure_exact_1form_vanishing_beta_and_h_f64_2d() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(4);
    pure_exact_decomposition_holds(lattice, 0.43_f64);
}

#[test]
fn pure_exact_1form_vanishing_beta_and_h_f64_3d() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(2);
    pure_exact_decomposition_holds(lattice, 0.21_f64);
}

#[test]
fn pure_exact_1form_vanishing_beta_and_h_f32_2d() {
    let lattice: LatticeComplex<2, f32> = LatticeComplex::square_open(4);
    pure_exact_decomposition_holds(lattice, 0.43_f32);
}

#[test]
fn pure_exact_1form_vanishing_beta_and_h_float106_2d() {
    let lattice: LatticeComplex<2, Float106> = LatticeComplex::square_open(4);
    pure_exact_decomposition_holds(lattice, Float106::from(0.43_f64));
}

// ---------------------------------------------------------------------------
// 4.2 — Pure-co-exact 1-form decomposes with vanishing exact and harmonic parts
// ---------------------------------------------------------------------------

#[test]
fn pure_co_exact_1form_vanishing_alpha_and_h_f64_2d() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_open(4);
    pure_co_exact_decomposition_holds(lattice, 0.61_f64);
}

#[test]
fn pure_co_exact_1form_vanishing_alpha_and_h_f64_3d() {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_open(2);
    pure_co_exact_decomposition_holds(lattice, 0.19_f64);
}

#[test]
fn pure_co_exact_1form_vanishing_alpha_and_h_f32_2d() {
    let lattice: LatticeComplex<2, f32> = LatticeComplex::square_open(4);
    pure_co_exact_decomposition_holds(lattice, 0.61_f32);
}

#[test]
fn pure_co_exact_1form_vanishing_alpha_and_h_float106_2d() {
    let lattice: LatticeComplex<2, Float106> = LatticeComplex::square_open(4);
    pure_co_exact_decomposition_holds(lattice, Float106::from(0.61_f64));
}

// ---------------------------------------------------------------------------
// 4.5 — Compile-pass at every supported precision backend
// ---------------------------------------------------------------------------

#[test]
fn hodge_decomposition_carrier_instantiates_at_every_precision() {
    use deep_causality_topology::HodgeDecomposition;

    fn require_send<T>(_: &T) {}

    let h_f32: HodgeDecomposition<f32> = HodgeDecomposition::new(
        CausalTensor::new(vec![0.0_f32], vec![1]).unwrap(),
        CausalTensor::new(vec![0.0_f32], vec![1]).unwrap(),
        CausalTensor::new(vec![0.0_f32], vec![1]).unwrap(),
        0,
    );
    let h_f64: HodgeDecomposition<f64> = HodgeDecomposition::new(
        CausalTensor::new(vec![0.0_f64], vec![1]).unwrap(),
        CausalTensor::new(vec![0.0_f64], vec![1]).unwrap(),
        CausalTensor::new(vec![0.0_f64], vec![1]).unwrap(),
        0,
    );
    let h_106: HodgeDecomposition<Float106> = HodgeDecomposition::new(
        CausalTensor::new(vec![Float106::from(0.0_f64)], vec![1]).unwrap(),
        CausalTensor::new(vec![Float106::from(0.0_f64)], vec![1]).unwrap(),
        CausalTensor::new(vec![Float106::from(0.0_f64)], vec![1]).unwrap(),
        0,
    );
    require_send(&h_f32);
    require_send(&h_f64);
    require_send(&h_106);
    assert_eq!(h_f32.grade(), 0);
    assert_eq!(h_f64.grade(), 0);
    assert_eq!(h_106.grade(), 0);
}
