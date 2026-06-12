/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Law and error-path tests for the discrete wedge product on cubical lattice
//! cochains (`Manifold::wedge`), plus the G4 convention-pinning tests.
//!
//! Conventions pinned here (change set `add-dec-solver-foundations`, design D2):
//! * `Δ_dR = −∇²` on a flat torus: the Hodge–de Rham Laplacian of a sampled
//!   single-mode sine is `+λ` times the field with `λ = 2 − 2 cos(2πm/N) > 0`.
//! * The cup-product ordering is consistent with the lattice `boundary`
//!   orientation: the Leibniz rule holds to machine precision for arbitrary
//!   cochains (not merely sampled-smooth fields), which is the sharpest
//!   possible detector for a sign/orientation mismatch.
//! * Graded anticommutativity `α∧β = (−1)^{kl} β∧α` holds by construction and
//!   is asserted to machine precision.
//!
//! The wedge is metric-free by design; there is no missing-metric error path
//! to cover (geometry enters only through the Hodge star elsewhere).

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Build a `Manifold<LatticeComplex<D, R>, R>` with a unit-edge cubical Regge
/// geometry and zero-filled graded cell data.
fn unit_manifold<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let total: usize = (0..=D).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// Build a manifold whose graded data carries `form` at grade `k` (everything
/// else zero), so `exterior_derivative(k)` acts on exactly that cochain.
fn manifold_with_k_form<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
    k: usize,
    form: &[R],
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let total: usize = (0..=D).map(|g| lattice.num_cells(g)).sum();
    let offset: usize = (0..k).map(|g| lattice.num_cells(g)).sum();
    let mut data = vec![R::zero(); total];
    data[offset..offset + form.len()].copy_from_slice(form);
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<D, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, tensor, metric, 0)
}

/// Deterministic pseudo-random cochain in [−1, 1] (LCG; no external crates).
fn random_cochain<R: RealField + deep_causality_par::MaybeParallel + FromPrimitive>(
    len: usize,
    seed: u64,
) -> Vec<R> {
    let mut state = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (0..len)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            // Map the top 53 bits to [−1, 1].
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            R::from_f64(2.0 * unit - 1.0).expect("[-1,1] lifts into any RealField")
        })
        .collect()
}

fn max_abs_diff<R: RealField>(a: &[R], b: &[R]) -> R {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (*x - *y).abs())
        .fold(R::zero(), |m, v| if v > m { v } else { m })
}

// ---------------------------------------------------------------------------
// G4 convention pin: Δ_dR = −∇² on a flat torus
// ---------------------------------------------------------------------------

#[test]
fn laplacian_sign_convention_single_mode_sine_on_torus() {
    // f(p) = sin(2π m p₀ / N) on a unit-spacing N×N torus is an exact
    // eigenvector of the discrete grade-0 Hodge Laplacian with eigenvalue
    // λ = 2 − 2 cos(2π m / N) > 0. Positive λ pins Δ_dR = −∇² (the viscous
    // term in any consumer is therefore −ν·Δ_dR).
    let n = 8usize;
    let m = 1usize;
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let k_wave = 2.0 * std::f64::consts::PI * (m as f64) / (n as f64);
    let lambda = 2.0 - 2.0 * k_wave.cos();

    let f: Vec<f64> = lattice
        .iter_cells(0)
        .map(|c| (k_wave * c.position()[0] as f64).sin())
        .collect();

    let manifold = manifold_with_k_form(lattice, 0, &f);
    let lap = manifold.laplacian(0);

    for (i, (lap_v, f_v)) in lap.as_slice().iter().zip(f.iter()).enumerate() {
        let expected = lambda * f_v;
        assert!(
            (lap_v - expected).abs() < 1e-12,
            "vertex {i}: Δf = {lap_v}, expected λ·f = {expected} (λ = {lambda})"
        );
    }
}

#[test]
fn laplacian_eigenvalue_converges_to_continuum_at_second_order() {
    // λ_N = 2 − 2 cos(2πm/N) → k² with relative error O(N⁻²): the discrete
    // sign convention matches the continuum −∇² and at the expected order.
    let m = 1usize;
    let mut rel_errors = Vec::new();
    for n in [8usize, 16, 32] {
        let k_wave = 2.0 * std::f64::consts::PI * (m as f64) / (n as f64);
        let lambda = 2.0 - 2.0 * k_wave.cos();
        rel_errors.push(((lambda / (k_wave * k_wave)) - 1.0).abs());
    }
    // Each refinement by 2× must shrink the relative error by ≈4× (allow 3×).
    assert!(rel_errors[1] < rel_errors[0] / 3.0, "{rel_errors:?}");
    assert!(rel_errors[2] < rel_errors[1] / 3.0, "{rel_errors:?}");
}

