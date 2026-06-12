/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Run-loop tests: fixed-horizon count, predicate stop, bound exhaustion,
//! and the failing-step index in both loops.

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_physics::DecNsSolver;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

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

fn tg_state(
    solver: &DecNsSolver<'_, 2, f64>,
    manifold: &Manifold<LatticeComplex<2, f64>, f64>,
    n: usize,
) -> deep_causality_physics::SolenoidalField<f64> {
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = (k * x).sin() * (k * y).cos();
        vertex[2 * vi + 1] = -(k * x).cos() * (k * y).sin();
    }
    let t = CausalTensor::new(vertex, vec![2 * n0]).unwrap();
    solver.seed_from_vertex_vectors(&t).unwrap()
}

#[test]
fn run_n_marches_exactly_n_steps() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = tg_state(&solver, &manifold, n);

    let run = solver.run_n(state, 4).unwrap();
    assert_eq!(run.steps(), 4);
    assert!(run.satisfied());
}

#[test]
fn run_until_stops_on_predicate() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = tg_state(&solver, &manifold, n);

    // Stop on accumulated simulated time: i·dt ≥ 0.3 → step 3.
    let dt = solver.dt();
    let run = solver
        .run_until(state, |i, _out| (i as f64) * dt >= 0.3, 10)
        .unwrap();
    assert_eq!(run.steps(), 3);
    assert!(run.satisfied());
}

#[test]
fn run_until_exhausts_bound() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = tg_state(&solver, &manifold, n);

    let run = solver.run_until(state, |_, _| false, 5).unwrap();
    assert_eq!(run.steps(), 5);
    assert!(!run.satisfied());
}

#[test]
fn run_n_error_names_the_failing_step() {
    // CFL violation as the failure injector: the spectral projection on
    // this periodic lattice cannot fail, so an over-long dt against the
    // unit-speed field trips the advective guard at the first step.
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let healthy = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = tg_state(&healthy, &manifold, n);

    let cfl_violating = DecNsSolver::new(&manifold, 0.0, 2.0, None).unwrap();
    let err = cfl_violating.run_n(state, 3).unwrap_err();
    assert!(err.to_string().contains("at step 1"), "{err}");
}

#[test]
fn run_until_error_names_the_failing_step() {
    // Same CFL injector as `run_n_error_names_the_failing_step`.
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let healthy = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = tg_state(&healthy, &manifold, n);

    let cfl_violating = DecNsSolver::new(&manifold, 0.0, 2.0, None).unwrap();
    let err = cfl_violating.run_until(state, |_, _| false, 3).unwrap_err();
    assert!(err.to_string().contains("at step 1"), "{err}");
}
