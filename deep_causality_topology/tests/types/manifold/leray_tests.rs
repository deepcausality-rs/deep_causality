/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the Leray projection (`Manifold::{leray_project,
//! leray_project_opts, leray_vs_hodge_gradient_gap}`) and for the full Hodge
//! decomposition on **periodic** lattices (the G6 concern: the β-step solves
//! `Δ_{k+1} ψ = dω`, whose operator has a nontrivial harmonic kernel on tori).
//!
//! The projector itself uses only the gauge-fixed grade-0 solve, so it is
//! well-posed on tori by construction; the torus tests of the *full*
//! decomposition pin the empirical behavior of the consistent-but-singular
//! β-step (its RHS `dω` is M-orthogonal to the harmonic kernel in exact
//! arithmetic).

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn unit_manifold<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField
        + deep_causality_topology::MaybeParallel
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

fn manifold_with_k_form<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
    k: usize,
    form: &[R],
) -> Manifold<LatticeComplex<D, R>, R>
where
    R: RealField
        + deep_causality_topology::MaybeParallel
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

fn random_cochain<R: RealField + deep_causality_topology::MaybeParallel + FromPrimitive>(
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
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            R::from_f64(2.0 * unit - 1.0).expect("[-1,1] lifts into any RealField")
        })
        .collect()
}

fn sup_norm<R: RealField>(v: &[R]) -> R {
    v.iter()
        .map(|x| x.abs())
        .fold(R::zero(), |m, x| if x > m { x } else { m })
}

/// Discrete divergence (δ) of a 1-form on the given lattice.
fn divergence_of<const D: usize, R>(
    lattice_fn: impl Fn() -> LatticeComplex<D, R>,
    one_form: &[R],
) -> Vec<R>
where
    R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let m = manifold_with_k_form(lattice_fn(), 1, one_form);
    m.codifferential(1).as_slice().to_vec()
}

// ---------------------------------------------------------------------------
// Projection laws
// ---------------------------------------------------------------------------

#[test]
fn projection_annihilates_exact_gradients() {
    // ω = dφ for a random potential φ: P(ω) must vanish to CG tolerance.
    let n = 8usize;
    let lattice_fn = || LatticeComplex::<2, f64>::square_torus(n);
    let n0 = lattice_fn().num_cells(0);
    let phi = random_cochain::<f64>(n0, 3);
    let m_phi = manifold_with_k_form(lattice_fn(), 0, &phi);
    let d_phi = m_phi.exterior_derivative(0);

    let manifold = unit_manifold(lattice_fn());
    let projection = manifold.leray_project(&d_phi).unwrap();

    let scale = sup_norm(d_phi.as_slice()).max(1.0);
    let resid = sup_norm(projection.projected().as_slice());
    assert!(
        resid / scale < 1e-8,
        "P(dφ) should vanish: relative sup-norm {}",
        resid / scale
    );
}

fn assert_projection_divergence_free<R>(rel_tol: R)
where
    R: RealField
        + deep_causality_topology::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let n = 6usize;
    let lattice_fn = || LatticeComplex::<2, R>::square_torus(n);
    let manifold = unit_manifold(lattice_fn());
    let n1 = lattice_fn().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<R>(n1, 11), vec![n1]).unwrap();

    let projection = manifold.leray_project(&omega).unwrap();
    let div = divergence_of(lattice_fn, projection.projected().as_slice());

    let scale = sup_norm(omega.as_slice());
    let resid = sup_norm(&div);
    assert!(
        resid < rel_tol * scale,
        "post-projection divergence too large: {resid} (scale {scale})"
    );
}

#[test]
fn projection_is_divergence_free_f64() {
    assert_projection_divergence_free::<f64>(1e-7);
}

#[test]
fn projection_is_divergence_free_f32() {
    // f32 default tolerance floors at ~1.2e-5 (epsilon clamp).
    assert_projection_divergence_free::<f32>(1e-3_f32);
}

#[test]
fn projection_is_divergence_free_float106() {
    assert_projection_divergence_free::<Float106>(Float106::from_f64(1e-7));
}

#[test]
fn projection_is_idempotent() {
    let n = 6usize;
    let lattice_fn = || LatticeComplex::<2, f64>::square_torus(n);
    let manifold = unit_manifold(lattice_fn());
    let n1 = lattice_fn().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 19), vec![n1]).unwrap();

    let once = manifold.leray_project(&omega).unwrap();
    let twice = manifold.leray_project(once.projected()).unwrap();

    let mut max_diff = 0.0_f64;
    for (a, b) in twice
        .projected()
        .as_slice()
        .iter()
        .zip(once.projected().as_slice().iter())
    {
        max_diff = max_diff.max((a - b).abs());
    }
    let scale = sup_norm(once.projected().as_slice());
    assert!(
        max_diff < 1e-7 * scale.max(1.0),
        "P(P(ω)) deviates from P(ω) by {max_diff}"
    );
}

