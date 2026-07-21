// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT rank study — the *dynamic* follow-up: does a **marcher** keep the field low-rank over many
//! steps, or does the bond dimension inflate under repeated apply + round?
//!
//! The static study (`qtt_rank_study`) measured representability of frozen profiles. It did **not**
//! test the live question: a real rollout repeatedly applies a stencil operator and re-rounds, and a
//! steep feature translated to ever-changing sub-grid positions can creep in rank step over step.
//! This is the threat that decides whether the `O(chi^2 L)` cost survives *in time*.
//!
//! Vehicle: the existing `QttLinear1d` marcher — linear advection-diffusion
//! `u <- round(u + dt*(-c d_x u + nu d_xx u))` on a periodic 2^L grid. (Caveat: this is LINEAR
//! transport — it does not *form* shocks; it transports an imposed steep feature. It is the minimal
//! dynamic probe the current crate allows; a nonlinear shock-forming marcher does not exist yet.)
//!
//! Questions: does a smooth bump advected stay bounded (expect yes); does a steep top-hat at LOW
//! diffusion grow in rank (the dynamic threat); does HIGH diffusion (thickening / artificial
//! viscosity) control it (the lever).
//!
//! Self-verifying: gates encode the findings and exit non-zero on regression.

use deep_causality_cfd::{Marcher, QttLinear1d, quantize};
use deep_causality_tensor::{CausalTensor, Truncation};

