/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation rung 6 (`cfd-gap.md` §7): inviscid invariants. At `ν = 0`
//! the rotational-form/DEC march should conserve kinetic energy (2D and
//! 3D) and helicity (3D); the bounds below are **measured drift bounds,
//! recorded here**, not theoretical guarantees — the structure-preserving
//! behavior is asserted, not assumed.

use deep_causality_physics::{DecNsSolver, dec_helicity, dec_kinetic_energy};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

fn unit_manifold2(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

fn unit_manifold3(n: usize) -> Manifold<LatticeComplex<3, f64>, f64> {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(n);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// 3D ABC (Beltrami) flow: nonzero helicity, an exact steady Euler
/// solution in the continuum. Recorded drift bounds over 20 steps at
/// `dt = 0.05`: energy 1e-2 relative, helicity 1e-2 relative.
#[test]
fn abc_flow_conserves_energy_and_helicity() {
    let n = 8usize;
    let manifold = unit_manifold3(n);
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);

    let mut vertex = vec![0.0; 3 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let p = v.position();
        let (x, y, z) = (p[0] as f64, p[1] as f64, p[2] as f64);
        vertex[3 * vi] = (k * z).sin() + (k * y).cos();
        vertex[3 * vi + 1] = (k * x).sin() + (k * z).cos();
        vertex[3 * vi + 2] = (k * y).sin() + (k * x).cos();
    }
    let vertex_tensor = CausalTensor::new(vertex, vec![3 * n0]).unwrap();

    let solver = DecNsSolver::new(&manifold, 0.0, 0.05, None).unwrap();
    let state = solver.seed_from_vertex_vectors(&vertex_tensor).unwrap();

    let e0 = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();
    let h0 = dec_helicity(&manifold, state.as_one_form()).unwrap();
    assert!(h0 > 0.0, "ABC helicity must be positive at t = 0");

    let run = solver.run_n(state, 20).unwrap();
    let e_t = dec_kinetic_energy(&manifold, run.state().as_one_form()).unwrap();
    let h_t = dec_helicity(&manifold, run.state().as_one_form()).unwrap();

    let e_drift = (e_t - e0).abs() / e0;
    let h_drift = (h_t - h0).abs() / h0;
    assert!(
        e_drift <= 1e-2,
        "energy drift {e_drift} above recorded 1e-2"
    );
    assert!(
        h_drift <= 1e-2,
        "helicity drift {h_drift} above recorded 1e-2"
    );
}

/// 2D inviscid Taylor–Green: energy drift bound (recorded: 2e-2 relative
/// over 20 steps at `dt = 0.1` on the 16² torus — the discrete convective
/// residue is the second-order spatial error, not a temporal leak).
#[test]
fn tg_2d_inviscid_energy_drift_bounded() {
    let n = 16usize;
    let manifold = unit_manifold2(n);
    let k = 2.0 * std::f64::consts::PI / (n as f64);
    let n0 = manifold.complex().num_cells(0);

    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        vertex[2 * vi] = (k * x).sin() * (k * y).cos();
        vertex[2 * vi + 1] = -(k * x).cos() * (k * y).sin();
    }
    let vertex_tensor = CausalTensor::new(vertex, vec![2 * n0]).unwrap();

    let solver = DecNsSolver::new(&manifold, 0.0, 0.1, None).unwrap();
    let state = solver.seed_from_vertex_vectors(&vertex_tensor).unwrap();
    let e0 = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();

    let run = solver.run_n(state, 20).unwrap();
    let e_t = dec_kinetic_energy(&manifold, run.state().as_one_form()).unwrap();

    let drift = (e_t - e0).abs() / e0;
    assert!(
        drift <= 2e-2,
        "2D inviscid energy drift {drift} above recorded 2e-2"
    );
}
