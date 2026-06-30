// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! Trajectory FS-1 — the generator: is the inverse-square core an **exact constant-generator matrix
//! exponential**? (Gap-3 Resolution-3, de-risking Resolution-1's B1 "exact conformal core" — items ①/②.)
//!
//! Resolution 1 asserts the bound inverse-square (Kepler) trajectory equals `ψ(s) = e^{G·s} ψ(0)` for a
//! **constant** generator `G` under a time reparametrisation, but never gives `G`. This study supplies the
//! concrete, textbook realisation — the **eccentric-anomaly linearisation** (the 1-D essence of
//! Kustaanheimo–Stiefel / Levi-Civita regularisation; Stiefel & Scheifele, *Linear and Regular Celestial
//! Mechanics*, 1971): in eccentric anomaly `s = E`, the recentred perifocal coordinate `Q = (a·cos E,
//! b·sin E)` solves the unit-frequency harmonic oscillator `Q'' = −Q`, so the phase state `ψ = (Q, Q')`
//! advances by the **constant** 4×4 symplectic generator `Ω = [[0, I₂], [−I₂, 0]]`. Physical time is the
//! closed form `Δt = (M − M₀)/n`, `M = E − e·sin E` (Kepler's equation).
//!
//! Measurements (gates encode the finding; exit non-zero on regression):
//!   G1  the *generic* matrix exponential `e^{Ω·2π}` returns the identity to round-off — the flow is
//!       exactly periodic (the orbit closes), i.e. the matrix exponential is an exact Kepler solver.
//!   G2  the generic scaling-and-squaring `e^{Ω·s}` matches the closed-form cos/sin block to ~1e-13 — the
//!       "matrix exponential" is literal, not hand-waved.
//!   G3  physical positions reconstructed from `e^{Ω·s} ψ₀` match an **independent** orbital-element Kepler
//!       propagation (Newton-solved Kepler equation) at the matched physical times, to ≲ 1e-9·a over a full
//!       orbit.
//!   G4  the generator is `s`-independent: the semigroup law `e^{Ω·s₁}·e^{Ω·s₂} = e^{Ω·(s₁+s₂)}` holds to
//!       round-off — one constant `G` generates the whole flow.

use deep_causality_physics::EARTH_GM;

const TWO_PI: f64 = std::f64::consts::TAU;

type Vec2 = [f64; 2];
type Mat4 = [[f64; 4]; 4];

fn norm2(v: Vec2) -> f64 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}
fn dot2(a: Vec2, b: Vec2) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}

// ── Independent reference: classical orbital-element Kepler propagation ──────────────────────────────

/// Solve Kepler's equation `M = E − e·sin E` for the eccentric anomaly `E` (Newton, round-off tight).
fn solve_kepler(m: f64, e: f64) -> f64 {
    let m = m.rem_euclid(TWO_PI);
    let mut ea = if e < 0.8 { m } else { std::f64::consts::PI };
    for _ in 0..100 {
        let f = ea - e * ea.sin() - m;
        let fp = 1.0 - e * ea.cos();
        let d = f / fp;
        ea -= d;
        if d.abs() < 1e-15 {
            break;
        }
    }
    ea
}

/// Orbital elements derived from a planar physical state `(r, v)` at `μ`.
struct Elements {
    a: f64,
    e: f64,
    n: f64,
    omega_peri: f64,
    ea0: f64,
}

fn elements_from_state(r0: Vec2, v0: Vec2, mu: f64) -> Elements {
    let r = norm2(r0);
    let v2 = dot2(v0, v0);
    let energy = 0.5 * v2 - mu / r;
    let a = -mu / (2.0 * energy);
    let rv = dot2(r0, v0);
    // Eccentricity vector e_vec = ((v² − μ/r)·r − (r·v)·v)/μ.
    let e_vec = [
        ((v2 - mu / r) * r0[0] - rv * v0[0]) / mu,
        ((v2 - mu / r) * r0[1] - rv * v0[1]) / mu,
    ];
    let e = norm2(e_vec);
    let n = (mu / (a * a * a)).sqrt();
    let omega_peri = e_vec[1].atan2(e_vec[0]);
    // True anomaly of the initial point.
    let cos_nu = (dot2(e_vec, r0) / (e * r)).clamp(-1.0, 1.0);
    let mut nu0 = cos_nu.acos();
    if rv < 0.0 {
        nu0 = TWO_PI - nu0;
    }
    // Eccentric anomaly from true anomaly.
    let ea0 =
        2.0 * ((1.0 - e).sqrt() * (nu0 / 2.0).sin()).atan2((1.0 + e).sqrt() * (nu0 / 2.0).cos());
    Elements {
        a,
        e,
        n,
        omega_peri,
        ea0,
    }
}

