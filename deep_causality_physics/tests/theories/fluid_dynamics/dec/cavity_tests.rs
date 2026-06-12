/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The coarse lid-driven-cavity rung of the validation ladder
//! (dec-ns-validation): Re-1000 cavity on a coarse all-walls grid, marched
//! from rest, compared against the Ghia, Ghia & Shin (1982) centerline
//! tables (J. Comput. Phys. 48, 387–411; Re = 1000 columns) with a pinned
//! RMSE gate and an asserted refinement trend. The full-resolution run
//! with the vortex-center table ships as an example program.

use deep_causality_physics::DecNsSolver;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, CubicalReggeGeometry, LatticeComplex, Manifold};

const RE: f64 = 1000.0;
const LID_SPEED: f64 = 1.0;

/// Ghia et al. (1982), Table I, Re = 1000: u along the vertical centerline
/// through the geometric center, as (y, u).
const GHIA_U: [(f64, f64); 17] = [
    (1.0000, 1.00000),
    (0.9766, 0.65928),
    (0.9688, 0.57492),
    (0.9609, 0.51117),
    (0.9531, 0.46604),
    (0.8516, 0.33304),
    (0.7344, 0.18719),
    (0.6172, 0.05702),
    (0.5000, -0.06080),
    (0.4531, -0.10648),
    (0.2813, -0.27805),
    (0.1719, -0.38289),
    (0.1016, -0.29730),
    (0.0703, -0.22220),
    (0.0625, -0.20196),
    (0.0547, -0.18109),
    (0.0000, 0.00000),
];

/// Ghia et al. (1982), Table II, Re = 1000: v along the horizontal
/// centerline, as (x, v).
const GHIA_V: [(f64, f64); 17] = [
    (1.0000, 0.00000),
    (0.9688, -0.21388),
    (0.9609, -0.27669),
    (0.9531, -0.33714),
    (0.9453, -0.39188),
    (0.9063, -0.51500),
    (0.8594, -0.42665),
    (0.8047, -0.31966),
    (0.5000, 0.02526),
    (0.2344, 0.32235),
    (0.2266, 0.33075),
    (0.1563, 0.37095),
    (0.0938, 0.32627),
    (0.0781, 0.30353),
    (0.0703, 0.29012),
    (0.0625, 0.27485),
    (0.0000, 0.00000),
];

fn cavity_manifold(n: usize) -> Manifold<LatticeComplex<2, f64>, f64> {
    let h = 1.0 / (n - 1) as f64;
    let lattice = LatticeComplex::<2, f64>::new([n, n], [false, false]);
    let total: usize = (0..=2).map(|k| lattice.num_cells(k)).sum();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let metric: CubicalReggeGeometry<2, f64> = CubicalReggeGeometry::uniform(h);
    Manifold::from_cubical_with_metric(lattice, data, metric, 0)
}

/// March the Re-1000 cavity from rest to horizon `t_end`; return the
/// centerline RMSE against the Ghia tables (u and v pooled).
fn cavity_centerline_rmse(n: usize, t_end: f64) -> f64 {
    let m = cavity_manifold(n);
    let h = 1.0 / (n - 1) as f64;
    let nu = LID_SPEED / RE;
    // Advective CFL binds (u ≤ 1): dt = 0.45·h; diffusive limit is far
    // looser at this ν.
    let dt = 0.45 * h;
    // The projection tolerance is loosened to 1e-6 for CI economy: the
    // gate compares against reference data at the 1e-1 scale, so solve
    // noise five orders below it is invisible while the Jacobi-PCG
    // iteration count (the rung's dominant cost) roughly halves.
    let opts = deep_causality_topology::HodgeDecomposeOptions {
        tolerance: Some(1e-6),
        max_iterations: Some(10_000),
    };
    let solver = DecNsSolver::new(&m, nu, dt, None)
        .unwrap()
        .with_cg_options(opts)
        .with_moving_wall(1, true, [LID_SPEED, 0.0])
        .unwrap();

    let n0 = m.complex().num_cells(0);
    let rest = CausalTensor::new(vec![0.0; 2 * n0], vec![2 * n0]).unwrap();
    let mut state = solver.seed_from_vertex_vectors(&rest).unwrap();
    let steps = (t_end / dt).ceil() as usize;
    for _ in 0..steps {
        state = solver.step(&state).unwrap().into_state();
    }

    // Edge-value samplers: average the two edges flanking the center
    // column/row (n odd ⇒ x = 0.5 is a vertex column), divide by h to
    // convert edge integrals to velocities, then interpolate linearly to
    // the Ghia stations.
    let u = state.as_one_form().as_slice();
    let nx_edges = (n - 1) * n; // x-edges come first (orientation-major)
    let center = (n - 1) / 2;
    let x_edge = |i: usize, j: usize| u[j * (n - 1) + i] / h;
    let y_edge = |i: usize, j: usize| u[nx_edges + j * n + i] / h;

    // u(0.5, y_j) at vertex rows j.
    let u_profile: Vec<f64> = (0..n)
        .map(|j| 0.5 * (x_edge(center - 1, j) + x_edge(center, j)))
        .collect();
    // v(x_i, 0.5) at vertex columns i.
    let v_profile: Vec<f64> = (0..n)
        .map(|i| 0.5 * (y_edge(i, center - 1) + y_edge(i, center)))
        .collect();

    let interp = |profile: &[f64], s: f64| -> f64 {
        let pos = s / h;
        let i0 = (pos.floor() as usize).min(n - 2);
        let w = pos - i0 as f64;
        profile[i0] * (1.0 - w) + profile[i0 + 1] * w
    };

    let mut sq = 0.0;
    let mut count = 0usize;
    for &(y, u_ref) in &GHIA_U {
        let diff = interp(&u_profile, y) - u_ref;
        sq += diff * diff;
        count += 1;
    }
    for &(x, v_ref) in &GHIA_V {
        let diff = interp(&v_profile, x) - v_ref;
        sq += diff * diff;
        count += 1;
    }
    (sq / count as f64).sqrt()
}

/// The coarse CI rung: the pinned RMSE gate at the time-converged 17²
/// resolution (measured 0.2523 at pinning time; the gate carries ~25 %
/// headroom for cross-platform drift). Runs in every CI pass (~16 s
/// debug). The refinement-trend companion below carries the 33² rung.
#[test]
fn coarse_cavity_gates_against_ghia() {
    let rmse = cavity_centerline_rmse(17, 20.0);
    println!("cavity RMSE: 17²={rmse:.5}");
    assert!(
        rmse < 0.32,
        "17² centerline RMSE {rmse:.4} above the pinned gate 0.32"
    );
}

/// The refinement-trend rung: 17² → 33² centerline RMSE must decrease
/// (measured 0.2523 → 0.1730 at the T = 20 spin-up; time-converged
/// values 0.252 / 0.133). The 33² march is the ladder's most expensive
/// rung (~11 s release, minutes unoptimized). The full-resolution
/// comparison ships as the `dec_lid_cavity_re1000` example program.
#[test]
fn cavity_refinement_trend_against_ghia() {
    let coarse = cavity_centerline_rmse(17, 20.0);
    let fine = cavity_centerline_rmse(33, 20.0);
    println!("cavity RMSE: 17²={coarse:.5} 33²={fine:.5}");
    assert!(
        fine < 0.22,
        "33² centerline RMSE {fine:.4} above the pinned gate 0.22"
    );
    assert!(
        fine < coarse - 0.04,
        "refinement trend broken: 33² RMSE {fine:.4} vs 17² RMSE {coarse:.4}"
    );
}
