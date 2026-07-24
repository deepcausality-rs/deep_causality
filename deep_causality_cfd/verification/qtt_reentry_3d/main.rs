/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT 3-D re-entry forebody sheath — the Stage-6 rank-lever gate (Tier-B)
//!
//! The re-entry **forebody sheath** is a curved bow-shock surface standing off the nose at a constant
//! *physical* radius `R`. In a body-fitted spherical coordinate (radial axis `ζ` across the shock) that
//! surface is a step in `ζ` — a function of one axis — so its quantized-tensor-train bond `χ` is `O(1)`
//! and **resolution-independent**. Sampled on a Cartesian `2^l × 2^l × 2^l` lattice the identical physical
//! shell is curved, so `χ` grows with resolution (the `qtt_rank_3d` upper bound). This is the 3-D form of
//! the Stage-5 rank lever, on the crate's serial `x`-`y`-`z` codec (`quantize_3d`).
//!
//! Scope (design D9): the **forebody** is in scope and gated. The **wake** is out of scope — a
//! separated/unsteady wake needs turbulence (a non-goal) and is a multi-feature structure no single fitted
//! coordinate aligns; its bond is reported as an out-of-scope datapoint for the standing `qtt_rank_3d`
//! research question, never gated. The dynamic *marched* forebody rank (the Cartesian 3-D marcher, no 3-D
//! body-fit metric yet) is likewise reported, not gated — re-pinning + an exact-RH interface is the open
//! remainder.
//!
//! Usage:
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_reentry_3d
//! ```

use deep_causality_cfd::{CompressibleMarcher3d, EulerState3d, EvidenceClass, quantize_3d};
use deep_causality_tensor::{CausalTensor, Truncation};

const GAMMA: f64 = 1.4;
const R_SHOCK: f64 = 1.5; // standoff radius
const R0: f64 = 1.0; // body-fitted radial range [r0, r0+dr]
const DR: f64 = 1.0;
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

/// Bond of a `side³` dense field via the crate's serial-`xyz` 3-D codec.
fn bond_of(field: Vec<f64>, side: usize) -> usize {
    quantize_3d(
        &CausalTensor::new(field, vec![side, side, side]).unwrap(),
        &tr(),
    )
    .unwrap_or_else(|e| fail("quantize_3d", e))
    .max_bond()
}

/// Forebody sheath in the **body-fitted** coordinate: a step in the radial axis `ζ` (the `z` index), so
/// the shell is a function of one axis only — low rank by alignment.
fn forebody_fitted_bond(l: usize, w: f64) -> usize {
    let side = 1usize << l;
    let mut field = vec![0.0f64; side * side * side];
    for ix in 0..side {
        for iy in 0..side {
            for iz in 0..side {
                let r = R0 + (iz as f64 / side as f64) * DR; // ζ → physical radius
                field[(ix * side + iy) * side + iz] = smoothed_step(r - R_SHOCK, w);
            }
        }
    }
    bond_of(field, side)
}

/// The same physical spherical sheath sampled on a Cartesian `[-2,2]³` lattice: curved → rank grows.
fn forebody_cartesian_bond(l: usize, w: f64) -> usize {
    let side = 1usize << l;
    let mut field = vec![0.0f64; side * side * side];
    for ix in 0..side {
        for iy in 0..side {
            for iz in 0..side {
                let x = -2.0 + 4.0 * ix as f64 / side as f64;
                let y = -2.0 + 4.0 * iy as f64 / side as f64;
                let z = -2.0 + 4.0 * iz as f64 / side as f64;
                let r = (x * x + y * y + z * z).sqrt();
                field[(ix * side + iy) * side + iz] = smoothed_step(r - R_SHOCK, w);
            }
        }
    }
    bond_of(field, side)
}

/// The **wake** (out of scope): a separated off-axis recirculation feature — not a single fitted surface.
fn wake_bond(l: usize, w: f64) -> usize {
    let side = 1usize << l;
    let mut field = vec![0.0f64; side * side * side];
    for ix in 0..side {
        for iy in 0..side {
            for iz in 0..side {
                let x = -2.0 + 4.0 * ix as f64 / side as f64;
                let y = -2.0 + 4.0 * iy as f64 / side as f64;
                let z = -2.0 + 4.0 * iz as f64 / side as f64;
                // Two off-axis lobes downstream of the body — a multi-feature, un-fittable structure.
                let d1 = ((x - 0.6).powi(2) + (y - 0.6).powi(2) + (z + 1.2).powi(2)).sqrt();
                let d2 = ((x + 0.6).powi(2) + (y + 0.6).powi(2) + (z + 1.2).powi(2)).sqrt();
                field[(ix * side + iy) * side + iz] =
                    smoothed_step(0.4 - d1, w) + smoothed_step(0.4 - d2, w);
            }
        }
    }
    bond_of(field, side)
}

