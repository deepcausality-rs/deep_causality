// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! Trajectory FS-2 — the coupling law: does an Encke/Strang split carry **non-conformal** aero forcing
//! without ever touching the conformal core? (Gap-3 Resolution-3, de-risking item ③ — the corridor's
//! "coupling Bars 2T to non-conformal external forcing is a research move, not textbook [open]".)
//!
//! The thesis: you do **not** need to express aero inside the conformal/regularised algebra. Split the
//! step — exact inverse-square core (the FS-1 generator, here exact Kepler drift) + a between-step
//! perturbation **kick applied in physical Cartesian velocity**. If the symmetric (Strang) composition is
//! 2nd-order accurate and its error vanishes with the perturbation, the [open] concern dissolves: the core
//! stays an untouched matrix exponential, the perturbation rides a standard operator split.
//!
//! Setup: a bound ellipse perturbed by a **mock drag** `a = −k·v` (non-conservative — the hard,
//! energy-changing aero analog). Strang per macro-step `H`:  v ·= e^(−kH/2)  →  exact Kepler drift H  →
//! v ·= e^(−kH/2)  (both sub-flows exact; the only error is the operator non-commutator).
//!
//! Measurements (vs an RK4 reference of the full EOM `ẍ = −μx/r³ − k·v` at a tiny step):
//!   G1  observed order ≈ 2 — `log₂(err_H / err_{H/2})` over a refinement sweep lands in [1.7, 2.3].
//!   G2  the split error **vanishes with the perturbation**: shrinking ε = |a_aero|/|a_grav| by 10×
//!       shrinks the error by ≳ 5× (as ε→0 the exact Kepler drift is the whole answer).
//!   G3  at a moderate macro-step the split tracks the reference to a small relative tolerance.

use deep_causality_physics::EARTH_GM;

const TWO_PI: f64 = std::f64::consts::TAU;
type Vec2 = [f64; 2];

