// SPDX-License-Identifier: MIT
// Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

//! QTT blend-metric study — is body-fittedness a valid, low-rank *free parameter*? (Resolution 4 / D8.)
//!
//! Resolution 4 promotes the coordinate to a `MetricProvider` with a continuous blend
//! `T_λ = (1−λ)·T_cart + λ·T_fit` — `λ = 0` is Cartesian capture (any geometry, high rank), `λ = 1` is
//! full body-fitting (this geometry, low rank). Two claims must hold for the blend to be a usable dial:
//!
//!   BM-A  **validity** — `T_λ` stays a valid map (no folded cells: `det J` non-vanishing, one sign)
//!         across the whole sweep `λ ∈ [0,1]`. This is the one open residual Res 4 flagged.
//!   BM-B  **rank dial** — a fixed *physical* curved shock, sampled on the `λ`-blended lattice, runs
//!         monotonically from high rank (`λ=0`, the curved front on a square grid) to `O(10)` (`λ=1`,
//!         the front aligned to the radial axis). So `λ` really is a rank knob, and intermediate `λ`
//!         gives intermediate rank at zero asymptotic cost.
//!
//! Here `T_cart` and `T_fit` are compatibly-oriented charts over the same `(ξ,η)` patch in front of the
//! nose (η ≈ radial / across the shock, ξ ≈ transverse). Self-verifying: gates exit non-zero on
//! regression.

use deep_causality_cfd::{BlendedMap, BlendedMapConfig, quantize_2d};
use deep_causality_tensor::{CausalTensor, Truncation};

const PI: f64 = std::f64::consts::PI;

// Geometry: nose at the origin; a fan in front of it.
const R0: f64 = 1.0; // inner radius
const DR: f64 = 1.0; // radial extent  (r ∈ [1, 2])
const DTHETA: f64 = PI / 2.0; // angular extent (±45°)
const RSHOCK: f64 = 1.5; // standoff radius of the test shock