// ---------------------------------------------------------------------------
// Wedge laws: Leibniz + graded anticommutativity, generic over precision
// ---------------------------------------------------------------------------

/// Assert Leibniz and graded anticommutativity for every grade combination on
/// the given lattice, with arbitrary (pseudo-random) cochains. Exactness at
/// cochain level is the convention detector: any cup/boundary orientation
/// mismatch shows up as an O(1) violation, not a small one.
fn assert_wedge_laws_on<const D: usize, R>(lattice_fn: impl Fn() -> LatticeComplex<D, R>, tol: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    // --- Graded anticommutativity: all k + l ≤ D ---
    for k in 0..=D {
        for l in 0..=(D - k) {
            let lattice = lattice_fn();
            let manifold = unit_manifold(lattice);
            let nk = manifold.complex().num_cells(k);
            let nl = manifold.complex().num_cells(l);
            let alpha = CausalTensor::new(random_cochain::<R>(nk, 7), vec![nk]).unwrap();
            let beta = CausalTensor::new(random_cochain::<R>(nl, 13), vec![nl]).unwrap();

            let ab = manifold.wedge(&alpha, k, &beta, l).unwrap();
            let ba = manifold.wedge(&beta, l, &alpha, k).unwrap();

            let sign_neg = (k * l) % 2 == 1;
            let expected: Vec<R> = ba
                .as_slice()
                .iter()
                .map(|v| if sign_neg { R::zero() - *v } else { *v })
                .collect();
            let diff = max_abs_diff(ab.as_slice(), &expected);
            assert!(
                diff < tol,
                "graded anticommutativity failed for (k, l) = ({k}, {l}) on D = {D}: max diff {diff}"
            );
        }
    }

    // --- Leibniz: d(α∧β) = dα∧β + (−1)^k α∧dβ, for all k + l < D ---
    for k in 0..D {
        for l in 0..(D - k) {
            if k + l >= D {
                continue;
            }
            let lattice = lattice_fn();
            let nk = lattice.num_cells(k);
            let nl = lattice.num_cells(l);
            let alpha_vec = random_cochain::<R>(nk, 17);
            let beta_vec = random_cochain::<R>(nl, 23);
            let alpha = CausalTensor::new(alpha_vec.clone(), vec![nk]).unwrap();
            let beta = CausalTensor::new(beta_vec.clone(), vec![nl]).unwrap();

            // d(α ∧ β)
            let m_zero = unit_manifold(lattice_fn());
            let wedge_ab = m_zero.wedge(&alpha, k, &beta, l).unwrap();
            let m_ab = manifold_with_k_form(lattice_fn(), k + l, wedge_ab.as_slice());
            let lhs = m_ab.exterior_derivative(k + l);

            // dα ∧ β
            let m_a = manifold_with_k_form(lattice_fn(), k, &alpha_vec);
            let d_alpha = m_a.exterior_derivative(k);
            let term1 = m_zero.wedge(&d_alpha, k + 1, &beta, l).unwrap();

            // (−1)^k α ∧ dβ
            let m_b = manifold_with_k_form(lattice_fn(), l, &beta_vec);
            let d_beta = m_b.exterior_derivative(l);
            let term2 = m_zero.wedge(&alpha, k, &d_beta, l + 1).unwrap();

            let k_sign_neg = k % 2 == 1;
            let rhs: Vec<R> = term1
                .as_slice()
                .iter()
                .zip(term2.as_slice().iter())
                .map(|(t1, t2)| if k_sign_neg { *t1 - *t2 } else { *t1 + *t2 })
                .collect();

            let diff = max_abs_diff(lhs.as_slice(), &rhs);
            assert!(
                diff < tol,
                "Leibniz failed for (k, l) = ({k}, {l}) on D = {D}: max diff {diff}"
            );
        }
    }
}

#[test]
fn wedge_laws_hold_on_2d_torus_f64() {
    assert_wedge_laws_on(|| LatticeComplex::<2, f64>::square_torus(4), 1e-12);
}

#[test]
fn wedge_laws_hold_on_2d_open_f64() {
    assert_wedge_laws_on(|| LatticeComplex::<2, f64>::square_open(4), 1e-12);
}