fn main() {
    let mut failures: Vec<String> = Vec::new();

    let l = 12usize;
    let n = 1usize << l; // 4096
    let dx = 1.0 / n as f64;
    let c = 1.0;
    let grid = Grid {
        l,
        dx,
        dt: 0.15 * dx / c, // advection-limited step
        c,
        tol: 1e-8,
        steps: 3000,
        every: 300,
    };

    // smooth periodic bump (von Mises): low rank, the control
    let smooth: Vec<f64> = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            (8.0 * ((std::f64::consts::TAU * x).cos() - 1.0)).exp()
        })
        .collect();

    // steep periodic top-hat: two sharp tanh fronts (the shock-like feature)
    let tophat = |delta_cells: f64| -> Vec<f64> {
        let d = delta_cells * dx;
        let (a, b) = (0.35, 0.60);
        (0..n)
            .map(|i| {
                let x = i as f64 / n as f64;
                0.5 * (((x - a) / d).tanh() - ((x - b) / d).tanh())
            })
            .collect::<Vec<f64>>()
    };
    let steep = tophat(0.7); // near-grid-scale fronts -> dispersive trailing oscillations

    // diffusion stability ceiling for explicit Euler: nu <= dx^2/(2 dt)
    let nu_low = 4.0e-5;
    let nu_high = 6.0e-4;

    println!(
        "=== QTT dynamic rank study  (L={l}, N={n}, c={c}, dt={:.2e}, steps={}, tol={:e}) ===\n",
        grid.dt, grid.steps, grid.tol
    );

    // A and B share nu_low (only the profile differs); B and C share the steep profile (only nu differs).
    let (a0, a_peak, a_fin, a_s) = march_ranks(&smooth, &grid, nu_low);
    let (b0, b_peak, b_fin, b_s) = march_ranks(&steep, &grid, nu_low);
    let (c0, c_peak, c_fin, c_s) = march_ranks(&steep, &grid, nu_high);

    println!("max_bond over time:");
    println!(
        "  {:>5} | {:>22} | {:>26} | {:>27}",
        "step", "A smooth (nu_low)", "B steep (nu_low=4e-5)", "C steep (nu_high=6e-4)"
    );
    for (i, &(st, ra)) in a_s.iter().enumerate() {
        let rb = b_s.get(i).map(|x| x.1).unwrap_or(0);
        let rc = c_s.get(i).map(|x| x.1).unwrap_or(0);
        println!("  {st:>5} | {ra:>22} | {rb:>26} | {rc:>27}");
    }
    println!("\n  init / PEAK / final:");
    println!("    A smooth          : {a0} / {a_peak} / {a_fin}");
    println!("    B steep, low  nu  : {b0} / {b_peak} / {b_fin}");
    println!("    C steep, high nu  : {c0} / {c_peak} / {c_fin}");

    // What this CAN show (robust): under fixed-tolerance rounding, a linear marcher does not let
    // rank run away — a transported feature settles near its static representability rank, and more
    // diffusion (thickening) does not increase it.  What it CANNOT show: nonlinear shock *steepening*
    // rank growth — QttLinear1d transports a fixed shape, it never forms a shock.  That threat needs
    // a nonlinear (Burgers / compressible) marcher, which does not exist yet -> still OPEN.

    // Gate G1: rank does not run away for any case (no unbounded growth under apply+round).
    let cap = 32;
    if a_peak > cap || b_peak > cap || c_peak > cap {
        failures.push(format!(
            "G1: rank ran away (smooth={a_peak}, steep_low={b_peak}, steep_high={c_peak}, cap={cap})"
        ));
    }
    // Gate G2: a near-grid-scale steep feature GAINS rank while it is marched — the transport is
    // doing real work, so the study is not reporting the rank of a field nothing happened to.
    //
    // The previous predicate was `b_peak < b0`. `march_ranks` initialises `peak = init` and only
    // ever `max`es into it, so `peak >= init` holds identically and that comparison could never be
    // true — it gated nothing. Requiring strict growth is the falsifiable form of the same intent.
    //
    // Measured: init 4 -> peak 8, so the margin is a factor of two.
    //
    // BREAKING CONDITION: make the marcher a no-op (or round hard enough to hold the initial
    // encode) and `b_peak == b0`, failing this gate.
    if b_peak <= b0 {
        failures.push(format!(
            "G2: steep low-nu feature gained no rank under marching (init={b0}, peak={b_peak}) — \
             the transport is not exercising the representation"
        ));
    }
    // Gate G3 (the lever, stated conservatively): more diffusion must not INCREASE the peak rank.
    if c_peak > b_peak {
        failures.push(format!(
            "G3: higher diffusion increased peak rank (low-nu peak={b_peak}, high-nu peak={c_peak})"
        ));
    }

    if failures.is_empty() {
        println!("\nALL GATES PASSED.");
        println!(
            "Finding: under fixed-tolerance rounding the linear marcher is RANK-SAFE — a transported"
        );
        println!(
            "feature settles near its static rank ({a_peak}/{b_peak}/{c_peak}), no runaway; more diffusion"
        );
        println!("does not increase rank (low-nu peak {b_peak} >= high-nu peak {c_peak}).");
        println!(
            "LIMIT: linear transport cannot STEEPEN a feature, so the nonlinear shock-steepening rank"
        );
        println!(
            "threat is NOT tested here — it needs a nonlinear marcher and remains OPEN for Tier-B."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Periodic `2^L`-grid march configuration shared by every run.
struct Grid {
    l: usize,
    dx: f64,
    dt: f64,
    c: f64,
    tol: f64,
    steps: usize,
    every: usize,
}

/// March `u0` on `g` at diffusivity `nu`, sampling `max_bond` every `g.every` steps.
/// Returns (initial_bond, peak_bond, final_bond, samples).
fn march_ranks(u0: &[f64], g: &Grid, nu: f64) -> (usize, usize, usize, Vec<(usize, usize)>) {
    let trunc = Truncation::<f64>::by_tol(g.tol).expect("tolerance");
    let solver = QttLinear1d::<f64>::new(g.l, g.dx, g.dt, g.c, nu, trunc).expect("marcher");
    let field = CausalTensor::new(u0.to_vec(), vec![u0.len()]).expect("dense field");
    let mut state = quantize(&field, &trunc).expect("encode");

    let init = state.max_bond();
    let mut peak = init;
    let mut samples = vec![(0usize, init)];
    for step in 1..=g.steps {
        state = solver.advance(&state, &()).expect("advance");
        let b = state.max_bond();
        peak = peak.max(b);
        if step % g.every == 0 {
            samples.push((step, b));
        }
    }
    (init, peak, state.max_bond(), samples)
}
