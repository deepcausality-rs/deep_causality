/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the spectral grade-0 Poisson fast path: on fully periodic
//! uniform lattices with Euclidean spacings, `leray_project` (and the
//! grade-0 branch of `hodge_decompose`) solve `Δ₀ φ = δω` by FFT instead
//! of CG. These tests pin the agreement between the two solvers, the
//! residual exactness of the spectral solution, the gauge convention,
//! and the dispatch boundary (mixed/open/per-edge stay on CG).

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_sparse::cg_solve;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

// ---------------------------------------------------------------------------
// Fixtures (mirrors leray_tests.rs; Bazel test suites are per-file)
// ---------------------------------------------------------------------------

fn manifold_with_metric<const D: usize, R>(
    lattice: LatticeComplex<D, R>,
    metric: CubicalReggeGeometry<D, R>,
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
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

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
            let unit = (state >> 11) as f64 / (1u64 << 53) as f64;
            R::from_f64(2.0 * unit - 1.0).expect("[-1,1] lifts into any RealField")
        })
        .collect()
}

fn sup_norm(v: &[f64]) -> f64 {
    v.iter().fold(0.0, |m, x| m.max(x.abs()))
}

fn subtract_mean(v: &mut [f64]) {
    let mean = v.iter().sum::<f64>() / v.len() as f64;
    for x in v.iter_mut() {
        *x -= mean;
    }
}

/// Reference grade-0 CG solve on the same manifold and operator — the
/// pre-dispatch behavior, replicated so both solvers can be compared on
/// the identical problem.
fn cg_reference_potential<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    field: &CausalTensor<f64>,
) -> Vec<f64> {
    let n0 = manifold.complex().num_cells(0);
    let mut rhs = manifold.codifferential_of(field.as_slice(), 1).into_vec();
    rhs.resize(n0, 0.0);
    subtract_mean(&mut rhs);
    let apply = |v: &[f64]| -> Vec<f64> {
        let mut out = manifold.laplacian_of(v, 0).into_vec();
        out.resize(n0, 0.0);
        out
    };
    let mut phi = cg_solve(apply, &rhs, 1e-13, 10_000).expect("CG reference converges");
    subtract_mean(&mut phi);
    phi
}

fn leray_potential<const D: usize>(
    manifold: &Manifold<LatticeComplex<D, f64>, f64>,
    field: &CausalTensor<f64>,
) -> Vec<f64> {
    manifold
        .leray_project(field)
        .expect("projection succeeds")
        .potential()
        .as_slice()
        .to_vec()
}

// ---------------------------------------------------------------------------
// Spectral vs CG agreement
// ---------------------------------------------------------------------------

#[test]
fn spectral_and_cg_agree_on_unit_torus() {
    let manifold = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(8),
        CubicalReggeGeometry::unit(),
    );
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 7), vec![n1]).unwrap();

    let spectral = leray_potential(&manifold, &omega);
    let cg = cg_reference_potential(&manifold, &omega);
    let gap = sup_norm(
        &spectral
            .iter()
            .zip(cg.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<_>>(),
    );
    assert!(gap < 1e-9, "spectral vs CG gap {gap:e}");
}

#[test]
fn spectral_and_cg_agree_anisotropic_non_power_of_two() {
    // Per-axis spacings plus a non-power-of-two axis (6 → Bluestein
    // inside the spectral pipeline).
    let manifold = manifold_with_metric(
        LatticeComplex::<2, f64>::new([4, 6], [true, true]),
        CubicalReggeGeometry::per_axis([0.5, 0.25]),
    );
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 11), vec![n1]).unwrap();

    let spectral = leray_potential(&manifold, &omega);
    let cg = cg_reference_potential(&manifold, &omega);
    let gap = sup_norm(
        &spectral
            .iter()
            .zip(cg.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<_>>(),
    );
    assert!(gap < 1e-9, "spectral vs CG gap {gap:e}");
}

#[test]
fn spectral_and_cg_agree_3d_uniform_spacing() {
    let manifold = manifold_with_metric(
        LatticeComplex::<3, f64>::cubic_torus(4),
        CubicalReggeGeometry::uniform(0.5),
    );
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 13), vec![n1]).unwrap();

    let spectral = leray_potential(&manifold, &omega);
    let cg = cg_reference_potential(&manifold, &omega);
    let gap = sup_norm(
        &spectral
            .iter()
            .zip(cg.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<_>>(),
    );
    assert!(gap < 1e-9, "spectral vs CG gap {gap:e}");
}

// ---------------------------------------------------------------------------
// Exactness and gauge
// ---------------------------------------------------------------------------

