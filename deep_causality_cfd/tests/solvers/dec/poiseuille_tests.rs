/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Poiseuille channel rung of the validation ladder (dec-ns-validation,
//! the analytic-first gate of the wall substrate): body-force-driven
//! laminar flow between two no-slip walls (periodic-x, wall-y), marched to
//! steady state and compared against the exact parabolic profile
//! `u_x(y) = (G/2ν)·y·(H−y)`.
//!
//! With vertex-collocated walls the Dirichlet rows sit exactly on the
//! boundary and the 3-point viscous stencil is exact on quadratics, while
//! the convective term of an x-uniform shear vanishes under the
//! constrained projection (its y-component is an exact discrete gradient
//! and its x-component carries `u_y = 0`). The discrete steady state is
//! therefore the exact parabola at every resolution — the rung gates
//! **exactness at rounding**, which supersedes the spec's ≥ 1.9 observed
//! order (a stronger statement: the error has no resolvable h-dependence
//! to fit an order to).

use deep_causality_cfd::{BodyForceOneForm, DecNsSolver};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const NU: f64 = 0.1;
const HEIGHT: f64 = 1.0;

fn channel_manifold(ny: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let h = HEIGHT / (ny - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([4, ny], [true, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// March the channel from rest to steady state; return the sup error of
/// the x-edge profile against the exact parabola (relative to the
/// centerline maximum `G·H²/(8ν) = 1`).
fn poiseuille_profile_error(ny: usize) -> f64 {
    let m = channel_manifold(ny);
    let h = HEIGHT / (ny - 1) as f64;
    // G chosen for u_max = G·H²/(8ν) = 1.
    let g = 8.0 * NU;

    // Body force: G on every x-edge (edge integral G·h), zero on y-edges.
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();

    // Diffusive CFL: dt ≤ 0.9·h²/(4ν); advective is looser at u ≤ 1.
    let dt = 0.5 * h * h / (4.0 * NU);
    let solver = DecNsSolver::new(&m, NU, dt, Some(&force)).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();

    // March to stationarity (diffusion time H²/ν, with early exit).
    let mut previous = state.as_one_form().as_slice().to_vec();
    for _ in 0..40_000 {
        state = solver.step(&state).unwrap().into_state();
        let now = state.as_one_form().as_slice();
        let delta = now
            .iter()
            .zip(previous.iter())
            .fold(0.0f64, |acc, (a, b)| acc.max((a - b).abs()));
        if delta < 1e-13 {
            break;
        }
        previous = now.to_vec();
    }

    // Compare x-edge integrals against u_x(y)·h.
    let u = state.as_one_form().as_slice();
    let mut err = 0.0f64;
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize != 0 {
            continue;
        }
        let y = cell.position()[1] as f64 * h;
        let exact = (g / (2.0 * NU)) * y * (HEIGHT - y) * h;
        err = err.max((u[idx] - exact).abs() / h);
    }
    err
}

/// Wall-consistency at steady state: tangential edges exactly zero and the
/// divergence residual at the solve's exactness (the spec's second
/// scenario), checked on the middle rung.
#[test]
fn poiseuille_steady_state_is_wall_consistent() {
    let ny = 9;
    let m = channel_manifold(ny);
    let h = HEIGHT / (ny - 1) as f64;
    let g = 8.0 * NU;
    let n1 = m.complex().num_cells(1);
    let mut force = vec![0.0; n1];
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        if cell.orientation().trailing_zeros() as usize == 0 {
            force[idx] = g * h;
        }
    }
    let force = BodyForceOneForm::new(CausalTensor::new(force, vec![n1]).unwrap(), &m).unwrap();
    let dt = 0.5 * h * h / (4.0 * NU);
    let solver = DecNsSolver::new(&m, NU, dt, Some(&force)).unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();
    let mut residual = f64::INFINITY;
    for _ in 0..5_000 {
        let out = solver.step(&state).unwrap();
        residual = out.divergence_residual();
        state = out.into_state();
    }

    assert!(residual < 1e-9, "divergence residual {residual:e}");
    let shape = m.complex().shape();
    let u = state.as_one_form().as_slice();
    for (idx, cell) in m.complex().iter_cells(1).enumerate() {
        let axis = cell.orientation().trailing_zeros() as usize;
        let y = cell.position()[1];
        if axis == 0 && (y == 0 || y + 1 == shape[1]) {
            assert_eq!(u[idx], 0.0, "wall edge {idx} nonzero at steady state");
        }
    }
}

/// The refinement ladder: the steady profile reproduces the exact
/// parabola at rounding on every rung.
#[test]
fn poiseuille_profile_is_exact_over_the_ladder() {
    for ny in [5usize, 9, 17] {
        let err = poiseuille_profile_error(ny);
        assert!(
            err < 1e-8,
            "ny = {ny}: Poiseuille profile error {err:e} above rounding"
        );
    }
}
