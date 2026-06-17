/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Lid-driven cavity at Re 1000, DEC-native
//!
//! The square cavity with a moving lid is the canonical wall-bounded
//! incompressible benchmark: the steady state at Re 1000 has a primary
//! vortex near the center and counter-rotating eddies in the bottom
//! corners, tabulated by Ghia, Ghia & Shin (1982), J. Comput. Phys. 48,
//! 387–411 — the reference every cavity solver is compared against.
//!
//! The wall substrate of the DEC solver appears together here:
//!
//! - **No-slip walls**: all four boundaries are walls; the lid (the y-max
//!   face) carries the tangential velocity `U = 1` through the solver's
//!   `with_moving_wall` lift, the other walls pin tangential velocity to
//!   zero.
//! - **The constrained Leray projector**: each stage projects onto the
//!   intersection of the divergence-free and no-slip subspaces, so the
//!   march keeps both invariants exactly at every step boundary.
//! - **The boundary-corrected Hodge star** supplies the clipped dual
//!   volumes that make the wall operators M-symmetric.
//!
//! Usage:
//!
//! ```text
//! cargo run --release --example dec_lid_cavity_re1000 [grid] [t_end]
//! cargo run --release --example dec_lid_cavity_re1000 trend
//! ```
//!
//! `grid` defaults to 65 (minutes of runtime); the reporting resolution is
//! 129 with `t_end ≥ 150` (hours — Ghia's own grid). The `trend` mode is
//! the refinement-trend verification (17² → 33² at time-converged
//! horizons, gated, nonzero exit on violation) — it lives here rather
//! than in the test suite because tests stay fast by design while
//! verification runs as long as it needs. Output:
//!
//! - `cavity_centerline_u.csv` / `cavity_centerline_v.csv` — computed
//!   centerline profiles at every grid station plus the Ghia stations with
//!   reference values and pointwise differences.
//! - stdout — the run header, the centerline RMSE, and the detected
//!   vortex centers (primary and bottom corner eddies, located at the
//!   streamfunction extrema) against Ghia's node-snapped values.

use std::env;

use deep_causality_cfd::{CfdConfigBuilder, CfdFlow, IoAction, Mesh, Seed, fail, write_csv};

const RE: f64 = 1000.0;
const LID_SPEED: f64 = 1.0;

/// Ghia et al. (1982), Re = 1000: u along the vertical centerline, (y, u).
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

/// Ghia et al. (1982), Re = 1000: v along the horizontal centerline, (x, v).
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

/// Ghia et al. (1982), Re = 1000 vortex centers (node-snapped to their
/// 129² grid): (name, x, y).
const GHIA_VORTICES: [(&str, f64, f64); 3] = [
    ("primary", 0.5313, 0.5625),
    ("bottom-left", 0.0859, 0.0781),
    ("bottom-right", 0.8594, 0.1094),
];

/// March the Re-1000 cavity from rest on an `n × n` grid to `t_end`; returns the final edge
/// cochain (velocity 1-form coefficients). The compute runs through the `deep_causality_cfd`
/// **CfdFlow** DSL — an all-walls box mesh, the DEC incompressible solver at `ν = U/Re`, the moving
/// lid, and a rest seed — which lowers onto the same projected DEC step the hand-rolled loop used,
/// so the marched field is reproduced exactly. The per-step progress line is a `run_with` hook; the
/// final field is taken from the report. The centerline/vortex analysis and CSV stay here (the file
/// writes await the IO monad).
fn march(n: usize, t_end: f64) -> Vec<f64> {
    let h = 1.0 / (n - 1) as f64;
    let nu = LID_SPEED / RE;
    let dt = 0.45 * h;
    let steps = (t_end / dt).ceil() as usize;

    // Configuration (the "what"): all-walls unit square at spacing `h`, the DEC solver, the y-max
    // lid, marched `steps` steps from rest.
    let config = CfdConfigBuilder::march::<2, f64>("cavity-re1000")
        .mesh(Mesh::box_domain([n, n]).spacing(h))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(nu)
                .time_step(dt)
                .build()
                .expect("cavity solver configuration"),
        )
        .lid([LID_SPEED, 0.0])
        .seed(Seed::Rest)
        .march_for(steps)
        .build()
        .expect("cavity march configuration");

    // B1: the caller owns the geometry; `CfdFlow` borrows it for the run.
    let manifold = config.materialize().expect("cavity geometry");

    let report_every = (steps / 20).max(1);
    let report = CfdFlow::march(&config)
        .on(&manifold)
        .run_with(|step: &deep_causality_cfd::StepView<'_, 2, f64>| {
            let s = step.step();
            if s % report_every == 0 {
                eprintln!("# t = {:8.2} ({s}/{steps})", step.time());
            }
        })
        .expect("cavity march");

    report
        .final_field()
        .expect("the marching report carries the final field")
        .to_vec()
}

