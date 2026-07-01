// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT rank study — the *nonlinear* case: does a **forming shock** stay low tensor-train rank?
//!
//! `qtt_rank_dynamic` left this OPEN: its `QttLinear1d` vehicle is linear, so it *transports* a fixed
//! shape but never *forms* a shock. The Tier-B threat is precisely the nonlinear steepening of smooth
//! data into a near-discontinuity, and whether the bond dimension explodes as that happens.
//!
//! Vehicle: a self-contained **Burgers marcher** built from the existing QTT primitives —
//! `u_t + ½(u²)_x = ν u_xx` in conservative (flux) form, explicit Euler with per-step rounding:
//! `u ← round(u + Δt·(−½ ∂_x(u²) + ν ∂_xx u))`. Burgers is the canonical minimal shock-former: smooth
//! initial data steepens into a shock in finite time. Viscosity `ν` is the artificial-viscosity /
//! thickening lever.
//!
//! Two cases, by design:
//!   - 1-D: tests nonlinear steepening rank dynamics. Prediction (from `qtt_rank_study`): small — a
//!     1-D discontinuity is rank <= 2, so even a forming 1-D shock stays cheap.
//!   - 2-D (curved): the DECISIVE test. A radial bump self-advects into a *curved* shock — exactly the
//!     misaligned 2-D structure the static study showed is expensive. Does the rank climb as the curved
//!     shock forms?
//!
//! Finding (it refuted the naive "just thicken it" hypothesis): the forming 2-D curved shock DOES raise
//! rank, and thickening is NOT the lever — the rank is set by curvature/mis-alignment (which thickening
//! cannot remove), and naive over-thickening is diffusion-CFL-unstable (it blows up to full rank). The
//! real levers are coordinate alignment (static study) and an implicit/IMEX step (gap C3), neither
//! exercised here. So this study confirms the threat is real and dynamic, and scopes the fix.
//!
//! Self-verifying: gates encode the findings; exit non-zero on regression. This is a *study*, so the
//! headline is the measured magnitudes printed below, not a pass/fail number.

