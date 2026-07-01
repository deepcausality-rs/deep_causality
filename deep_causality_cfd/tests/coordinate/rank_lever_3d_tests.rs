/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 3-D rank lever (Stage 1.1c) — the measured precondition for the 3-D QTT compressible marcher.
//!
//! The SAME physical bow shock (a `tanh` shell standing off the nose at constant radius `R`) is encoded
//! two ways and its QTT bond dimension read at two resolutions:
//!
//!   * **Cartesian capture** (`CartesianIdentity3d`): a curved shell on a Cartesian grid — bond is high
//!     and **grows with resolution** (`χ ~ √side`).
//!   * **Body-fitted** (`BodyFittedCoordinate3d`): the same shell is the surface `ζ = const`, so the
//!     field is a step in `ζ` constant in `(ξ, η)` — bond is **low `O(10)` and resolution-flat**.
//!
//! This is the whole reason the fitted coordinate is mandatory for the marcher: it turns "variable mesh"
//! into "tensor rank", and the rank stays bounded only when the coordinate is aligned to the shock.

use deep_causality_cfd::{BodyFittedCoordinate3d, CartesianIdentity3d, MetricProvider3d};
use deep_causality_tensor::Truncation;

const TAU: f64 = core::f64::consts::TAU;

fn trunc() -> Truncation<f64> {
    Truncation::by_tol(1e-8).unwrap()
}

/// Bond of the bow shock captured on a Cartesian grid: `tanh((|x − c| − R)/δ)` over the unit box.
fn cartesian_bond(l: usize) -> usize {
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let cart = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, trunc()).unwrap();
    let u = cart
        .sample(|xi, eta, zeta| {
            let (x, y, z) = (xi - 0.5, eta - 0.5, zeta - 0.5);
            let r = (x * x + y * y + z * z).sqrt();
            ((r - 0.3) / 0.03).tanh()
        })
        .unwrap();
    u.max_bond()
}

/// Bond of the same shell in the fitted coordinate: `tanh((r(ζ) − R)/δ)`, a function of `ζ` only.
fn body_fitted_bond(l: usize) -> usize {
    // r ∈ [0.05, 0.55] straddles the shock at R = 0.3; θ ∈ (0, π); full azimuth.
    let shell = BodyFittedCoordinate3d::<f64>::new(l, l, l, 0.05, 0.5, 0.3, 2.0, 0.0, TAU, trunc())
        .unwrap();
    let u = shell
        .sample(|_xi, _eta, zeta| {
            let r = 0.05 + zeta * 0.5;
            ((r - 0.3) / 0.03).tanh()
        })
        .unwrap();
    u.max_bond()
}

#[test]
fn body_fitted_holds_low_flat_rank_while_cartesian_grows() {
    let (cart_lo, cart_hi) = (cartesian_bond(4), cartesian_bond(5));
    let (bf_lo, bf_hi) = (body_fitted_bond(4), body_fitted_bond(5));

    // Body-fitted: low and resolution-flat (the shock is a ζ-step; bond is O(1), independent of N).
    assert!(bf_hi <= 4, "body-fitted bond should be O(1), got {bf_hi}");
    assert_eq!(
        bf_lo, bf_hi,
        "body-fitted bond must be resolution-flat: {bf_lo} vs {bf_hi}"
    );

    // Cartesian capture: high and growing with resolution (the √side threat, dynamically).
    assert!(
        cart_hi > cart_lo,
        "captured curved shock bond should grow with resolution: {cart_lo} -> {cart_hi}"
    );

    // The lever: at the finer grid the fitted coordinate is dramatically cheaper.
    assert!(
        cart_hi >= 3 * bf_hi.max(1),
        "fitted coordinate must be much cheaper than capture: cart {cart_hi} vs fitted {bf_hi}"
    );
}