#[test]
fn spectral_residual_is_at_rounding() {
    // Δ₀φ must reproduce the mean-free RHS to rounding — not merely to a
    // CG tolerance. This is the eigenvalue/operator-match gate.
    let manifold = manifold_with_metric(
        LatticeComplex::<3, f64>::cubic_torus(4),
        CubicalReggeGeometry::uniform(0.5),
    );
    let n0 = manifold.complex().num_cells(0);
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 17), vec![n1]).unwrap();

    let mut rhs = manifold.codifferential_of(omega.as_slice(), 1).into_vec();
    rhs.resize(n0, 0.0);
    subtract_mean(&mut rhs);

    let phi = leray_potential(&manifold, &omega);
    let mut residual = manifold.laplacian_of(&phi, 0).into_vec();
    residual.resize(n0, 0.0);
    let err = sup_norm(
        &residual
            .iter()
            .zip(rhs.iter())
            .map(|(a, b)| a - b)
            .collect::<Vec<_>>(),
    );
    assert!(err < 1e-12, "spectral residual {err:e}");
}

#[test]
fn spectral_potential_is_mean_free() {
    let manifold = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(8),
        CubicalReggeGeometry::unit(),
    );
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 19), vec![n1]).unwrap();
    let phi = leray_potential(&manifold, &omega);
    let mean = phi.iter().sum::<f64>() / phi.len() as f64;
    assert!(mean.abs() < 1e-13, "potential mean {mean:e}");
}

#[test]
fn spectral_projection_annihilates_gradients_at_float106() {
    // The precision-generic payoff: at Float106 the projected gradient
    // lands far below f64 rounding.
    let lattice = LatticeComplex::<2, Float106>::square_torus(8);
    let n0 = lattice.num_cells(0);
    let phi0 = random_cochain::<Float106>(n0, 23);

    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let mut data = vec![Float106::from_f64(0.0); total];
    data[..n0].copy_from_slice(&phi0);
    let tensor = CausalTensor::new(data, vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, Float106> = CubicalReggeGeometry::unit();
    let manifold = Manifold::from_cubical_with_metric(lattice, tensor, metric, 0);

    let d_phi = manifold.exterior_derivative(0);
    let projection = manifold.leray_project(&d_phi).expect("projection succeeds");
    let sup = projection
        .projected()
        .as_slice()
        .iter()
        .map(|x| {
            let a = <Float106 as deep_causality_num::Float>::abs(*x);
            a.hi()
        })
        .fold(0.0f64, f64::max);
    assert!(sup < 1e-25, "projected gradient sup {sup:e}");
}

// ---------------------------------------------------------------------------
// Dispatch boundary
// ---------------------------------------------------------------------------

#[test]
fn spectral_path_ignores_iteration_budget() {
    // A one-iteration budget starves CG but not the spectral solve: on a
    // fully periodic lattice the projection must still succeed.
    let manifold = manifold_with_metric(
        LatticeComplex::<2, f64>::square_torus(8),
        CubicalReggeGeometry::unit(),
    );
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 29), vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(1),
    };
    assert!(manifold.leray_project_opts(&omega, &opts).is_ok());
}

#[test]
fn mixed_periodicity_stays_on_cg() {
    // Same starved budget on a mixed-periodicity lattice: CG runs and
    // its non-convergence surfaces, proving the dispatch boundary.
    let manifold = manifold_with_metric(
        LatticeComplex::<2, f64>::new([8, 8], [true, false]),
        CubicalReggeGeometry::unit(),
    );
    let n1 = manifold.complex().num_cells(1);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 31), vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(1),
    };
    let err = manifold.leray_project_opts(&omega, &opts).unwrap_err();
    assert!(format!("{err}").contains("did not converge"));
}

#[test]
fn per_edge_metric_stays_on_cg() {
    // Per-edge geometry has no per-axis spacings; the spectral path must
    // decline and CG must run (starved budget surfaces the failure).
    let lattice = LatticeComplex::<2, f64>::square_torus(4);
    let n1 = lattice.num_cells(1);
    let metric: CubicalReggeGeometry<2, f64> =
        CubicalReggeGeometry::from_edge_lengths(vec![1.0; n1]);
    let manifold = manifold_with_metric(lattice, metric);
    let omega = CausalTensor::new(random_cochain::<f64>(n1, 37), vec![n1]).unwrap();
    let opts = HodgeDecomposeOptions {
        tolerance: Some(1e-12),
        max_iterations: Some(1),
    };
    let err = manifold.leray_project_opts(&omega, &opts).unwrap_err();
    assert!(format!("{err}").contains("did not converge"));
}