fn norm2(v: Vec2) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}
fn dot2(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

// ── Exact Kepler drift (the conformal core, returns full state) ─────────────────────────────────────

fn solve_kepler(m: f64, e: f64) -> f64 {
    let m = m.rem_euclid(TWO_PI);
    let mut ea = if e < 0.8 { m } else { std::f64::consts::PI };
    for _ in 0..100 {
        let d = (ea - e * ea.sin() - m) / (1.0 - e * ea.cos());
        ea -= d;
        if d.abs() < 1e-15 {
            break;
        }
    }
    ea
}

/// Exact Kepler propagation of the full state `(r, v)` by `dt` (pure μ — the inverse-square core).
fn kepler_drift(r0: Vec2, v0: Vec2, mu: f64, dt: f64) -> (Vec2, Vec2) {
    let r = norm2(r0);
    let v2 = dot2(v0, v0);
    let energy = 0.5 * v2 - mu / r;
    let a = -mu / (2.0 * energy);
    let rv = dot2(r0, v0);
    let e_vec = [
        ((v2 - mu / r) * r0[0] - rv * v0[0]) / mu,
        ((v2 - mu / r) * r0[1] - rv * v0[1]) / mu,
    ];
    let e = norm2(e_vec);
    let n = (mu / (a * a * a)).sqrt();
    let omega = e_vec[1].atan2(e_vec[0]);
    let cos_nu = (dot2(e_vec, r0) / (e * r)).clamp(-1.0, 1.0);
    let mut nu0 = cos_nu.acos();
    if rv < 0.0 {
        nu0 = TWO_PI - nu0;
    }
    let ea0 =
        2.0 * ((1.0 - e).sqrt() * (nu0 / 2.0).sin()).atan2((1.0 + e).sqrt() * (nu0 / 2.0).cos());
    let m0 = ea0 - e * ea0.sin();
    let ea = solve_kepler(m0 + n * dt, e);
    let b = a * (1.0 - e * e).sqrt();
    let x_pf = a * (ea.cos() - e);
    let y_pf = b * ea.sin();
    let edot = n / (1.0 - e * ea.cos());
    let xdot_pf = -a * ea.sin() * edot;
    let ydot_pf = b * ea.cos() * edot;
    let (c, s) = (omega.cos(), omega.sin());
    let pos = [c * x_pf - s * y_pf, s * x_pf + c * y_pf];
    let vel = [c * xdot_pf - s * ydot_pf, s * xdot_pf + c * ydot_pf];
    (pos, vel)
}

// ── Reference: RK4 of the full perturbed equations of motion ────────────────────────────────────────

fn accel(r: Vec2, v: Vec2, mu: f64, k: f64) -> Vec2 {
    let rn = norm2(r);
    let g = -mu / (rn * rn * rn);
    // gravity + mock drag (−k·v).
    [g * r[0] - k * v[0], g * r[1] - k * v[1]]
}

/// RK4 of `[r, v]` under gravity + drag over `total` seconds in `nsteps` steps — the "truth" reference.
fn rk4_reference(mut r: Vec2, mut v: Vec2, mu: f64, k: f64, total: f64, nsteps: usize) -> Vec2 {
    let h = total / nsteps as f64;
    for _ in 0..nsteps {
        let a1 = accel(r, v, mu, k);
        let (k1r, k1v) = (v, a1);
        let r2 = [r[0] + 0.5 * h * k1r[0], r[1] + 0.5 * h * k1r[1]];
        let v2 = [v[0] + 0.5 * h * k1v[0], v[1] + 0.5 * h * k1v[1]];
        let a2 = accel(r2, v2, mu, k);
        let (k2r, k2v) = (v2, a2);
        let r3 = [r[0] + 0.5 * h * k2r[0], r[1] + 0.5 * h * k2r[1]];
        let v3 = [v[0] + 0.5 * h * k2v[0], v[1] + 0.5 * h * k2v[1]];
        let a3 = accel(r3, v3, mu, k);
        let (k3r, k3v) = (v3, a3);
        let r4 = [r[0] + h * k3r[0], r[1] + h * k3r[1]];
        let v4 = [v[0] + h * k3v[0], v[1] + h * k3v[1]];
        let a4 = accel(r4, v4, mu, k);
        let (k4r, k4v) = (v4, a4);
        for d in 0..2 {
            r[d] += h / 6.0 * (k1r[d] + 2.0 * k2r[d] + 2.0 * k3r[d] + k4r[d]);
            v[d] += h / 6.0 * (k1v[d] + 2.0 * k2v[d] + 2.0 * k3v[d] + k4v[d]);
        }
    }
    r
}

// ── The candidate: Strang split (exact drag kick — exact Kepler drift — exact drag kick) ────────────

fn strang_split(mut r: Vec2, mut v: Vec2, mu: f64, k: f64, total: f64, nsteps: usize) -> Vec2 {
    let h = total / nsteps as f64;
    let half = (-k * h / 2.0).exp(); // exact flow of the drag-only sub-system v' = −k v over H/2
    for _ in 0..nsteps {
        v = [v[0] * half, v[1] * half]; // half kick — applied in PHYSICAL Cartesian velocity
        let (rn, vn) = kepler_drift(r, v, mu, h); // exact inverse-square core — never modified
        r = rn;
        v = vn;
        v = [v[0] * half, v[1] * half]; // half kick
    }
    r
}

fn gate(label: &str, pass: bool) -> bool {
    println!("  [{}] {label}", if pass { "PASS" } else { "FAIL" });
    pass
}

fn main() {
    println!(
        "=== FS-2: non-conformal aero as a between-step kick on an untouched inverse-square core ===\n"
    );
    let mu = EARTH_GM;
    let r0: Vec2 = [7.0e6, 0.0];
    let v0: Vec2 = [1.0e3, 7.5e3];
    let r = norm2(r0);
    let a_grav = mu / (r * r);
    let period = {
        let energy = 0.5 * dot2(v0, v0) - mu / r;
        let a = -mu / (2.0 * energy);
        TWO_PI * (a * a * a / mu).sqrt()
    };
    let total = period; // integrate one orbit

    // Drag coefficient set for ε ≈ 1e-3 (a deep-perturbative regime, the matrix-exp's home).
    let k = 1.0e-6;
    let eps = k * norm2(v0) / a_grav;
    println!(
        "Orbit period {:.1} s; mock drag k = {:.1e}/s ⇒ ε = |a_aero|/|a_grav| = {:.3e}\n",
        period, k, eps
    );

    // High-resolution RK4 reference (truth).
    let r_ref = rk4_reference(r0, v0, mu, k, total, 400_000);

    // G1 — convergence order of the Strang split over a refinement sweep.
    println!("Strang-split convergence (vs RK4 reference):");
    let mut errs = Vec::new();
    let ns = [50usize, 100, 200, 400, 800];
    for &n in &ns {
        let rs = strang_split(r0, v0, mu, k, total, n);
        let err = ((rs[0] - r_ref[0]).powi(2) + (rs[1] - r_ref[1]).powi(2)).sqrt();
        errs.push(err);
        println!(
            "  N = {n:>4}  H = {:>7.1} s   |error| = {err:.4e} m",
            total / n as f64
        );
    }
    // Observed order from the two finest levels (where the asymptotic regime holds).
    let order = (errs[errs.len() - 2] / errs[errs.len() - 1]).log2();
    println!("  observed order (finest pair) = {order:.3}");

    // G2 — the split error vanishes with the perturbation: shrink ε by 10×, error should drop ≳ 5×.
    let n_fixed = 200usize;
    let err_eps = {
        let rs = strang_split(r0, v0, mu, k, total, n_fixed);
        let rr = rk4_reference(r0, v0, mu, k, total, 400_000);
        ((rs[0] - rr[0]).powi(2) + (rs[1] - rr[1]).powi(2)).sqrt()
    };
    let k10 = k / 10.0;
    let err_eps10 = {
        let rs = strang_split(r0, v0, mu, k10, total, n_fixed);
        let rr = rk4_reference(r0, v0, mu, k10, total, 400_000);
        ((rs[0] - rr[0]).powi(2) + (rs[1] - rr[1]).powi(2)).sqrt()
    };
    let drop = err_eps / err_eps10;
    println!(
        "\nPerturbation scaling at N = {n_fixed}:  ε → ε/10 drops the split error {err_eps:.3e} → {err_eps10:.3e}  ({drop:.1}×)"
    );

    // G3 — relative accuracy at a moderate macro-step.
    let semi_major = {
        let energy = 0.5 * dot2(v0, v0) - mu / r;
        -mu / (2.0 * energy)
    };
    let rel_moderate = errs[2] / semi_major; // N = 200
    println!(
        "Relative accuracy at N = 200: |error|/a = {rel_moderate:.3e}  (a = {:.1} km)\n",
        semi_major / 1000.0
    );

    println!("--- FS-2 gates ---");
    let g1 = gate(
        "Strang split is 2nd-order (observed order in [1.7, 2.3])",
        (1.7..=2.3).contains(&order),
    );
    let g2 = gate(
        "split error vanishes with the perturbation (ε/10 ⇒ ≳5× drop)",
        drop > 5.0,
    );
    let g3 = gate(
        "moderate macro-step tracks the reference (rel error < 1e-4·a)",
        rel_moderate < 1e-4,
    );

    if g1 && g2 && g3 {
        println!(
            "\n=== FINDING: non-conformal aero rides a between-step Cartesian kick at 2nd order, with the\n\
             inverse-square core left an untouched exact matrix exponential. The corridor's 'coupling Bars 2T\n\
             to non-conformal forcing [open]' concern DISSOLVES: you split in physical space (Encke/Strang),\n\
             you do not express aero in the conformal algebra. B1's perturbation factoring HOLDS. ==="
        );
    } else {
        std::process::exit(1);
    }
}
