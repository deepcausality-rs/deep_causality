/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Seeding tests: the de Rham + one-projection path produces a
//! divergence-free state with the analytic energy; the exact-integrals
//! variant agrees; every rejection branch fires.

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_physics::{DecNsSolver, dec_divergence_residual, dec_kinetic_energy};
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
fn tg_seed_is_divergence_free_with_analytic_energy() {
    let n = 16usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();

    let state = solver
        .seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n))
        .unwrap();

    let residual = dec_divergence_residual(&manifold, state.as_one_form()).unwrap();
    assert!(residual < 1e-8, "seed not divergence-free: {residual}");

    let e = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();
    let analytic = (n * n) as f64 / 4.0;
    assert!(
        (e - analytic).abs() / analytic < 0.05,
        "seed energy {e} not within 5% of analytic {analytic}"
    );
}

#[test]
fn integral_seeding_agrees_with_vertex_seeding() {
    let n = 8usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();

    let vertex = tg_vertex_tensor(&manifold, n);
    let from_vertices = solver.seed_from_vertex_vectors(&vertex).unwrap();

    // Feeding the de Rham output back through the validating integral
    // passthrough must reproduce the same state.
    let integrals = manifold.de_rham(&vertex).unwrap();
    let from_integrals = solver.seed_from_edge_integrals(&integrals).unwrap();

    assert_eq!(from_vertices, from_integrals);
}

#[test]
fn rejects_wrong_vertex_length() {
    let manifold = unit_manifold::<f64>(6);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let bad = CausalTensor::new(vec![0.0; 5], vec![5]).unwrap();
    let err = solver.seed_from_vertex_vectors(&bad).unwrap_err();
    assert!(err.to_string().contains("de Rham seeding failed"), "{err}");
}

#[test]
fn rejects_wrong_integral_length() {
    let manifold = unit_manifold::<f64>(6);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let bad = CausalTensor::new(vec![0.0; 5], vec![5]).unwrap();
    let err = solver.seed_from_edge_integrals(&bad).unwrap_err();
    assert!(
        err.to_string().contains("de Rham integral seeding failed"),
        "{err}"
    );
}

#[test]
fn rejects_non_finite_samples() {
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let solver = DecNsSolver::new(&manifold, 0.01, 0.1, None).unwrap();
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    vertex[3] = f64::NAN;
    let t = CausalTensor::new(vertex, vec![2 * n0]).unwrap();
    let err = solver.seed_from_vertex_vectors(&t).unwrap_err();
    assert!(err.to_string().contains("finite"), "{err}");
}

/// On the fully periodic Stage-1 lattice the seed projection runs the
/// spectral grade-0 solve, which has no convergence-failure mode: a
/// starved CG budget must not fail. CG error propagation is pinned at
/// the topology layer on mixed-periodicity lattices.
#[test]
fn seed_projection_spectral_path_ignores_cg_starvation() {
    use deep_causality_topology::HodgeDecomposeOptions;
    let n = 6usize;
    let manifold = unit_manifold::<f64>(n);
    let starved = DecNsSolver::new(&manifold, 0.01, 0.1, None)
        .unwrap()
        .with_cg_options(HodgeDecomposeOptions {
            tolerance: None,
            max_iterations: Some(1),
        });
    let result = starved.seed_from_vertex_vectors(&tg_vertex_tensor(&manifold, n));
    assert!(result.is_ok(), "{result:?}");
}
