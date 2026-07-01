// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT rank study — the *static* make-or-break experiment for the Tier-B compressible thesis.
//!
//! The Tier-B claim is "micrometre-over-metre resolution for free, because the flowfield is low
//! tensor-train rank." This example measures the actual QTT bond dimension of shock-like profiles
//! with the real codec and answers three questions, in order:
//!
//! 1. Is a discontinuity intrinsically high rank? No — a 1-D step is rank <= 2 at any position.
//! 2. What drives the rank up? Mis-alignment with the coordinate axes in 2-D, straight or curved — not
//!    "sharpness".
//! 3. Can it be made low-rank by construction? Yes — a shock-aligned / body-fitted coordinate collapses
//!    a curved bow shock from ~150 to ~5.
//!
//! It self-verifies: the gates below encode the findings and exit non-zero on regression.
//! See `README.md` and `openspec/notes/plasma-blackout/gap-2/` for the analysis.

use deep_causality_cfd::{quantize, quantize_2d};
use deep_causality_tensor::{CausalTensor, Truncation};

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let tol = 1e-8;

    // ===============================================================
    // 1-D: a discontinuity is NOT intrinsically high rank.
    // ===============================================================
    let l = 12usize;
    let n = 1usize << l;
    let x_nd = 1.0 / std::f64::consts::PI; // a non-dyadic ("moving") shock location

    println!("=== 1-D QTT rank study  (L={l}, N={n}, tol={tol:e}) ===");
    let sine: Vec<f64> = (0..n)
        .map(|i| (std::f64::consts::TAU * i as f64 / n as f64).sin())
        .collect();
    let step_dyadic: Vec<f64> = (0..n).map(|i| if i < n / 2 { 0.0 } else { 1.0 }).collect();
    let idx_nd = (x_nd * n as f64) as usize;
    let step_nondyadic: Vec<f64> = (0..n).map(|i| if i < idx_nd { 0.0 } else { 1.0 }).collect();
    let r_sine = rank_1d(&sine, tol);
    let r_dy = rank_1d(&step_dyadic, tol);
    let r_nd = rank_1d(&step_nondyadic, tol);
    println!("  smooth sine .............. {r_sine}");
    println!("  sharp step, dyadic ....... {r_dy}");
    println!("  sharp step, NON-dyadic ... {r_nd}");

    // Captured reentry stagnation line: uniform | shock jump | relaxation tail.
    let (peak, eq, tau) = (1.0, 0.3, 0.05);
    let captured: Vec<f64> = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            if x < x_nd {
                0.0
            } else {
                eq + (peak - eq) * (-(x - x_nd) / tau).exp()
            }
        })
        .collect();
    let r_cap = rank_1d(&captured, tol);
    println!(
        "  captured stagnation line . {r_cap}  (1-D is cheap either way -> fitting unneeded in 1-D)"
    );

    // Gate G1: a sharp 1-D step / captured 1-D profile stays low rank.
    if r_nd > 4 || r_cap > 6 {
        failures.push(format!(
            "G1: 1-D discontinuity unexpectedly high rank (step={r_nd}, captured={r_cap})"
        ));
    }

    // ===============================================================
    // 2-D: the rank explosion is ALIGNMENT, not curvature.
    // ===============================================================
    let l2 = 9usize;
    let side = 1usize << l2;
    let d2 = 2.0 / side as f64; // ~2-cell thickened shock

    println!(
        "\n=== 2-D QTT rank study  (Lx=Ly={l2}, {side}x{side}, delta=2 cells, tol={tol:e}) ==="
    );
    let flat = build_2d(side, &|x, _| 0.5 * (1.0 + ((x - 0.5) / d2).tanh()));
    let oblique = build_2d(side, &|x, y| {
        0.5 * (1.0 + (((x + y) * 0.5 - 0.5) / d2).tanh())
    });
    let bow = build_2d(side, &|x, y| {
        let r = ((x - 0.2).powi(2) + (y - 0.5).powi(2)).sqrt();
        0.5 * (1.0 + ((r - 0.35) / d2).tanh())
    });
    let r_flat = rank_2d(&flat, side, tol);
    let r_obl = rank_2d(&oblique, side, tol);
    let r_bow = rank_2d(&bow, side, tol);
    println!("  Cartesian, FLAT (axis-aligned) .. {r_flat}");
    println!("  Cartesian, OBLIQUE (45 deg) ..... {r_obl}  (straight, yet worse than the curve)");
    println!("  Cartesian, CURVED bow ........... {r_bow}");

    // The fix: a coordinate that aligns the front to an axis -> function of one variable.
    let bow_polar = build_2d(side, &|r_axis, _theta| {
        0.5 * (1.0 + ((r_axis * 0.7 - 0.35) / d2).tanh())
    });
    let obl_aligned = build_2d(side, &|s, _t| 0.5 * (1.0 + ((s - 0.5) / d2).tanh()));
    let r_polar = rank_2d(&bow_polar, side, tol);
    let r_obl_al = rank_2d(&obl_aligned, side, tol);
    println!("  body-fitted, bow as fn(r) ....... {r_polar}  (was {r_bow})");
    println!("  shock-aligned, oblique .......... {r_obl_al}  (was {r_obl})");

    // Gate G2: capturing a misaligned (curved or oblique) shock is expensive (>= 8x flat).
    if r_bow < 8 * r_flat || r_obl < 8 * r_flat {
        failures.push(format!(
            "G2: misaligned 2-D shock not expensive as expected (flat={r_flat}, bow={r_bow}, oblique={r_obl})"
        ));
    }
    // Gate G3: a shock-aligned coordinate collapses it back to ~flat (<= 2x flat).
    if r_polar > 2 * r_flat || r_obl_al > 2 * r_flat {
        failures.push(format!(
            "G3: aligned coordinate did not collapse the rank (flat={r_flat}, polar={r_polar}, obl_aligned={r_obl_al})"
        ));
    }

    // Quantify: cost ~ O(chi^2 * L). For 2^18 = 262144 dense values:
    let dense = (1usize << (2 * l2)) as f64;
    let params = |chi: usize| (2 * 2 * l2 * chi * chi) as f64; // ~ rough TT param count
    println!("\n  cost proxy (dense = {} values):", dense as usize);
    for (name, chi) in [
        ("flat/fitted", r_flat),
        ("curved Cartesian", r_bow),
        ("oblique Cartesian", r_obl),
    ] {
        let p = params(chi);
        let ratio = dense / p;
        let verdict = if ratio >= 1.0 {
            format!("{ratio:.0}x smaller")
        } else {
            format!("{:.1}x LARGER", 1.0 / ratio)
        };
        println!(
            "    {name:<18} chi={chi:<4} ~{:>8} params  -> {verdict} than dense",
            p as usize
        );
    }

    // ---------------------------------------------------------------
    if failures.is_empty() {
        println!(
            "\nALL GATES PASSED — discontinuity != high rank; alignment is the lever; the fix collapses it."
        );
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Max bond dimension of a 1-D profile of length `2^L`, encoded at relative tolerance `tol`.
fn rank_1d(profile: &[f64], tol: f64) -> usize {
    let n = profile.len();
    let t = CausalTensor::new(profile.to_vec(), vec![n]).expect("dense 1-D tensor");
    let trunc = Truncation::<f64>::by_tol(tol).expect("tolerance");
    quantize(&t, &trunc).expect("quantize 1-D").max_bond()
}

/// Max bond dimension of a `side x side` field (row-major), encoded at relative tolerance `tol`.
fn rank_2d(field: &[f64], side: usize, tol: f64) -> usize {
    let t = CausalTensor::new(field.to_vec(), vec![side, side]).expect("dense 2-D tensor");
    let trunc = Truncation::<f64>::by_tol(tol).expect("tolerance");
    quantize_2d(&t, &trunc).expect("quantize 2-D").max_bond()
}

fn build_2d(side: usize, f: &dyn Fn(f64, f64) -> f64) -> Vec<f64> {
    let dx = 1.0 / side as f64;
    let mut v = vec![0.0; side * side];
    for ix in 0..side {
        for iy in 0..side {
            v[ix * side + iy] = f(ix as f64 * dx, iy as f64 * dx);
        }
    }
    v
}
