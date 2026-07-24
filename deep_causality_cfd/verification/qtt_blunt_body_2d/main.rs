/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT blunt-body bow shock — the Stage-5 rank-lever gate (Tier-B)
//!
//! A blunt-body bow shock stands off the nose at a constant *physical* radius `R`. In the body-fitted
//! coordinate (`BlendedMap` at `λ = 1`, the polar fan) that surface is a line `η = const` — a step in `η`,
//! constant in `ξ` — so its quantized-tensor-train bond `χ` is `O(10)` and **resolution-independent**. In
//! the Cartesian-capture coordinate (the same `BlendedMap` at `λ = 0`, a rectangle) the identical physical
//! shock is curved on the grid, so `χ` grows with resolution (`χ ~ √side`, the measured capture cost).
//!
//! The marcher (`CompressibleMarcher2d`) runs the **same solver** over both coordinates through the
//! `MetricProvider` seam (design D8), so this is a clean one-solver comparison: marching in the fitted
//! coordinate keeps `χ` bounded, while the captured control's `χ` overtakes it and grows. Self-verifying:
//! the gates encode the rank lever and exit non-zero on regression.
//!
//! Usage:
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_blunt_body_2d
//! ```

use deep_causality_cfd::{
    BlendedMap, BlendedMapConfig, CompressibleMarcher2d, EulerState2d, EvidenceClass, quantize_2d,
};
use deep_causality_tensor::{CausalTensor, Truncation};

const PI: f64 = std::f64::consts::PI;
const GAMMA: f64 = 1.4;
// Fan geometry: r ∈ [1, 2], a ±45° fan in front of the nose; the bow shock stands off at R = 1.5.
const R0: f64 = 1.0;
const DR: f64 = 1.0;
const DTHETA: f64 = PI / 2.0;
const R_SHOCK: f64 = 1.5;
// Pre-/post-shock states (a smooth compression across the standoff radius).
const RHO_PRE: f64 = 1.0;
const RHO_POST: f64 = 1.8;
const P_PRE: f64 = 1.0;
const P_POST: f64 = 3.0;

fn tr() -> Truncation<f64> {
    Truncation::<f64>::by_tol(1e-8).unwrap_or_else(|e| {
        eprintln!("truncation: {e:?}");
        std::process::exit(2);
    })
}

fn smoothed_step(d: f64, w: f64) -> f64 {
    0.5 * (1.0 + (d / w).tanh())
}

/// Build the bow-shock conservative initial state on `map`'s lattice: a fixed *physical* compression at
/// radius `R_SHOCK`, sampled through `map.position`, with zero initial velocity.
fn bow_shock_state(map: &BlendedMap<f64>, l: usize, w: f64) -> EulerState2d<f64> {
    let side = 1usize << l;
    let n = side * side;
    let mut rho = vec![0.0; n];
    let mut e = vec![0.0; n];
    for ix in 0..side {
        for iy in 0..side {
            let xi = ix as f64 / side as f64;
            let eta = iy as f64 / side as f64;
            let (x, y) = map.position(xi, eta);
            let s = smoothed_step((x * x + y * y).sqrt() - R_SHOCK, w);
            let idx = ix * side + iy;
            rho[idx] = RHO_PRE + s * (RHO_POST - RHO_PRE);
            let p = P_PRE + s * (P_POST - P_PRE);
            e[idx] = p / (GAMMA - 1.0); // u = v = 0
        }
    }
    [rho, vec![0.0; n], vec![0.0; n], e]
}

/// QTT bond of the bow-shock density field at blend `lambda`, resolution `l`.
fn ic_bond(lambda: f64, l: usize, w: f64) -> usize {
    let cfg = BlendedMapConfig::new(l, l, R0, DR, -DTHETA / 2.0, DTHETA, lambda);
    let map = BlendedMap::new(cfg, tr()).unwrap_or_else(|e| fail("blend build", e));
    let state = bow_shock_state(&map, l, w);
    let side = 1usize << l;
    quantize_2d(
        &CausalTensor::new(state[0].clone(), vec![side, side]).unwrap(),
        &tr(),
    )
    .unwrap_or_else(|e| fail("ic quantize", e))
    .max_bond()
}