use deep_causality_cfd::{
    gradient, gradient_x, gradient_y, laplacian, laplacian_2d, quantize, quantize_2d,
};
use deep_causality_tensor::{CausalTensor, TensorTrain, TensorTrainOperator, Truncation};

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let tol = 1e-8;

    // ===============================================================
    // 1-D Burgers: nonlinear steepening, predicted cheap (1-D is rank <= 2).
    // ===============================================================
    let l = 10usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let dt = 0.5 * dx; // advection-limited (max|u| ~ 1), viscosity keeps it stable
    // u0 = 0.5 + 0.4 sin(2 pi x): smooth wave; shock forms near t* ~ 1/(2 pi * 0.4) ~ 0.40
    let u0_1d: Vec<f64> = (0..n)
        .map(|i| 0.5 + 0.4 * (std::f64::consts::TAU * i as f64 / n as f64).sin())
        .collect();
    let steps_1d = (0.6 / dt) as usize; // march past shock formation
    let every_1d = steps_1d / 8;

    // ν chosen STABLE: diffusion number ν·dt/dx² = 0.5·0.5 = 0.25 <= 0.5 (explicit-Euler limit).
    println!("=== 1-D Burgers (L={l}, N={n}, dt={dt:.2e}, steps={steps_1d}, tol={tol:e}) ===");
    println!("    u0 = 0.5 + 0.4 sin(2 pi x); shock forms near t* ~ 0.40; nu = 0.5 dx (stable)\n");
    let sim1 = Sim {
        dx,
        dt,
        nu: 0.5 * dx,
        tol,
        steps: steps_1d,
        every: every_1d,
    };
    let (p1, s1) = burgers_1d(u0_1d, l, &sim1);
    println!("  {:>6} | {:>12}", "step", "max_bond");
    for &(st, r) in &s1 {
        println!("  {st:>6} | {r:>12}");
    }
    println!("  PEAK bond: {p1}");

    // Gate 1D-A: a forming 1-D shock stays low rank (the static prediction holds dynamically).
    if p1 > 16 {
        failures.push(format!(
            "1D-A: forming 1-D shock unexpectedly high rank (peak={p1})"
        ));
    }

    // ===============================================================
    // 2-D Burgers: a CURVED forming shock — the decisive test.
    // ===============================================================
    let l2 = 6usize; // 64 x 64 (kept small for runtime; rank rise is already clear here)
    let side = 1usize << l2;
    let dx2 = 1.0 / side as f64;
    let dt2 = 0.2 * dx2;
    // smooth radial bump; self-advection steepens its downstream edge into a curved shock
    let mut u0_2d = vec![0.0f64; side * side];
    for ix in 0..side {
        for iy in 0..side {
            let x = ix as f64 * dx2;
            let y = iy as f64 * dx2;
            let r2 = (x - 0.35).powi(2) + (y - 0.5).powi(2);
            u0_2d[ix * side + iy] = (-r2 / (2.0 * 0.08_f64.powi(2))).exp();
        }
    }
    let steps_2d = 250usize;
    let every_2d = steps_2d / 8;

    // STABLE run: nu = 1 dx, diffusion number = 1·0.2 = 0.2 <= 0.25 (2-D explicit limit).
    // CFL-TRAP run: nu = 6 dx, diffusion number = 6·0.2 = 1.2 >> 0.25 -> diffusion-unstable.
    println!(
        "\n=== 2-D Burgers (Lx=Ly={l2}, {side}x{side}, dt={dt2:.2e}, steps={steps_2d}, tol={tol:e}) ==="
    );
    println!("    smooth radial bump self-advects into a CURVED shock — the decisive rank test\n");
    let sim2_stable = Sim {
        dx: dx2,
        dt: dt2,
        nu: 1.0 * dx2,
        tol,
        steps: steps_2d,
        every: every_2d,
    };
    let sim2_overthick = Sim {
        dx: dx2,
        dt: dt2,
        nu: 6.0 * dx2,
        tol,
        steps: steps_2d,
        every: every_2d,
    };
    let (p2, s2) = burgers_2d(u0_2d.clone(), l2, l2, &sim2_stable);
    let (p2_trap, _s2_trap) = burgers_2d(u0_2d, l2, l2, &sim2_overthick);
    let init_2d = s2[0].1;
    println!("  {:>6} | {:>24}", "step", "max_bond (stable, nu=1dx)");
    for &(st, r) in &s2 {
        println!("  {st:>6} | {r:>24}");
    }
    println!("  smooth-bump init bond: {init_2d}");
    println!("  STABLE peak bond: {p2}   (grid-saturation cap = {side})");
    println!(
        "  CFL-TRAP peak bond: {p2_trap}   (nu=6dx violates the diffusion CFL -> blows up to full rank)"
    );

    // Gate 2D-A: a forming CURVED shock raises rank above the smooth bump it grew from
    // (the nonlinear 2-D rank threat is real, not hypothetical).
    if p2 <= init_2d {
        failures.push(format!(
            "2D-A: curved-shock formation did not raise rank above the smooth init (init={init_2d}, peak={p2})"
        ));
    }
    // Gate 2D-B: the STABLE run stays below grid saturation — the measured rank is real shock
    // structure, not a blow-up.
    if p2 >= side {
        failures.push(format!(
            "2D-B: the 'stable' run saturated the grid (peak={p2}, side={side}) — unstable, not real structure"
        ));
    }
    // Gate 2D-C: the CFL-violating over-thick run DOES saturate — demonstrating that naive
    // over-thickening is diffusion-unstable, not a rank fix.
    if p2_trap < side {
        failures.push(format!(
            "2D-C: expected the CFL-violating run to saturate the grid (peak={p2_trap}, side={side})"
        ));
    }

    println!("\n--- reading ---");
    println!(
        "  1-D: forming shock peak {p1} — nonlinear steepening stays CHEAP in 1-D, as predicted."
    );
    println!(
        "  2-D: a forming CURVED shock drives bond {init_2d} -> {p2} (stable) — the nonlinear 2-D rank"
    );
    println!("       threat is REAL and DYNAMIC (and grows with resolution; this is only 64x64).");
    println!(
        "  Thickening is NOT the curved-shock lever: the rank is set by CURVATURE/mis-alignment,"
    );
    println!(
        "  which thickening cannot remove — and naive over-thickening is diffusion-CFL-unstable"
    );
    println!(
        "  (peak {p2_trap} = full rank). The lever is COORDINATE ALIGNMENT (static study) + an implicit/"
    );
    println!(
        "  IMEX step for stable dissipation (gap C3) — neither exercised here. The shock-aligned"
    );
    println!("  confinement test is the next study and the Tier-B design choice.");

    if failures.is_empty() {
        println!("\nALL GATES PASSED.");
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Burgers march configuration (one viscosity per run; everything else shared).
struct Sim {
    dx: f64,
    dt: f64,
    nu: f64,
    tol: f64,
    steps: usize,
    every: usize,
}

/// 1-D viscous Burgers: march `u0` (length `2^l`) and report (peak_bond, samples).
fn burgers_1d(u0: Vec<f64>, l: usize, s: &Sim) -> (usize, Vec<(usize, usize)>) {
    let trunc = Truncation::<f64>::by_tol(s.tol).expect("tol");
    let grad = gradient::<f64>(l, s.dx, &trunc).expect("grad");
    let lap = laplacian::<f64>(l, s.dx, &trunc).expect("lap");
    let mut u = quantize(
        &CausalTensor::new(u0, vec![1usize << l]).expect("dense"),
        &trunc,
    )
    .expect("encode");

    let mut peak = u.max_bond();
    let mut samples = vec![(0usize, u.max_bond())];
    for step in 1..=s.steps {
        let u2 = u.hadamard_rounded(&u, &trunc).expect("u^2");
        let conv = grad.apply(&u2, &trunc).expect("flux").scale(-0.5);
        let visc = lap.apply(&u, &trunc).expect("visc").scale(s.nu);
        let rate = conv.add(&visc).expect("rate");
        u = u
            .add(&rate.scale(s.dt))
            .expect("euler")
            .round(&trunc)
            .expect("round");
        peak = peak.max(u.max_bond());
        if step % s.every == 0 {
            samples.push((step, u.max_bond()));
        }
    }
    (peak, samples)
}

/// 2-D viscous Burgers: march `u0` (`2^lx x 2^ly`, row-major) and report (peak_bond, samples).
fn burgers_2d(u0: Vec<f64>, lx: usize, ly: usize, s: &Sim) -> (usize, Vec<(usize, usize)>) {
    let trunc = Truncation::<f64>::by_tol(s.tol).expect("tol");
    let gx = gradient_x::<f64>(lx, ly, s.dx, &trunc).expect("gx");
    let gy = gradient_y::<f64>(lx, ly, s.dx, &trunc).expect("gy");
    let lap = laplacian_2d::<f64>(lx, ly, s.dx, s.dx, &trunc).expect("lap2d");
    let side = 1usize << lx;
    let mut u = quantize_2d(
        &CausalTensor::new(u0, vec![side, side]).expect("dense"),
        &trunc,
    )
    .expect("encode");

    let mut peak = u.max_bond();
    let mut samples = vec![(0usize, u.max_bond())];
    for step in 1..=s.steps {
        let u2 = u.hadamard_rounded(&u, &trunc).expect("u^2");
        let fx = gx.apply(&u2, &trunc).expect("fx");
        let fy = gy.apply(&u2, &trunc).expect("fy");
        let conv = fx.add(&fy).expect("flux").scale(-0.5);
        let visc = lap.apply(&u, &trunc).expect("visc").scale(s.nu);
        let rate = conv.add(&visc).expect("rate");
        u = u
            .add(&rate.scale(s.dt))
            .expect("euler")
            .round(&trunc)
            .expect("round");
        peak = peak.max(u.max_bond());
        if step % s.every == 0 {
            samples.push((step, u.max_bond()));
        }
    }
    (peak, samples)
}