fn main() {
    let mut failures: Vec<String> = Vec::new();
    let tol = 1e-8;
    let l = 8usize;
    let side = 1usize << l;
    // The Cartesian-capture chart's transverse width. This MUST be `BlendedMap`'s definition — the
    // fan chord at *mid radius*, `2·(r0+½dr)·sin(½dθ)` — because both gates below measure the chart
    // `BlendedMap` builds. It was previously written as `2·RSHOCK·sin(½dθ)`, the fan width at the
    // *standoff* radius, which is a different quantity that happens to coincide here: `r0+½dr = 1.5`
    // and `RSHOCK = 1.5`. The shock standoff is a physically independent parameter, so moving it
    // (say to 1.6) would have silently pointed both gates at a chart the crate never constructs,
    // 6.7% wide of it.
    let span_y = 2.0 * (R0 + 0.5 * DR) * (DTHETA / 2.0).sin();

    println!(
        "=== QTT blend-metric (Res 4 / D8): body-fit as a valid low-rank free parameter ===\n"
    );
    println!(
        "  T_lambda = (1-lambda)*Cartesian + lambda*BodyFitted, over a {side}x{side} (xi,eta) patch\n"
    );

    let lambdas = [0.0, 0.25, 0.5, 0.75, 1.0];
    let delta = 2.0 * DR / side as f64; // ~2 radial cells: a sharp front

    println!("  lambda | min|detJ|   floor      margin |  shock-field bond");
    println!("  -------+-----------------------------+------------------");
    let mut min_det_overall = f64::INFINITY;
    let mut margin_overall = f64::INFINITY;
    let mut rejected: Vec<String> = Vec::new();
    let mut bonds: Vec<usize> = Vec::new();
    let trunc = Truncation::<f64>::by_tol(tol).expect("a positive tolerance is a valid truncation");
    for &lam in &lambdas {
        // BM-A runs through the **shipped** constructor. `BlendedMap::new` scans `det J_λ` over the
        // closed domain and refuses a fold or a near-singular map, so a rejection here *is* the gate
        // firing; `det_margin` then reports the number that scan measured. The study previously
        // carried its own copy of the Jacobian algebra, which meant BM-A verified the copy — a green
        // gate said nothing about the map the crate actually builds.
        let cfg = BlendedMapConfig::new(l, l, R0, DR, -DTHETA / 2.0, DTHETA, lam);
        match BlendedMap::new(cfg, trunc) {
            Ok(map) => {
                let (min_abs_det, floor) = map.det_margin();
                min_det_overall = min_det_overall.min(min_abs_det);
                margin_overall = margin_overall.min(min_abs_det / floor);
                let bond = shock_field_bond(side, lam, span_y, delta, tol);
                bonds.push(bond);
                println!(
                    "   {lam:>4.2} | {min_abs_det:>9.4}  {floor:>9.2e}  {:>8.1e} | {bond:>6}",
                    min_abs_det / floor
                );
            }
            Err(e) => {
                rejected.push(format!("lambda={lam:.2}: {e}"));
                println!(
                    "   {lam:>4.2} |            REJECTED BY BlendedMap::new            |      -"
                );
            }
        }
    }

    // BM-A rejections leave `bonds` short, so BM-B has nothing to read. Report the gate failures
    // rather than indexing into an empty vector — a panic here would exit non-zero for the wrong
    // reason and bury which gate actually fired.
    let (bond_cart, bond_fit) = match (bonds.first(), bonds.last()) {
        (Some(c), Some(f)) => (*c, *f),
        _ => {
            eprintln!("\nFAILED GATES:");
            eprintln!(
                "  - BM-A: BlendedMap::new refused every sweep point, so BM-B has no bond series \
                 to judge — {}",
                rejected.join("; ")
            );
            std::process::exit(1);
        }
    };

    // Gate BM-A: the blend never folds — `det J` stays one sign and bounded away from zero across λ.
    //
    // The sign and floor checks live in `BlendedMap::new`, so a rejection above is the failure. The
    // floor is a fraction of the geometric scale `dr · span_y`, not an absolute constant: `det` is an
    // area ratio, so an absolute bound would mean different things at different geometries.
    if !rejected.is_empty() {
        failures.push(format!(
            "BM-A: BlendedMap::new refused the blend at {} of {} sweep points — {}",
            rejected.len(),
            lambdas.len(),
            rejected.join("; ")
        ));
    }

    // Gate BM-B: λ is a genuine rank dial — fitted (λ=1) is O(10), Cartesian (λ=0) is several× larger,
    // and the sweep is monotone non-increasing (each step within a small slack).
    let monotone = bonds.windows(2).all(|w| w[1] <= w[0] + 1);
    if bond_fit > 16 || bond_cart < 3 * bond_fit || !monotone {
        failures.push(format!(
            "BM-B: λ is not a clean rank dial (cart={bond_cart}, fit={bond_fit}, monotone={monotone})"
        ));
    }

    println!("\n--- reading ---");
    println!(
        "  Validity: min|detJ| = {min_det_overall:.3}, which is {margin_overall:.1e}x the floor that"
    );
    println!(
        "  BlendedMap::new accepts against — the position-blend of two compatibly-oriented charts"
    );
    println!("  does NOT fold anywhere on this sweep.");
    println!(
        "  Both numbers come from the shipped constructor's own scan, so the gate measures the map"
    );
    println!("  the crate builds rather than a copy of its algebra.");
    println!(
        "  Rank dial: bond runs {bond_cart} (λ=0, Cartesian capture) -> {bond_fit} (λ=1, body-fitted),"
    );
    println!(
        "  monotonically — body-fittedness is a continuous low-rank free parameter, exactly as D8 claims."
    );

    if failures.is_empty() {
        println!("\nALL GATES PASSED — the blend is valid and λ is a clean rank dial.");
    } else {
        eprintln!("\nFAILED GATES:");
        for f in &failures {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
}

/// Physical position of the blended chart `T_λ` at computational `(ξ, η)`.
fn position(xi: f64, eta: f64, lam: f64, span_y: f64) -> (f64, f64) {
    // Cartesian chart: η → x (radial), ξ → y (transverse).
    let xc = R0 + eta * DR;
    let yc = -span_y / 2.0 + xi * span_y;
    // Body-fitted (polar fan) chart.
    let theta = -DTHETA / 2.0 + xi * DTHETA;
    let r = R0 + eta * DR;
    let xf = r * theta.cos();
    let yf = r * theta.sin();
    ((1.0 - lam) * xc + lam * xf, (1.0 - lam) * yc + lam * yf)
}

/// QTT bond of a fixed physical shock (`tanh` at radius `RSHOCK`) sampled on the `λ`-blended lattice.
fn shock_field_bond(side: usize, lam: f64, span_y: f64, delta: f64, tol: f64) -> usize {
    let mut data = vec![0.0f64; side * side];
    for ix in 0..side {
        for iy in 0..side {
            let xi = ix as f64 / side as f64;
            let eta = iy as f64 / side as f64;
            let (x, y) = position(xi, eta, lam, span_y);
            let dist = (x * x + y * y).sqrt(); // distance from the nose at the origin
            data[ix * side + iy] = 0.5 * (1.0 + ((dist - RSHOCK) / delta).tanh());
        }
    }
    let trunc = Truncation::<f64>::by_tol(tol).expect("tol");
    quantize_2d(
        &CausalTensor::new(data, vec![side, side]).expect("dense"),
        &trunc,
    )
    .expect("encode")
    .max_bond()
}
