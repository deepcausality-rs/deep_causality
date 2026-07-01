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

use deep_causality_cfd::quantize_2d;
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
    let span_y = 2.0 * RSHOCK * (DTHETA / 2.0).sin(); // fan width at the standoff radius

    println!(
        "=== QTT blend-metric (Res 4 / D8): body-fit as a valid low-rank free parameter ===\n"
    );
    println!(
        "  T_lambda = (1-lambda)*Cartesian + lambda*BodyFitted, over a {side}x{side} (xi,eta) patch\n"
    );

    let lambdas = [0.0, 0.25, 0.5, 0.75, 1.0];
    let delta = 2.0 * DR / side as f64; // ~2 radial cells: a sharp front

    println!("  lambda | min|detJ|  detJ sign |  shock-field bond");
    println!("  -------+----------------------+------------------");
    let mut min_det_overall = f64::INFINITY;
    let mut sign_consistent = true;
    let mut first_sign = 0i32;
    let mut bonds: Vec<usize> = Vec::new();
    for &lam in &lambdas {
        let (min_abs_det, sign) = jacobian_scan(side, lam, span_y);
        min_det_overall = min_det_overall.min(min_abs_det);
        if first_sign == 0 {
            first_sign = sign;
        } else if sign != first_sign {
            sign_consistent = false;
        }
        let bond = shock_field_bond(side, lam, span_y, delta, tol);
        bonds.push(bond);
        let signs = if sign >= 0 { "+" } else { "-" };
        println!("   {lam:>4.2} | {min_abs_det:>10.4}        {signs:>3}    | {bond:>6}");
    }

    let bond_cart = bonds[0];
    let bond_fit = *bonds.last().unwrap();

    // Gate BM-A: the blend never folds — det J stays one sign and bounded away from zero across λ.
    if !sign_consistent || min_det_overall <= 1e-6 {
        failures.push(format!(
            "BM-A: blend folds or det J vanishes (min|detJ|={min_det_overall:.2e}, sign_consistent={sign_consistent})"
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
        "  Validity: min|detJ| = {min_det_overall:.3} > 0 with one sign across the whole sweep — the"
    );
    println!(
        "  position-blend of two compatibly-oriented charts does NOT fold. The Res-4 residual holds"
    );
    println!(
        "  (with a bounded-λ-gradient + positive-Jacobian guard, which here is satisfied by construction)."
    );
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

/// Scan `det J` of `T_λ` over the lattice; return `(min|det J|, sign)`.
fn jacobian_scan(side: usize, lam: f64, span_y: f64) -> (f64, i32) {
    let mut min_abs = f64::INFINITY;
    let mut sign = 0i32;
    for ix in 0..side {
        for iy in 0..side {
            let xi = ix as f64 / side as f64;
            let eta = iy as f64 / side as f64;
            let theta = -DTHETA / 2.0 + xi * DTHETA;
            let r = R0 + eta * DR;
            // ∂(x,y)/∂(ξ,η) for each chart, blended linearly in λ.
            // Cartesian: ∂x/∂ξ=0, ∂x/∂η=DR, ∂y/∂ξ=span_y, ∂y/∂η=0.
            // Fitted:    ∂x/∂ξ=-r·sinθ·Δθ, ∂x/∂η=cosθ·DR, ∂y/∂ξ=r·cosθ·Δθ, ∂y/∂η=sinθ·DR.
            let dxdxi = (1.0 - lam) * 0.0 + lam * (-r * theta.sin() * DTHETA);
            let dxdeta = (1.0 - lam) * DR + lam * (theta.cos() * DR);
            let dydxi = (1.0 - lam) * span_y + lam * (r * theta.cos() * DTHETA);
            let dydeta = (1.0 - lam) * 0.0 + lam * (theta.sin() * DR);
            let det = dxdxi * dydeta - dxdeta * dydxi;
            if det.abs() < min_abs {
                min_abs = det.abs();
            }
            let s = if det >= 0.0 { 1 } else { -1 };
            if sign == 0 {
                sign = s;
            } else if s != sign {
                sign = 2; // mixed-sign marker (a fold)
            }
        }
    }
    (min_abs, sign)
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