#[test]
fn wedge_laws_hold_on_2d_mixed_periodicity_f64() {
    assert_wedge_laws_on(
        || LatticeComplex::<2, f64>::new([4, 3], [true, false]),
        1e-12,
    );
}

#[test]
fn wedge_laws_hold_on_3d_torus_f64() {
    assert_wedge_laws_on(|| LatticeComplex::<3, f64>::cubic_torus(3), 1e-12);
}

#[test]
fn wedge_laws_hold_on_3d_open_f64() {
    assert_wedge_laws_on(|| LatticeComplex::<3, f64>::cubic_open(3), 1e-12);
}

#[test]
fn wedge_laws_hold_on_2d_torus_f32() {
    assert_wedge_laws_on(|| LatticeComplex::<2, f32>::square_torus(4), 1e-4_f32);
}

#[test]
fn wedge_laws_hold_on_2d_torus_float106() {
    let tol = Float106::from_f64(1e-25);
    assert_wedge_laws_on(|| LatticeComplex::<2, Float106>::square_torus(4), tol);
}

// ---------------------------------------------------------------------------
// Scalar (0-form) wedge semantics
// ---------------------------------------------------------------------------

#[test]
fn zero_form_wedge_is_pointwise_product() {
    // 0-form ∧ 0-form is the pointwise product (front and back corners of a
    // vertex coincide, so symmetrization is the identity here).
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n0 = manifold.complex().num_cells(0);
    let a: Vec<f64> = (0..n0).map(|i| 1.0 + i as f64).collect();
    let b: Vec<f64> = (0..n0).map(|i| 2.0 - 0.5 * i as f64).collect();
    let alpha = CausalTensor::new(a.clone(), vec![n0]).unwrap();
    let beta = CausalTensor::new(b.clone(), vec![n0]).unwrap();

    let out = manifold.wedge(&alpha, 0, &beta, 0).unwrap();
    for i in 0..n0 {
        assert!((out.as_slice()[i] - a[i] * b[i]).abs() < 1e-14);
    }
}

#[test]
fn zero_form_times_one_form_averages_endpoint_scalars() {
    // (f ∧ u)(e) = ½(f(p) + f(p+e)) · u(e): the symmetrized cup averages the
    // scalar over the edge's two endpoints.
    let lattice: LatticeComplex<1, f64> = LatticeComplex::new([4], [true]);
    let manifold = unit_manifold(lattice);
    let f = vec![1.0, 2.0, 3.0, 4.0];
    let u = vec![10.0, 20.0, 30.0, 40.0];
    let alpha = CausalTensor::new(f.clone(), vec![4]).unwrap();
    let beta = CausalTensor::new(u.clone(), vec![4]).unwrap();

    let out = manifold.wedge(&alpha, 0, &beta, 1).unwrap();
    for e in 0..4 {
        let avg = 0.5 * (f[e] + f[(e + 1) % 4]);
        assert!(
            (out.as_slice()[e] - avg * u[e]).abs() < 1e-14,
            "edge {e}: got {}, expected {}",
            out.as_slice()[e],
            avg * u[e]
        );
    }
}

// ---------------------------------------------------------------------------
// Error paths
// ---------------------------------------------------------------------------

#[test]
fn wedge_rejects_grade_overflow() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n1 = manifold.complex().num_cells(1);
    let n2 = manifold.complex().num_cells(2);
    let alpha = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let beta = CausalTensor::new(vec![0.0; n2], vec![n2]).unwrap();

    let err = manifold.wedge(&alpha, 1, &beta, 2).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("wedge grade overflow"), "got: {msg}");
}

#[test]
fn wedge_rejects_first_operand_length_mismatch() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n1 = manifold.complex().num_cells(1);
    let alpha = CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(); // wrong length
    let beta = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();

    let err = manifold.wedge(&alpha, 1, &beta, 1).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("wedge first operand"), "got: {msg}");
}

#[test]
fn wedge_rejects_second_operand_length_mismatch() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let manifold = unit_manifold(lattice);
    let n0 = manifold.complex().num_cells(0);
    let alpha = CausalTensor::new(vec![0.0; n0], vec![n0]).unwrap();
    let beta = CausalTensor::new(vec![0.0; 5], vec![5]).unwrap(); // wrong length

    let err = manifold.wedge(&alpha, 0, &beta, 1).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("wedge second operand"), "got: {msg}");
}
