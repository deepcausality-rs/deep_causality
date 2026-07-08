/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Causal-monad wrapper tests: success lifts to `pure`, failure converts
//! to a `CausalityError` carrier.

use deep_causality_algebra::RealField;
use deep_causality_cfd::{DecNsSolver, dec_ns_step};
use deep_causality_num::FromPrimitive;
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

fn tg_vertex_tensor(
    manifold: &Manifold<LatticeComplex<2, f64>, f64>,
    n: usize,
) -> CausalTensor<f64> {
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = (k * x).sin() * (k * y).cos();
        vertex[2 * vi + 1] = -(k * x).cos() * (k * y).sin();
    }
    CausalTensor::new(vertex, vec![2 * n0]).unwrap()
}

#[test]
fn wrapper_lifts_success_to_pure() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let effect = dec_ns_step(&solver, &state);
    assert!(effect.is_ok());
}

#[test]
fn wrapper_converts_failure_to_causality_error() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    // CFL violation as the failure injector: the spectral projection on
    // this periodic lattice cannot fail, so an over-long dt against the
    // unit-speed field trips the advective guard inside the step.
    let solver = DecNsSolver::new(&manifold, 0.0, 2.0, None).unwrap();
    let seeder = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = seeder
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let effect = dec_ns_step(&solver, &state);
    assert!(effect.is_err());
}