/// Independent Kepler propagation: advance `(r0, v0)` by `dt` via the orbital elements + Newton solve.
fn propagate_kepler(r0: Vec2, v0: Vec2, mu: f64, dt: f64) -> Vec2 {
    let el = elements_from_state(r0, v0, mu);
    let b = el.a * (1.0 - el.e * el.e).sqrt();
    let m0 = el.ea0 - el.e * el.ea0.sin();
    let m = m0 + el.n * dt;
    let ea = solve_kepler(m, el.e);
    // Perifocal position, then rotate by the argument of periapsis.
    let x_pf = el.a * (ea.cos() - el.e);
    let y_pf = b * ea.sin();
    let (c, s) = (el.omega_peri.cos(), el.omega_peri.sin());
    [c * x_pf - s * y_pf, s * x_pf + c * y_pf]
}

// ── The candidate: a literal 4×4 matrix exponential of the constant generator Ω ─────────────────────

fn mat_id() -> Mat4 {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m
}
fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut c = [[0.0; 4]; 4];
    for (i, ci) in c.iter_mut().enumerate() {
        for (k, &aik) in a[i].iter().enumerate() {
            for (j, cij) in ci.iter_mut().enumerate() {
                *cij += aik * b[k][j];
            }
        }
    }
    c
}
fn mat_scale(a: &Mat4, s: f64) -> Mat4 {
    let mut m = *a;
    for row in m.iter_mut() {
        for x in row.iter_mut() {
            *x *= s;
        }
    }
    m
}
fn mat_add(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut m = *a;
    for (i, row) in m.iter_mut().enumerate() {
        for (j, x) in row.iter_mut().enumerate() {
            *x += b[i][j];
        }
    }
    m
}
fn mat_max_abs_diff(a: &Mat4, b: &Mat4) -> f64 {
    let mut d = 0.0f64;
    for i in 0..4 {
        for j in 0..4 {
            d = d.max((a[i][j] - b[i][j]).abs());
        }
    }
    d
}

/// Generic matrix exponential via scaling-and-squaring with a truncated Taylor series — deliberately *not*
/// the closed form, so G2 is a genuine cross-check that the flow is a real matrix exponential.
fn mat_exp(a: &Mat4) -> Mat4 {
    // Scale so the spectral size is small, exponentiate the Taylor series, then square back.
    let mut norm = 0.0f64;
    for row in a {
        for &x in row {
            norm = norm.max(x.abs());
        }
    }
    let k = (norm.log2().ceil().max(0.0)) as i32 + 4;
    let scale = 0.5f64.powi(k);
    let a_s = mat_scale(a, scale);
    // Taylor: I + A + A²/2! + …
    let mut term = mat_id();
    let mut acc = mat_id();
    for n in 1..=18 {
        term = mat_scale(&mat_mul(&term, &a_s), 1.0 / n as f64);
        acc = mat_add(&acc, &term);
    }
    // Square k times.
    for _ in 0..k {
        acc = mat_mul(&acc, &acc);
    }
    acc
}

/// The constant generator Ω = [[0, I₂], [−I₂, 0]] (the harmonic-oscillator / symplectic-rotation matrix).
fn generator_omega() -> Mat4 {
    let mut g = [[0.0; 4]; 4];
    g[0][2] = 1.0;
    g[1][3] = 1.0;
    g[2][0] = -1.0;
    g[3][1] = -1.0;
    g
}

/// Closed form e^{Ω·s} = [[cos s·I₂, sin s·I₂], [−sin s·I₂, cos s·I₂]].
fn omega_flow_closed(s: f64) -> Mat4 {
    let (cs, sn) = (s.cos(), s.sin());
    let mut m = [[0.0; 4]; 4];
    m[0][0] = cs;
    m[1][1] = cs;
    m[2][2] = cs;
    m[3][3] = cs;
    m[0][2] = sn;
    m[1][3] = sn;
    m[2][0] = -sn;
    m[3][1] = -sn;
    m
}

fn apply(m: &Mat4, v: [f64; 4]) -> [f64; 4] {
    let mut o = [0.0; 4];
    for (i, oi) in o.iter_mut().enumerate() {
        for (j, &vj) in v.iter().enumerate() {
            *oi += m[i][j] * vj;
        }
    }
    o
}

fn gate(label: &str, pass: bool) -> bool {
    println!("  [{}] {label}", if pass { "PASS" } else { "FAIL" });
    pass
}

