/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Step tests: divergence-free output at every precision, the inviscid
//! zero-state fixed point, CG short-circuit, both CFL violations with
//! their messages, and the residual-consistency contract.

use deep_causality_num::{Float106, FromPrimitive, RealField};
use deep_causality_physics::{DecNsSolver, dec_divergence_residual};
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

/// One step from the seeded Taylor–Green state stays divergence-free at
/// the precision's projection tolerance.
fn assert_step_divergence_free<R>(tol: R)
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
        R::from_f64(0.1).unwrap(),
        None,
    )
    .unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let output = solver.step(&state).unwrap();
    assert!(
        output.divergence_residual() <= tol,
        "divergence residual {} above tolerance {tol}",
        output.divergence_residual()
    );
}

#[test]
fn step_divergence_free_f64() {
    assert_step_divergence_free::<f64>(1e-8);
}

#[test]
fn step_divergence_free_f32() {
    assert_step_divergence_free::<f32>(1e-3);
}

#[test]
fn step_divergence_free_float106() {
    assert_step_divergence_free::<Float106>(Float106::from_f64(1e-8));
}

/// The zero state with `ν = 0` and no body force is a fixed point: both
/// CFL guards skip and the state passes through unchanged.
#[test]
fn zero_state_inviscid_fixed_point() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.0, 0.1, None).unwrap();
    let n0 = manifold.complex().num_cells(0);
    let zero_vertex = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let state = solver.seed_from_vertex_vectors(&zero_vertex).unwrap();

    let output = solver.step(&state).unwrap();
    assert_eq!(output.max_speed(), 0.0);
    assert_eq!(output.state(), &state);
}

/// On the fully periodic Stage-1 lattice the projection is spectral and
/// has no convergence-failure mode: a starved CG budget must not fail
/// the step. Step-level failure propagation is pinned by the CFL test
/// below; CG error propagation by the topology-layer mixed-periodicity
/// tests.
#[test]
fn spectral_path_ignores_cg_starvation_in_step() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let seeder = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = seeder
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let starved = DecNsSolver::new(&manifold, 0.01, 0.1, None)
        .unwrap()
        .with_cg_options(HodgeDecomposeOptions {
            tolerance: None,
            max_iterations: Some(1),
        });
    let result = starved.step(&state);
    assert!(result.is_ok(), "{result:?}");
}

/// An over-long `dt` against a unit-speed field trips the advective
/// limit, and the message names both the limit and the actual `dt`.
#[test]
fn advective_cfl_violation_names_both_numbers() {
    let n = 8usize;
    let manifold = unit_manifold::<f64>(n);
    // ν = 0 keeps the diffusive guard out of the picture.
    let solver = DecNsSolver::new(&manifold, 0.0, 2.0, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let err = solver.step(&state).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("advective"), "{msg}");
    assert!(msg.contains("dt 2"), "{msg}");
    assert!(msg.contains("limit"), "{msg}");
}

/// Large viscosity with an over-long `dt` trips the diffusive limit even
/// on a state at rest (the advective guard skips at zero speed).
#[test]
fn diffusive_cfl_violation_at_rest() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 10.0, 1.0, None).unwrap();
    let n0 = manifold.complex().num_cells(0);
    let zero_vertex = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let state = solver.seed_from_vertex_vectors(&zero_vertex).unwrap();

    let err = solver.step(&state).unwrap_err();
    assert!(err.to_string().contains("diffusive"), "{err}");
}

/// The residual carried by the step output equals an independent direct
/// evaluation on the returned state.
#[test]
fn reported_residual_matches_direct_evaluation() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let output = solver.step(&state).unwrap();
    let direct = dec_divergence_residual(&manifold, output.state().as_one_form()).unwrap();
    assert_eq!(output.divergence_residual(), direct);
}