#[test]
fn projection_retains_harmonic_mean_flow_on_torus() {
    // A constant 1-form along each axis is harmonic on the torus (the mean
    // flow): the projector must return it unchanged.
    for (d_label, result) in [
        ("2d", {
            let lattice_fn = || LatticeComplex::<2, f64>::square_torus(5);
            let manifold = unit_manifold(lattice_fn());
            let lattice = lattice_fn();
            let n1 = lattice.num_cells(1);
            let vals: Vec<f64> = lattice
                .iter_cells(1)
                .map(|c| {
                    if c.orientation().trailing_zeros() == 0 {
                        0.75
                    } else {
                        -1.5
                    }
                })
                .collect();
            let omega = CausalTensor::new(vals.clone(), vec![n1]).unwrap();
            let projection = manifold.leray_project(&omega).unwrap();
            let mut max_diff = 0.0_f64;
            for (a, b) in projection.projected().as_slice().iter().zip(vals.iter()) {
                max_diff = max_diff.max((a - b).abs());
            }
            max_diff
        }),
        ("3d", {
            let lattice_fn = || LatticeComplex::<3, f64>::cubic_torus(3);
            let manifold = unit_manifold(lattice_fn());
            let lattice = lattice_fn();
            let n1 = lattice.num_cells(1);
            let vals: Vec<f64> = lattice
                .iter_cells(1)
                .map(|c| match c.orientation().trailing_zeros() {
                    0 => 1.0,
                    1 => 2.0,
                    _ => -3.0,
                })
                .collect();
            let omega = CausalTensor::new(vals.clone(), vec![n1]).unwrap();
            let projection = manifold.leray_project(&omega).unwrap();
            let mut max_diff = 0.0_f64;
            for (a, b) in projection.projected().as_slice().iter().zip(vals.iter()) {
                max_diff = max_diff.max((a - b).abs());
            }
            max_diff
        }),
    ] {
        assert!(
            result < 1e-8,
            "{d_label}: harmonic mean flow altered by {result}"
        );
    }
}

// ---------------------------------------------------------------------------
// Full decomposition on periodic lattices (the G6 pin)
// ---------------------------------------------------------------------------

#[test]
fn full_decomposition_converges_on_2d_and_3d_tori() {
    // The β-step operator is singular on tori (harmonic kernel, β_k > 0), but
    // its RHS `dω` is M-orthogonal to that kernel in exact arithmetic; CG on
    // the consistent singular system must converge. This test pins that
    // behavior at both 2D and 3D.
    let lattice2 = || LatticeComplex::<2, f64>::square_torus(5);
    let m2 = unit_manifold(lattice2());
    let n1 = lattice2().num_cells(1);
    let omega2 = CausalTensor::new(random_cochain::<f64>(n1, 29), vec![n1]).unwrap();
    let d2 = m2
        .hodge_decompose(&omega2, 1)
        .expect("2D torus decomposition must converge");

    let lattice3 = || LatticeComplex::<3, f64>::cubic_torus(3);
    let m3 = unit_manifold(lattice3());
    let n1_3 = lattice3().num_cells(1);
    let omega3 = CausalTensor::new(random_cochain::<f64>(n1_3, 31), vec![n1_3]).unwrap();
    let d3 = m3
        .hodge_decompose(&omega3, 1)
        .expect("3D torus decomposition must converge");

    // Reconstruction: ω = α + β + h by construction; pin it anyway.
    for (dec, omega) in [(&d2, &omega2), (&d3, &omega3)] {
        for i in 0..omega.len() {
            let sum = dec.exact().as_slice()[i]
                + dec.co_exact().as_slice()[i]
                + dec.harmonic().as_slice()[i];
            assert!((sum - omega.as_slice()[i]).abs() < 1e-12);
        }
    }
}

#[test]
fn full_decomposition_survives_larger_torus_at_default_tolerance() {
    // Guard against small-fixture optimism: at 16x16 (512 edges) the
    // consistent singular beta-step must still converge at the default
    // (1e-10 relative) tolerance without kernel-drift stagnation. If this
    // ever fails at larger scales, the documented fallback is constructive
    // harmonic-basis deflation (see the leray-projection spec).
    let lattice_fn = || LatticeComplex::<2, f64>::square_torus(16);
    let manifold = unit_manifold(lattice_fn());
    let n1 = lattice_fn().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 47), vec![n1]).unwrap();
    let dec = manifold
        .hodge_decompose(&omega, 1)
        .expect("16x16 torus decomposition must converge at default tolerance");
    for i in 0..n1 {
        let sum =
            dec.exact().as_slice()[i] + dec.co_exact().as_slice()[i] + dec.harmonic().as_slice()[i];
        assert!((sum - omega.as_slice()[i]).abs() < 1e-10);
    }
}