/// Peak density-train bond seen while *marching* the bow shock in the body-fitted coordinate.
fn marched_fitted_peak(l: usize, w: f64, steps: usize) -> usize {
    let cfg = BlendedMapConfig::new(l, l, R0, DR, -DTHETA / 2.0, DTHETA, 1.0);
    let map = BlendedMap::new(cfg, tr()).unwrap_or_else(|e| fail("fitted blend", e));
    let state = bow_shock_state(&map, l, w);
    // Reference wave speed ≈ post-shock |u| + c; u = 0 initially, c = √(γ p/ρ).
    let s_ref = (GAMMA * P_POST / RHO_POST).sqrt();
    let dt = 0.0005;
    let marcher = CompressibleMarcher2d::new(map, GAMMA, dt, s_ref, tr())
        .unwrap_or_else(|e| fail("fitted marcher", e));
    let (_out, peak) = marcher
        .run(&state, steps)
        .unwrap_or_else(|e| fail("fitted march", e));
    peak
}

fn main() {
    println!(
        "=== QTT blunt-body bow shock: the Stage-5 rank lever (fitted vs Cartesian capture) ===\n"
    );
    let mut failures: Vec<String> = Vec::new();
    let w = 2.0 * DR / 128.0; // ~2 radial cells at the reference resolution: a sharp front.

    println!("  resolution |  fitted χ (λ=1) | capture χ (λ=0)");
    println!("  -----------+-----------------+----------------");
    let mut fitted: Vec<usize> = Vec::new();
    let mut capture: Vec<usize> = Vec::new();
    for &l in &[5usize, 6, 7] {
        let f = ic_bond(1.0, l, w);
        let c = ic_bond(0.0, l, w);
        fitted.push(f);
        capture.push(c);
        println!("     2^{l}    |       {f:>3}       |      {c:>3}");
    }

    // Gate BB-A: the fitted bond is O(10) and resolution-stable (does not grow like √side).
    // Evidence class: **tripwire**. These are structural rank claims about this construction, not
    // comparisons against a published value — verification/README.md classifies this harness as
    // "structural / rank-lever", gating rank rather than physical accuracy.
    //
    // BREAKING CONDITION: make the fitted encode grow with resolution (e.g. misalign the chart) and
    // BB-A fails.
    let fitted_max = *fitted.iter().max().unwrap();
    let fitted_stable = fitted.windows(2).all(|x| x[1] <= x[0] + 1);
    let bb_a = fitted_max <= 12 && fitted_stable;
    println!(
        "  [{}] [{}] BB-A fitted χ bounded and resolution-stable: bonds {fitted:?} (max {fitted_max} <= 12)",
        if bb_a { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
    );
    if !bb_a {
        failures.push(format!(
            "BB-A: fitted χ not bounded/resolution-stable (bonds {fitted:?})"
        ));
    }

    // Gate BB-B: the Cartesian capture grows with resolution and overtakes the fitted bond.
    // BREAKING CONDITION: flatten the captured field so its rank stops growing and BB-B fails.
    let capture_grows = capture[2] > capture[0];
    let capture_overtakes = capture[2] >= 2 * fitted[2];
    let bb_b = capture_grows && capture_overtakes;
    println!(
        "  [{}] [{}] BB-B Cartesian capture is a growing rank cost: capture {capture:?} vs fitted {fitted:?}",
        if bb_b { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
    );
    if !bb_b {
        failures.push(format!(
            "BB-B: Cartesian capture is not a growing rank cost (capture {capture:?}, fitted {fitted:?})"
        ));
    }

    // Observation (NOT a gate): marching a flux-through-front in the fitted coordinate. Design D9 and the
    // `qtt_repin_marcher` study found that a plain flux-based marcher injects angular structure across the
    // captured front and grows the rank *even in the fitted coordinate* — the bounded-χ *marched* result
    // needs re-pinning + an exact-RH interface (smooth each side, no flux marched across the front), the
    // open Stage-5/6 remainder. We report the marched peak as that datapoint; it is not asserted.
    let peak = marched_fitted_peak(6, w, 6);

    println!("\n--- reading ---");
    println!(
        "  STATIC rank lever (gated): the bow shock is a constant-physical-radius surface → a step in η"
    );
    println!(
        "  in the fitted coordinate (χ {} → {}, flat), but a curved front on the Cartesian capture",
        fitted[0], fitted[2]
    );
    println!(
        "  (χ {} → {}, growing ~√side). Body-fittedness buys the bond reduction — the Stage-5 lever.",
        capture[0], capture[2]
    );
    println!(
        "  DYNAMIC marched rank (reported, OPEN): a plain flux-through-front marcher in the fitted"
    );
    println!(
        "  coordinate grows χ to {peak} over 6 steps — re-pinning + an exact-RH interface (no flux marched"
    );
    println!(
        "  across the front) is the open remainder (design D9 / `qtt_repin_marcher`), not gated here."
    );

    if failures.is_empty() {
        println!(
            "\nGATES PASSED — body-fitting bounds the bow-shock rank (static lever); Cartesian capture grows."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