/// Peak density-train bond while marching the Cartesian forebody a few steps (the dynamic datapoint).
fn marched_forebody_peak(l: usize, w: f64, steps: usize) -> usize {
    let side = 1usize << l;
    let n = side * side * side;
    let dx = 4.0 / side as f64;
    let mut rho = vec![0.0; n];
    let mut e = vec![0.0; n];
    for ix in 0..side {
        for iy in 0..side {
            for iz in 0..side {
                let x = -2.0 + 4.0 * ix as f64 / side as f64;
                let y = -2.0 + 4.0 * iy as f64 / side as f64;
                let z = -2.0 + 4.0 * iz as f64 / side as f64;
                let s = smoothed_step((x * x + y * y + z * z).sqrt() - R_SHOCK, w);
                let idx = (ix * side + iy) * side + iz;
                rho[idx] = RHO_PRE + s * (RHO_POST - RHO_PRE);
                e[idx] = (P_PRE + s * (P_POST - P_PRE)) / (GAMMA - 1.0);
            }
        }
    }
    let state: EulerState3d<f64> = [rho, vec![0.0; n], vec![0.0; n], vec![0.0; n], e];
    let s_ref = (GAMMA * P_POST / RHO_POST).sqrt();
    let marcher = CompressibleMarcher3d::new((l, l, l), dx, GAMMA, 0.0005, s_ref, tr())
        .unwrap_or_else(|e| fail("3-D marcher", e));
    let (_out, peak) = marcher
        .run(&state, steps)
        .unwrap_or_else(|e| fail("3-D march", e));
    peak
}

fn main() {
    println!("=== QTT 3-D re-entry forebody sheath: the Stage-6 rank lever ===\n");
    let mut failures: Vec<String> = Vec::new();
    let w = 4.0 / 32.0; // ~1 cell at the reference resolution: a sharp front.

    println!("  resolution | fitted χ (fn ζ) | Cartesian χ");
    println!("  -----------+-----------------+------------");
    let mut fitted: Vec<usize> = Vec::new();
    let mut cart: Vec<usize> = Vec::new();
    for &l in &[3usize, 4, 5] {
        let f = forebody_fitted_bond(l, w);
        let c = forebody_cartesian_bond(l, w);
        fitted.push(f);
        cart.push(c);
        println!("     2^{l}    |       {f:>3}       |     {c:>3}");
    }

    // Gate RE-A: the body-fitted forebody bond is O(1), and resolution-independent *at scale* — the bond
    // plateaus (a flat high-resolution tail), not growing like the Cartesian capture.
    // Evidence class: **tripwire** for both. Structural rank claims about this construction, not
    // published values — verification/README.md classifies this harness as "structural /
    // rank-lever", gating rank rather than physical accuracy.
    //
    // BREAKING CONDITION: make the fitted encode grow with resolution and RE-A fails.
    let fitted_max = *fitted.iter().max().unwrap();
    let tail_flat = fitted[2] <= fitted[1] + 1;
    let re_a = fitted_max <= 8 && tail_flat;
    println!(
        "  [{}] [{}] RE-A fitted forebody χ bounded and resolution-stable: bonds {fitted:?} (max {fitted_max} <= 8)",
        if re_a { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
    );
    if !re_a {
        failures.push(format!(
            "RE-A: fitted forebody χ not bounded/resolution-stable (bonds {fitted:?})"
        ));
    }
    // Gate RE-B: the Cartesian capture grows with resolution and overtakes the fitted bond.
    // BREAKING CONDITION: flatten the captured field so its rank stops growing and RE-B fails.
    let re_b = cart[2] > cart[0] && cart[2] >= 2 * fitted[2].max(1);
    println!(
        "  [{}] [{}] RE-B Cartesian forebody capture is a growing rank cost: cart {cart:?} vs fitted {fitted:?}",
        if re_b { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
    );
    if !re_b {
        failures.push(format!(
            "RE-B: Cartesian forebody capture is not a growing rank cost (cart {cart:?}, fitted {fitted:?})"
        ));
    }

    // Out-of-scope datapoints (reported, NOT gated).
    let wake = wake_bond(5, w);
    let marched = marched_forebody_peak(3, w, 6);

    println!("\n--- reading ---");
    println!(
        "  FOREBODY (gated): the bow-shock sheath is a constant-radius surface → a step in the radial"
    );
    println!(
        "  axis ζ (χ {} → {}, flat), but a curved shell on Cartesian (χ {} → {}, growing). Body-fitting",
        fitted[0], fitted[2], cart[0], cart[2]
    );
    println!(
        "  bounds the forebody rank — the Stage-6 lever (the `qtt_rank_3d` upper bound, in the codec)."
    );
    println!(
        "  WAKE (out of scope): a separated multi-lobe wake has χ = {wake} at 2^5 — no single fitted"
    );
    println!(
        "  coordinate aligns it; reported for the standing `qtt_rank_3d` question, never gated (D9)."
    );
    println!(
        "  DYNAMIC marched forebody (open): the Cartesian 3-D marcher grows χ to {marched} over 6 steps —"
    );
    println!("  a 3-D body-fit metric + re-pinning is the open remainder, not gated here.");

    if failures.is_empty() {
        println!(
            "\nGATES PASSED — body-fitting bounds the 3-D forebody sheath rank; Cartesian capture grows; wake out of scope."
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