#[test]
fn full_decomposition_components_are_orthogonal_on_torus() {
    // Pairwise (unit-metric) inner products of the three components vanish to
    // solver tolerance, relative to the component magnitudes.
    let lattice_fn = || LatticeComplex::<2, f64>::square_torus(5);
    let manifold = unit_manifold(lattice_fn());
    let n1 = lattice_fn().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 37), vec![n1]).unwrap();
    let dec = manifold.hodge_decompose(&omega, 1).unwrap();

    let dot = |a: &[f64], b: &[f64]| -> f64 { a.iter().zip(b).map(|(x, y)| x * y).sum() };
    let norm = |a: &[f64]| -> f64 { dot(a, a).sqrt().max(1e-300) };

    let (e, c, h) = (
        dec.exact().as_slice(),
        dec.co_exact().as_slice(),
        dec.harmonic().as_slice(),
    );
    for (label, x, y) in [("e·c", e, c), ("e·h", e, h), ("c·h", c, h)] {
        let rel = dot(x, y).abs() / (norm(x) * norm(y));
        assert!(rel < 1e-6, "{label} not orthogonal: relative {rel}");
    }
}

#[test]
fn half_and_full_decomposition_agree_on_gradient_part() {
    let lattice_fn = || LatticeComplex::<2, f64>::square_torus(5);
    let manifold = unit_manifold(lattice_fn());
    let n1 = lattice_fn().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 41), vec![n1]).unwrap();

    let gap = manifold
        .leray_vs_hodge_gradient_gap(&omega, &HodgeDecomposeOptions::default())
        .unwrap();
    let scale = sup_norm(omega.as_slice());
    assert!(gap < 1e-7 * scale, "half-vs-full gradient gap {gap}");
}

#[test]
fn full_decomposition_converges_on_mixed_periodicity_lattice() {
    // [periodic, open]: a partial harmonic basis exists (β₁ = 1); the
    // consistent singular β/α solves must still converge.
    let lattice_fn = || LatticeComplex::<2, f64>::new([5, 4], [true, false]);
    let manifold = unit_manifold(lattice_fn());
    let n1 = lattice_fn().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 43), vec![n1]).unwrap();
    let dec = manifold
        .hodge_decompose(&omega, 1)
        .expect("mixed-periodicity decomposition must converge");
    for i in 0..n1 {
        let sum =
            dec.exact().as_slice()[i] + dec.co_exact().as_slice()[i] + dec.harmonic().as_slice()[i];
        assert!((sum - omega.as_slice()[i]).abs() < 1e-12);
    }
}

// ---------------------------------------------------------------------------
// Error paths and option branches
// ---------------------------------------------------------------------------

#[test]
fn leray_rejects_length_mismatch() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let bad = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let err = manifold.leray_project(&bad).unwrap_err();
    assert!(format!("{err}").contains("leray_project"));
}

#[test]
fn leray_rejects_missing_metric() {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(3);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let n1 = lattice.num_cells(1);
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::from_cubical(lattice, data, 0);
    let omega = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let err = manifold.leray_project(&omega).unwrap_err();
    assert!(format!("{err}").contains("requires a metric"));
}

#[test]
fn leray_surfaces_cg_nonconvergence() {
    // A one-iteration budget cannot converge the Poisson solve for a strong
    // gradient field: the CgFailure must surface as HodgeDecompositionFailed.
    let n = 8usize;
    let lattice_fn = || LatticeComplex::<2, f64>::square_torus(n);
    let n0 = lattice_fn().num_cells(0);
    let phi = random_cochain::<f64>(n0, 53);
    let m_phi = manifold_with_k_form(lattice_fn(), 0, &phi);
    let d_phi = m_phi.exterior_derivative(0);

    let manifold = unit_manifold(lattice_fn());
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(1),
    };
    let err = manifold.leray_project_opts(&d_phi, &opts).unwrap_err();
    assert!(format!("{err}").contains("did not converge"));
}

#[test]
fn leray_rejects_non_positive_tolerance() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(vec![0.0; n1], vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(0.0),
        max_iterations: None,
    };
    let err = manifold.leray_project_opts(&omega, &opts).unwrap_err();
    assert!(format!("{err}").contains("strictly positive"));
}

#[test]
fn leray_accepts_caller_supplied_options() {
    // Custom (looser) tolerance and budget succeed: the options branch.
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(4));
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 59), vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-6),
        max_iterations: Some(500),
    };
    let projection = manifold.leray_project_opts(&omega, &opts).unwrap();
    assert_eq!(projection.projected().len(), n1);
    assert_eq!(
        projection.potential().len(),
        manifold.complex().num_cells(0)
    );
}

#[test]
fn leray_projection_carrier_accessors_and_parts() {
    let manifold = unit_manifold(LatticeComplex::<2, f64>::square_torus(3));
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 61), vec![n1]).unwrap();
    let projection = manifold.leray_project(&omega).unwrap();

    // Debug/Clone/PartialEq surface of the carrier.
    let clone = projection.clone();
    assert_eq!(clone, projection);
    assert!(format!("{projection:?}").contains("LerayProjection"));

    let (projected, potential) = projection.into_parts();
    assert_eq!(projected.len(), n1);
    assert_eq!(potential.len(), manifold.complex().num_cells(0));
}
