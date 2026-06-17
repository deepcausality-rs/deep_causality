/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation rung 8 (`cfd-gap.md` §7): the 2D double shear layer
//! (Brown–Minion form) at one modest resolution, f64 only — a
//! physics-behavior rung, not a precision rung. No closed form exists, so
//! the gates are structural:
//!
//! 1. **Roll-up witness**: the cross-stream kinetic energy grows from its
//!    perturbation seed by at least an order of magnitude.
//! 2. **2D conservation character**: at `ν > 0`, kinetic energy and
//!    enstrophy are monotonically non-increasing within a documented
//!    relative slack of 1e-3 per sample (no vortex stretching in 2D), and
//!    every sampled state stays divergence-free at projection tolerance.
//! 3. **Coherent-structure tap**: the existing Q-criterion kernel, fed by
//!    a test-side central-difference gradient of the `sharp`-recovered
//!    field, reports vortex-core (positive-Q) cells in the rolled-up
//!    state that are absent at `t = 0` (a shear layer is pure shear:
//!    `‖S‖ = ‖Ω‖`, so Q ≈ 0 before roll-up).

use deep_causality_cfd::{
    DecNsSolver, SolenoidalField, VelocityGradient, dec_divergence_residual, dec_enstrophy,
    dec_kinetic_energy,
};
use deep_causality_physics::q_criterion_kernel;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const N: usize = 32;
const NU: f64 = 0.005;
/// `dt = 0.5` halves the CI step count against the original `dt = 0.25`
/// run; the advective limit (0.9·dx/max|u| with max|u| ≈ 1) holds, and
/// the measured cross-stream growth curve is identical at both steps
/// (×13.06 at `T = 20`).
const DT: f64 = 0.5;
const STEPS: usize = 40;
const SAMPLE_EVERY: usize = 5;
const PERTURBATION: f64 = 0.05;

fn unit_manifold(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let lattice: LatticeComplex<2, f64> = LatticeComplex::square_torus(n);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::unit();
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// Brown–Minion double shear layer with a small sinusoidal cross-stream
/// perturbation, scaled to the `[0, n]²` unit-spacing torus.
fn shear_layer_vertex_tensor(
    manifold: &Manifold<LatticeComplex<2, f64>, f64>,
    n: usize,
) -> CausalTensor<f64> {
    let nf = n as f64;
    let delta = nf / 30.0; // layer thickness
    let n0 = manifold.complex().num_cells(0);
    let mut vertex = vec![0.0; 2 * n0];
    for (vi, v) in manifold.complex().iter_cells(0).enumerate() {
        let (x, y) = (v.position()[0] as f64, v.position()[1] as f64);
        let u = if y <= nf / 2.0 {
            ((y - nf / 4.0) / delta).tanh()
        } else {
            ((3.0 * nf / 4.0 - y) / delta).tanh()
        };
        let w = PERTURBATION * (2.0 * std::f64::consts::PI * x / nf).sin();
        vertex[2 * vi] = u;
        vertex[2 * vi + 1] = w;
    }
    CausalTensor::new(vertex, vec![2 * n0]).unwrap()
}

/// Cross-stream (v-component) kinetic energy from the sharp-recovered
/// vertex vectors: the roll-up growth witness.
fn cross_stream_energy(
    manifold: &Manifold<LatticeComplex<2, f64>, f64>,
    state: &SolenoidalField<f64>,
) -> f64 {
    let vv = manifold.sharp(state.as_one_form()).unwrap();
    vv.as_slice()
        .chunks_exact(2)
        .map(|c| 0.5 * c[1] * c[1])
        .sum()
}

/// Counts vertices whose Q-criterion (from a central-difference gradient
/// of the sharp-recovered field) exceeds the threshold.
fn positive_q_cells(
    manifold: &Manifold<LatticeComplex<2, f64>, f64>,
    state: &SolenoidalField<f64>,
    threshold: f64,
) -> usize {
    let vv = manifold.sharp(state.as_one_form()).unwrap();
    let v = vv.as_slice();
    let n = N;
    let idx = |x: usize, y: usize| y * n + x; // axis-0-fastest vertex layout

    let mut count = 0usize;
    for y in 0..n {
        for x in 0..n {
            let xp = (x + 1) % n;
            let xm = (x + n - 1) % n;
            let yp = (y + 1) % n;
            let ym = (y + n - 1) % n;
            // grad[i][j] = ∂u_i/∂x_j by central differences (unit spacing).
            let mut g = [[0.0_f64; 3]; 3];
            for i in 0..2 {
                g[i][0] = (v[2 * idx(xp, y) + i] - v[2 * idx(xm, y) + i]) / 2.0;
                g[i][1] = (v[2 * idx(x, yp) + i] - v[2 * idx(x, ym) + i]) / 2.0;
            }
            let q = q_criterion_kernel(&VelocityGradient::new_unchecked(g)).unwrap();
            if q > threshold {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn double_shear_layer_rolls_up_with_2d_conservation_character() {
    let manifold = unit_manifold(N);
    let solver = DecNsSolver::new(&manifold, NU, DT, None).unwrap();
    let mut state = solver
        .seed_from_vertex_vectors(&shear_layer_vertex_tensor(&manifold, N))
        .unwrap();

    let ev0 = cross_stream_energy(&manifold, &state);
    assert!(ev0 > 0.0, "perturbation seed must carry energy");

    let q_threshold = 0.02;
    let q0 = positive_q_cells(&manifold, &state, q_threshold);
    assert_eq!(
        q0, 0,
        "a pure shear layer must report no vortex cores at t = 0"
    );

    let mut e_prev = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();
    let mut z_prev = dec_enstrophy(&manifold, state.as_one_form()).unwrap();

    // Documented relative slack on monotonicity per sample.
    let slack = 1e-3;

    for step in 1..=STEPS {
        let output = solver.step(&state).unwrap();
        state = output.into_state();

        if step % SAMPLE_EVERY == 0 {
            let e = dec_kinetic_energy(&manifold, state.as_one_form()).unwrap();
            let z = dec_enstrophy(&manifold, state.as_one_form()).unwrap();
            let r = dec_divergence_residual(&manifold, state.as_one_form()).unwrap();

            assert!(
                e <= e_prev * (1.0 + slack),
                "kinetic energy rose at step {step}: {e_prev} -> {e}"
            );
            assert!(
                z <= z_prev * (1.0 + slack),
                "enstrophy rose at step {step}: {z_prev} -> {z}"
            );
            assert!(r < 1e-8, "divergence residual {r} at step {step}");

            e_prev = e;
            z_prev = z;
        }
    }

    // Roll-up witness: an order of magnitude of cross-stream growth.
    let ev_t = cross_stream_energy(&manifold, &state);
    assert!(
        ev_t >= 10.0 * ev0,
        "no roll-up: cross-stream energy {ev0} -> {ev_t}"
    );

    // Vortex cores where none existed.
    let q_t = positive_q_cells(&manifold, &state, q_threshold);
    assert!(
        q_t > 0,
        "rolled-up state must report positive-Q vortex cores"
    );
}