/// The centerline velocity profiles `(u(0.5, y_j), v(x_i, 0.5))` at the
/// vertex stations, from the edge cochain.
fn centerline_profiles(u: &[f64], n: usize, h: f64) -> (Vec<f64>, Vec<f64>) {
    let nx_edges = (n - 1) * n;
    let center = (n - 1) / 2;
    let x_edge = |i: usize, j: usize| u[j * (n - 1) + i] / h;
    let y_edge = |i: usize, j: usize| u[nx_edges + j * n + i] / h;
    let u_profile: Vec<f64> = (0..n)
        .map(|j| 0.5 * (x_edge(center - 1, j) + x_edge(center, j)))
        .collect();
    let v_profile: Vec<f64> = (0..n)
        .map(|i| 0.5 * (y_edge(i, center - 1) + y_edge(i, center)))
        .collect();
    (u_profile, v_profile)
}

/// Linear interpolation of a vertex-station profile to coordinate `s`.
fn interp_profile(profile: &[f64], h: f64, s: f64) -> f64 {
    let n = profile.len();
    let pos = s / h;
    let i0 = (pos.floor() as usize).min(n - 2);
    let w = pos - i0 as f64;
    profile[i0] * (1.0 - w) + profile[i0 + 1] * w
}

/// Pooled centerline RMSE against the Ghia tables.
fn centerline_rmse(u_profile: &[f64], v_profile: &[f64], h: f64) -> f64 {
    let mut sq = 0.0;
    for &(y, u_ref) in &GHIA_U {
        let d = interp_profile(u_profile, h, y) - u_ref;
        sq += d * d;
    }
    for &(x, v_ref) in &GHIA_V {
        let d = interp_profile(v_profile, h, x) - v_ref;
        sq += d * d;
    }
    (sq / (GHIA_U.len() + GHIA_V.len()) as f64).sqrt()
}

/// The refinement-trend verification (moved here from the test suite:
/// tests stay fast, thorough verification runs as long as it needs).
/// Time-converged horizons; exits nonzero on a violated gate.
fn run_trend() {
    println!("# DEC lid-driven cavity, Re = {RE}: refinement trend vs Ghia (1982)");
    const T_END: f64 = 60.0; // time-converged for both grids
    let mut results: Vec<(usize, f64)> = Vec::new();
    for n in [17usize, 33] {
        let h = 1.0 / (n - 1) as f64;
        let u_form = march(n, T_END);
        let (u_p, v_p) = centerline_profiles(&u_form, n, h);
        let rmse = centerline_rmse(&u_p, &v_p, h);
        println!("grid {n:>3}², t_end {T_END}: centerline RMSE = {rmse:.4}");
        results.push((n, rmse));
    }
    // Gates from the pinning measurements (time-converged 0.252 / 0.133,
    // ~25 % headroom) plus the strict refinement-trend margin.
    let coarse = results[0].1;
    let fine = results[1].1;
    let mut failed = false;
    if coarse >= 0.32 {
        eprintln!("FAIL: 17² RMSE {coarse:.4} above the pinned gate 0.32");
        failed = true;
    }
    if fine >= 0.20 {
        eprintln!("FAIL: 33² RMSE {fine:.4} above the pinned gate 0.20");
        failed = true;
    }
    if fine >= coarse - 0.04 {
        eprintln!("FAIL: refinement trend broken: 33² {fine:.4} vs 17² {coarse:.4}");
        failed = true;
    }
    if failed {
        std::process::exit(1);
    }
    println!("# trend holds: RMSE decreases under refinement");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.get(1).map(String::as_str) == Some("trend") {
        run_trend();
        return;
    }
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(65);
    let t_end: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(100.0);
    let h = 1.0 / (n - 1) as f64;
    let dt = 0.45 * h;
    let steps = (t_end / dt).ceil() as usize;

    println!("# DEC lid-driven cavity, Re = {RE}");
    println!("# grid {n}x{n} (h = {h:.5}), dt = {dt:.5}, t_end = {t_end}, steps = {steps}");

    let u_form = march(n, t_end);
    let (u_profile, v_profile) = centerline_profiles(&u_form, n, h);
    let interp = |profile: &[f64], s: f64| -> f64 { interp_profile(profile, h, s) };

    write_centerline_csv(
        "cavity_centerline_u.csv",
        "y,u_computed,u_ghia,diff",
        &u_profile,
        h,
        &GHIA_U,
        &interp,
    );
    write_centerline_csv(
        "cavity_centerline_v.csv",
        "x,v_computed,v_ghia,diff",
        &v_profile,
        h,
        &GHIA_V,
        &interp,
    );

    let rmse = centerline_rmse(&u_profile, &v_profile, h);
    println!("# centerline RMSE vs Ghia: {rmse:.4}");

    // --- Vortex centers (streamfunction extrema) ------------------------
    // ψ = 0 on the walls (they are streamlines); integrate up each column:
    // Δψ across the vertical edge (i, j)→(i, j+1) is ∫u_x dy ≈ the average
    // of the four flanking x-edge velocities times h.
    let x_edge = |i: usize, j: usize| u_form[j * (n - 1) + i] / h;
    let mut psi = vec![0.0f64; n * n];
    for i in 0..n {
        for j in 0..(n - 1) {
            let il = i.saturating_sub(1).min(n - 2);
            let ir = i.min(n - 2);
            let u_mid =
                0.25 * (x_edge(il, j) + x_edge(ir, j) + x_edge(il, j + 1) + x_edge(ir, j + 1));
            psi[(j + 1) * n + i] = psi[j * n + i] + u_mid * h;
        }
    }

    println!("# vortex centers (streamfunction extrema) vs Ghia (1982):");
    println!("vortex,x,y,psi,ghia_x,ghia_y");
    // Primary: the global |ψ| extremum; corner eddies: the opposite-signed
    // extrema in the bottom corner quadrants.
    let (pi_, pj, p_psi) = extremum(&psi, n, |_, _| true, None);
    println!(
        "primary,{:.4},{:.4},{:+.4e},{:.4},{:.4}",
        pi_ as f64 * h,
        pj as f64 * h,
        p_psi,
        GHIA_VORTICES[0].1,
        GHIA_VORTICES[0].2
    );
    let corner_sign = -p_psi.signum();
    for (slot, region) in [
        (1usize, [0.0, 0.3, 0.0, 0.3]),
        (2usize, [0.7, 1.0, 0.0, 0.3]),
    ] {
        let (name, gx, gy) = GHIA_VORTICES[slot];
        let (ci, cj, c_psi) = extremum(
            &psi,
            n,
            |x, y| x >= region[0] && x <= region[1] && y >= region[2] && y <= region[3],
            Some(corner_sign),
        );
        if c_psi == 0.0 {
            // The corner eddy did not separate at this resolution/horizon;
            // the reporting run (129², t_end ≥ 150) resolves it.
            println!("{name},unresolved,unresolved,,{gx:.4},{gy:.4}");
        } else {
            println!(
                "{name},{:.4},{:.4},{:+.4e},{gx:.4},{gy:.4}",
                ci as f64 * h,
                cj as f64 * h,
                c_psi
            );
        }
    }
}

