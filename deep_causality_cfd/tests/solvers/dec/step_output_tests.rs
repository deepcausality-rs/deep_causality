/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `StepOutput` / `RunOutput` accessor tests. Both types are produced only
//! by the solver, so the fixtures run one real (tiny) march.

use deep_causality_cfd::{DecNsSolver, dec_divergence_residual};
use deep_causality_num::{FromPrimitive, RealField};
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
fn step_output_accessors_are_consistent() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let output = solver.step(&state).unwrap();

    // Getters agree with independent evaluation on the carried state.
    let residual = dec_divergence_residual(&manifold, output.state().as_one_form()).unwrap();
    assert_eq!(output.divergence_residual(), residual);
    assert!(output.max_speed() > 0.0);

    // Clone/Debug/PartialEq derives and into_state.
    let clone = output.clone();
    assert_eq!(clone, output);
    assert!(!format!("{output:?}").is_empty());
    let state2 = output.into_state();
    assert_eq!(&state2, clone.state());
}

#[test]
fn run_output_accessors_are_consistent() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let run = solver.run_n(state, 2).unwrap();
    assert_eq!(run.steps(), 2);
    assert!(run.satisfied());
    assert!(!run.state().is_empty());

    let clone = run.clone();
    assert_eq!(clone, run);
    assert!(!format!("{run:?}").is_empty());
    let final_state = run.into_state();
    assert_eq!(&final_state, clone.state());
}
