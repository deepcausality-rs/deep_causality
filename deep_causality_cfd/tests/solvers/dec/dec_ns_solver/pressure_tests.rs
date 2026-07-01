/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pressure-recovery tests: the static convention against the analytic
//! Taylor–Green pressure up to gauge over the refinement ladder, the exact
//! Bernoulli/static identity, and the CG failure path.

use deep_causality_cfd::DecNsSolver;
use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    ChainComplex, CubicalReggeGeometry, HodgeDecomposeOptions, LatticeComplex, Manifold,
};

fn unit_manifold<R>(n: usize) -> Manifold<LatticeComplex<2, R>, R>
where
    R: RealField + FromPrimitive,
{
    let lattice: LatticeComplex<2, R> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![R::zero(); total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, R> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn tg_vertex_tensor<R>(manifold: &Manifold<LatticeComplex<2, R>, R>, n: usize) -> CausalTensor<R>
where
    R: RealField + FromPrimitive,
{
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![R::zero(); 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = R::from_f64((k * x).sin() * (k * y).cos()).unwrap();
        vertex[2 * vi + 1] = R::from_f64(-(k * x).cos() * (k * y).sin()).unwrap();
    }
    CausalTensor::new(vertex, vec![2 * n0]).unwrap()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// Static pressure vs. the analytic Taylor–Green pressure, both
/// mean-subtracted (pressure on a torus is defined up to a constant), at
/// second order over the ladder. For this phase convention
/// (`u = (sin kx cos ky, −cos kx sin ky)`) steady Euler gives
/// `(u·∇)u = (½sin2kx, ½sin2ky) = −∇p`, hence
/// `p = +¼(cos2kx + cos2ky)`.
#[test]
fn tg_static_pressure_converges_to_analytic_field() {
    let mut rel_errors = Vec::new();

    for n in [8usize, 16, 32] {
        let k = 2.0 * std::f64::consts::PI / (n as f64);
        let manifold = unit_manifold::<f64>(n);
        let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
        let state = solver
            .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
            .unwrap();

        let (_bernoulli, static_p) = solver.pressure_diagnostic(&state).unwrap();
        let p_num = static_p.as_tensor().as_slice();
        let p_num_mean = mean(p_num);

        let mut analytic = Vec::with_capacity(p_num.len());
        for v in manifold.complex().iter_cells(0) {
            let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
            analytic.push(0.25 * ((2.0 * k * x).cos() + (2.0 * k * y).cos()));
        }
        let analytic_mean = mean(&analytic);

        let mut max_err = 0.0_f64;
        let mut max_ref = 0.0_f64;
        for (pn, pa) in p_num.iter().zip(analytic.iter()) {
            max_err = max_err.max(((pn - p_num_mean) - (pa - analytic_mean)).abs());
            max_ref = max_ref.max((pa - analytic_mean).abs());
        }
        rel_errors.push(max_err / max_ref);
    }

    assert!(
        rel_errors[1] < rel_errors[0] / 3.0 && rel_errors[2] < rel_errors[1] / 3.0,
        "static pressure not second order: {rel_errors:?}"
    );
}

/// `bernoulli − static = ½|u|²` up to machine rounding (the static field
/// is `b − k`, so `b − (b − k)` re-rounds), per precision.
fn assert_convention_identity<R>(tol: R)
where
    R: RealField
        + deep_causality_par::MaybeParallel
        + FromPrimitive
        + Default
        + PartialEq
        + core::fmt::Debug
        + core::fmt::Display,
{
    let n = 6usize;
    let manifold = unit_manifold::<R>(n);
    let solver = DecNsSolver::new(
        &manifold,
        R::from_f64(0.01).unwrap(),
        R::from_f64(0.05).unwrap(),
        None,
    )
    .unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let (bernoulli, static_p) = solver.pressure_diagnostic(&state).unwrap();

    // Recompute ½|u|² from sharp, exactly as the diagnostic does.
    let vertex_vectors = manifold.sharp(state.as_one_form()).unwrap();
    let half = R::from_f64(0.5).unwrap();
    for (i, chunk) in vertex_vectors.as_slice().chunks_exact(2).enumerate() {
        let kinetic = (chunk[0] * chunk[0] + chunk[1] * chunk[1]) * half;
        let diff = bernoulli.as_tensor().as_slice()[i] - static_p.as_tensor().as_slice()[i];
        let scale = if kinetic.abs() > R::one() {
            kinetic.abs()
        } else {
            R::one()
        };
        assert!(
            (diff - kinetic).abs() <= tol * scale,
            "convention identity broken at vertex {i}: {diff} vs {kinetic}"
        );
    }
}

#[test]
fn convention_identity_f64() {
    assert_convention_identity::<f64>(1e-12);
}

#[test]
fn convention_identity_f32() {
    assert_convention_identity::<f32>(1e-5);
}

#[test]
fn convention_identity_float106() {
    assert_convention_identity::<Float106>(Float106::from_f64(1e-12));
}

/// On the fully periodic Stage-1 lattice the diagnostic's grade-0 solve
/// is spectral and cannot fail to converge: a starved CG budget must not
/// error. CG error propagation is pinned at the topology layer on
/// mixed-periodicity lattices.
#[test]
fn diagnostic_spectral_path_ignores_cg_starvation() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let healthy = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = healthy
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let starved = DecNsSolver::new(&manifold, 0.01, 0.1, None)
        .unwrap()
        .with_cg_options(HodgeDecomposeOptions {
            tolerance: None,
            max_iterations: Some(1),
        });
    let result = starved.pressure_diagnostic(&state);
    assert!(result.is_ok(), "{result:?}");
}

/// On a mixed-periodicity (wall-bounded) lattice the diagnostic's grade-0
/// solve runs the iterative constrained CG, which a one-iteration budget
/// starves: the projection fails and the diagnostic surfaces it as a
/// `TopologyError` naming the pressure-diagnostic projection.
#[test]
fn diagnostic_reports_cg_failure_on_wall_bounded_lattice() {
    let lattice = LatticeComplex::<2, f64>::new([8, 6], [true, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    let manifold = Manifold::from_cubical_with_metric(lattice, data, metric, 0);

    let healthy = DecNsSolver::new(&manifold, 0.01, 0.05, None).unwrap();
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = (0.7 * x).sin() + 0.3 * y;
        vertex[2 * vi + 1] = (0.5 * y).cos() * 0.2;
    }
    let seed = CausalTensor::new(vertex, vec![2 * n0]).unwrap();
    let state = healthy.seed_from_vertex_vectors(&seed).unwrap();

    let starved = DecNsSolver::new(&manifold, 0.01, 0.05, None)
        .unwrap()
        .with_cg_options(HodgeDecomposeOptions {
            tolerance: Some(1e-14),
            max_iterations: Some(1),
        });
    let err = starved.pressure_diagnostic(&state).unwrap_err();
    assert!(
        err.to_string().contains("pressure diagnostic projection"),
        "{err}"
    );
}
