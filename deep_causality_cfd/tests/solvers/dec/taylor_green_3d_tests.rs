/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation rung 5 (`cfd-gap.md` §7): the 2D-in-3D Taylor–Green vortex
//! (`w = 0`) on `cubic_torus` — the same analytic energy envelope while
//! every 3D operator path (wedge, star, interior product, projection,
//! CFL) is exercised, and the vertical velocity stays at projection
//! tolerance throughout the march.

use deep_causality_cfd::{DecNsSolver, dec_kinetic_energy};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.05;
const DT: f64 = 0.2;
/// 12 steps (T = 2.4) keep the CI cost bounded; the analytic envelope
/// ratio is ≈ 0.929 there — ample signal against the 2% gate and the
/// ≈ 0.1% discrete-Laplacian truncation.
const STEPS: usize = 12;
const N: usize = 16;

fn unit_manifold3(n: usize) -> Manifold<LatticeComplex<3, f64>, f64> {
    let lattice: LatticeComplex<3, f64> = LatticeComplex::cubic_torus(n);
    let total: usize = (0..=3).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<3, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// Maximum |coefficient| over z-aligned edges — the discrete `max|w|`.
fn max_w(manifold: &Manifold<LatticeComplex<3, f64>, f64>, edge_form: &[f64]) -> f64 {
    let mut max = 0.0_f64;
    for (i, cell) in manifold.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 2 {
            max = max.max(edge_form[i].abs());
        }
    }
    max
}

#[test]
fn tg_2d_in_3d_tracks_envelope_with_zero_w() {
    let manifold = unit_manifold3(N);
    let k = 2.0 * std::f64::consts::PI / (N as f64);
    let n0 = manifold.complex().num_cells(0);

    // w = 0 Taylor–Green, independent of z.
    let mut vertex = vec![0.0; 3 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let p = v.position();
        let (x, y) = (p[0] as f64, p[1] as f64);
        vertex[3 * vi] = (k * x).sin() * (k * y).cos();
        vertex[3 * vi + 1] = -(k * x).cos() * (k * y).sin();
        vertex[3 * vi + 2] = 0.0;
    }
    let vertex_tensor = CausalTensor::new(vertex, vec![3 * n0]).unwrap();

    let solver = DecNsSolver::new(&manifold, NU, DT, None).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&vertex_tensor).unwrap();
    let e0 = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();
    assert!(
        max_w(&manifold, state.as_one_form().as_slice()) < 1e-8,
        "seeded w must vanish"
    );

    // March, checking max|w| at every step.
    for _ in 0..STEPS {
        let output = solver.step(&state).unwrap();
        state = output.into_state();
        let w = max_w(&manifold, state.as_one_form().as_slice());
        assert!(w < 1e-8, "vertical velocity grew during the march: {w}");
    }

    let e_t = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();
    let t = DT * STEPS as f64;
    let analytic = (-4.0 * NU * k * k * t).exp();
    let err = ((e_t / e0) - analytic).abs() / analytic;
    assert!(
        err < 0.02,
        "3D envelope error {err} above the documented 2% gate"
    );
}