/// The interior vertex maximizing `|ψ|` within `region` (lattice
/// coordinates in [0, 1]); `sign` restricts to one rotation sense.
fn extremum(
    psi: &[f64],
    n: usize,
    region: impl Fn(f64, f64) -> bool,
    sign: Option<f64>,
) -> (usize, usize, f64) {
    let h = 1.0 / (n - 1) as f64;
    let mut best = (0usize, 0usize, 0.0f64);
    for j in 1..(n - 1) {
        for i in 1..(n - 1) {
            let (x, y) = (i as f64 * h, j as f64 * h);
            if !region(x, y) {
                continue;
            }
            let v = psi[j * n + i];
            if let Some(s) = sign
                && v.signum() != s
            {
                continue;
            }
            if v.abs() > best.2.abs() {
                best = (i, j, v);
            }
        }
    }
    best
}

#[allow(clippy::too_many_arguments)]
fn write_centerline_csv(
    path: &str,
    header: &str,
    profile: &[f64],
    h: f64,
    ghia: &[(f64, f64); 17],
    interp: &impl Fn(&[f64], f64) -> f64,
) {
    // Render every field to a string with the exact same specifiers as before, so the bytes are
    // identical; the IO effect only handles the write. `write_csv` builds a deferred action and
    // `run` executes it once, at the edge — an IO failure surfaces as a `CausalityError`.
    let header_fields: Vec<String> = header.split(',').map(|s| s.to_string()).collect();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(ghia.len() + profile.len());
    // The Ghia stations with reference values and differences.
    for &(s, reference) in ghia {
        let computed = interp(profile, s);
        rows.push(vec![
            format!("{s:.4}"),
            format!("{computed:.6}"),
            format!("{reference:.5}"),
            format!("{:+.6}", computed - reference),
        ]);
    }
    // The full computed profile (reference column empty).
    for (j, value) in profile.iter().enumerate() {
        rows.push(vec![
            format!("{:.4}", j as f64 * h),
            format!("{value:.6}"),
            String::new(),
            String::new(),
        ]);
    }
    write_csv(path, header_fields, rows)
        .run()
        .unwrap_or_else(|e| fail("centerline csv", e));
    println!("# wrote {path}");
}