fn main() {
    println!(
        "=== FS-1: the inverse-square core as an exact constant-generator matrix exponential ===\n"
    );
    let mu = EARTH_GM;

    // A representative bound elliptical orbit (LEO-ish, moderate eccentricity).
    let r0: Vec2 = [7.0e6, 0.0];
    let v0: Vec2 = [1.0e3, 7.5e3];
    let el = elements_from_state(r0, v0, mu);
    let b = el.a * (1.0 - el.e * el.e).sqrt();
    let period = TWO_PI / el.n;
    println!(
        "Orbit: a = {:.1} km, e = {:.4}, period = {:.1} s  (μ = EARTH_GM = {:.6e})",
        el.a / 1000.0,
        el.e,
        period,
        mu
    );

    let omega = generator_omega();

    // G1 — exact periodicity: e^{Ω·2π} = I to round-off (the orbit closes; the matrix exp is exact).
    let full = mat_exp(&mat_scale(&omega, TWO_PI));
    let g1_err = mat_max_abs_diff(&full, &mat_id());
    println!("\nG1 one-period closure ‖e^(Ω·2π) − I‖_max = {g1_err:.3e}");

    // G2 — the generic matrix exponential matches the closed form.
    let s_probe = 1.234;
    let g2_err = mat_max_abs_diff(
        &mat_exp(&mat_scale(&omega, s_probe)),
        &omega_flow_closed(s_probe),
    );
    println!("G2 matrix-exp vs closed-form  ‖·‖_max = {g2_err:.3e}");

    // G4 — semigroup (single constant generator): e^{Ω·s1}·e^{Ω·s2} = e^{Ω·(s1+s2)}.
    let (s1, s2) = (0.7, 1.9);
    let compose = mat_mul(&omega_flow_closed(s1), &omega_flow_closed(s2));
    let g4_err = mat_max_abs_diff(&compose, &omega_flow_closed(s1 + s2));
    println!("G4 semigroup law  ‖e^(Ωs1)e^(Ωs2) − e^(Ω(s1+s2))‖_max = {g4_err:.3e}");

    // G3 — physical trajectory from the matrix-exp flow vs the independent element propagation.
    // Initial phase state ψ₀ = (Q₀, Q₀') with Q₀ = (a·cos E₀, b·sin E₀), Q₀' = dQ/dE.
    let psi0 = [
        el.a * el.ea0.cos(),
        b * el.ea0.sin(),
        -el.a * el.ea0.sin(),
        b * el.ea0.cos(),
    ];
    let m0 = el.ea0 - el.e * el.ea0.sin();
    let (cw, sw) = (el.omega_peri.cos(), el.omega_peri.sin());
    let steps = 360usize;
    let mut max_pos_err = 0.0f64;
    for kk in 0..=steps {
        let s = TWO_PI * kk as f64 / steps as f64;
        let psi = apply(&omega_flow_closed(s), psi0); // ψ(s) = e^{Ω·s} ψ₀
        let (q1, q2) = (psi[0], psi[1]); // Q(s) = (a cos E, b sin E), E = E₀ + s
        // Physical perifocal position is Q recentred onto the focus, then rotated by ω_peri.
        let (x_pf, y_pf) = (q1 - el.a * el.e, q2);
        let pos_exp = [cw * x_pf - sw * y_pf, sw * x_pf + cw * y_pf];
        // Matched physical time from Kepler's equation, then the independent propagation.
        let ea = el.ea0 + s;
        let dt = (ea - el.e * ea.sin() - m0) / el.n;
        let pos_ref = propagate_kepler(r0, v0, mu, dt);
        let err = ((pos_exp[0] - pos_ref[0]).powi(2) + (pos_exp[1] - pos_ref[1]).powi(2)).sqrt();
        max_pos_err = max_pos_err.max(err);
    }
    let g3_rel = max_pos_err / el.a;
    println!(
        "G3 max position error over a full orbit = {max_pos_err:.3e} m  ({g3_rel:.3e}·a, matrix-exp vs element propagation)"
    );

    println!("\n--- FS-1 gates ---");
    let g1 = gate(
        "e^(Ω·2π) = I to round-off (exact periodicity)",
        g1_err < 1e-11,
    );
    let g2 = gate("generic matrix-exp matches the closed form", g2_err < 1e-12);
    let g3 = gate(
        "matrix-exp trajectory matches independent Kepler to ~1e-9·a",
        g3_rel < 1e-9,
    );
    let g4 = gate(
        "constant generator (semigroup law to round-off)",
        g4_err < 1e-12,
    );

    if g1 && g2 && g3 && g4 {
        println!(
            "\n=== FINDING: the bound inverse-square core IS an exact constant-generator matrix exponential.\n\
             Resolution-1's B1 'exact conformal core' HOLDS, with the concrete generator Ω (eccentric-anomaly\n\
             / KS realisation). The production 3-D, singularity-free, perturbation-ready form is KS\n\
             regularisation (Stiefel-Scheifele); the heavier Bars (4,2) packaging is optional, not required. ==="
        );
    } else {
        std::process::exit(1);
    }
}
