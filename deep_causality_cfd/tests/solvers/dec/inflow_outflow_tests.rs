/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Groups I + O — the inflow and outflow boundary zones (`add-boundary-zone-abstraction`).
//!
//! The solver-level analytic gate: a uniform `Inflow` on the west face against an `Outflow`
//! pressure reference on the east face of a periodic-y channel marches to the exact uniform,
//! divergence-free flow, with the free outflow flux balancing the prescribed inflow (mass
//! conservation). The inflow types never enter the solver core — they are folded into the
//! `with_zones` builder.

use deep_causality_cfd::{DecNsSolver, Inflow, Outflow};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.1;
const U_IN: f64 = 0.5;

fn channel(nx: usize, ny: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    // x: open (inflow west / outflow east), y: periodic — the exact answer is uniform.
    let lattice = LatticeComplex::<2, f64>::new([nx, ny], [false, true]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(1.0);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

#[test]
fn uniform_inflow_outflow_channel_marches_to_uniform_flow() {
    let (nx, ny) = (6, 4);
    let m = channel(nx, ny);
    let dt = 0.2 * 1.0 / (4.0 * NU); // diffusive-safe at h = 1.

    // West inflow (axis 0, min face) + east outflow reference (axis 0, max face).
    let zones = (
        Inflow::<2, f64>::new(0, false, U_IN).unwrap(),
        Outflow::<2>::new(0, true).unwrap(),
    );
    let solver = DecNsSolver::with_zones(&m, NU, dt, zones).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    // March to stationarity (early-exit on a settled field).
    let mut prev = state.as_one_form().as_slice().to_vec();
    let mut converged = false;
    for _ in 0..5000 {
        state = solver.step(&state).unwrap().into_state();
        let now = state.as_one_form().as_slice();
        let delta = now
            .iter()
            .zip(prev.iter())
            .fold(0.0f64, |acc, (a, b)| acc.max((a - b).abs()));
        if delta < 1e-11 {
            converged = true;
            break;
        }
        prev = now.to_vec();
    }
    assert!(
        converged,
        "the inflow/outflow channel did not reach steady state"
    );

    // Steady state is uniform `u_x = U_IN` (edge integrals at h = 1), `u_y = 0`.
    let u = state.as_one_form().as_slice();
    for (i, c) in m.complex().iter_cells(1).enumerate() {
        if c.orientation().trailing_zeros() == 0 {
            assert!(
                (u[i] - U_IN).abs() < 1e-6,
                "x-edge {i} not uniform: {} vs {U_IN}",
                u[i]
            );
        } else {
            assert!(u[i].abs() < 1e-6, "y-edge {i} nonzero: {}", u[i]);
        }
    }

    // Divergence-free at interior vertices (open inlet/outlet columns carry boundary flux).
    let codiff = m.codifferential_of(u, 1).into_vec();
    let interior_div = m
        .complex()
        .iter_cells(0)
        .enumerate()
        .filter_map(|(i, c)| {
            let x = c.position()[0];
            (x > 0 && x + 1 < nx).then_some(codiff[i].abs())
        })
        .fold(0.0f64, f64::max);
    assert!(
        interior_div < 1e-6,
        "interior divergence {interior_div:e} above tolerance"
    );
}
