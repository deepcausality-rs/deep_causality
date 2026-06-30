/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `MetricProvider` seam (design D8): `CartesianIdentity` correctness, `BodyFittedCoordinate`
//! reached through the trait, and a generic `over M: MetricProvider` consumer (static dispatch).

use deep_causality_cfd::{BodyFittedCoordinate, CartesianIdentity, MetricProvider, dequantize_2d};
use deep_causality_tensor::Truncation;

const TAU: f64 = core::f64::consts::TAU;

fn trunc() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

/// A generic consumer: proves both providers satisfy the bound and are usable by static dispatch.
fn lattice_size<M: MetricProvider<f64>>(m: &M) -> usize {
    let (lx, ly) = m.dims();
    (1usize << lx) * (1usize << ly)
}

#[test]
fn cartesian_identity_physical_gradient_matches_analytic() {
    // Identity chart on [0,1)²: x = ξ, so u = sin(2πξ) = sin(2πx) and ∂u/∂x = 2π cos(2πx), ∂u/∂y = 0.
    let l = 7usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let m = CartesianIdentity::<f64>::new(l, l, dx, dx, trunc()).unwrap();
    let u = m.sample(|xi, _eta| (TAU * xi).sin()).unwrap();

    let (dudx, dudy) = m.physical_gradient(&u).unwrap();
    let gx = dequantize_2d(&dudx, l, l).unwrap();
    let gy = dequantize_2d(&dudy, l, l).unwrap();
    let gxs = gx.as_slice();
    let gys = gy.as_slice();

    let mut max_err_x = 0.0f64;
    let mut max_abs_y = 0.0f64;
    for ix in 0..n {
        let x = ix as f64 / n as f64;
        let exact = TAU * (TAU * x).cos();
        // Field is constant in y, so the iy = 0 column holds the same x-gradient.
        let got_x = gxs[ix * n];
        max_err_x = max_err_x.max((got_x - exact).abs());
        max_abs_y = max_abs_y.max(gys[ix * n].abs());
    }
    // 2nd-order central difference: error ~ (2π)³/6 · dx² ≈ 0.015 at N=128.
    assert!(max_err_x < 0.05, "∂/∂x error too large: {max_err_x}");
    assert!(
        max_abs_y < 1e-9,
        "∂/∂y of an x-only field should vanish: {max_abs_y}"
    );
}

#[test]
fn cartesian_identity_jacobian_is_cell_volume() {
    let l = 5usize;
    let n = 1usize << l;
    let (dx, dy) = (1.0 / n as f64, 2.0 / n as f64);
    let m = CartesianIdentity::<f64>::new(l, l, dx, dy, trunc()).unwrap();
    let jac = dequantize_2d(m.jacobian(), l, l).unwrap();
    let cell = dx * dy;
    for v in jac.as_slice() {
        assert!(
            (v - cell).abs() < 1e-12,
            "jacobian should be the constant cell volume {cell}, got {v}"
        );
    }
}

#[test]
fn body_fitted_reached_through_the_trait_matches_inherent() {
    let l = 6usize;
    let coord = BodyFittedCoordinate::<f64>::new(l, l, 1.0, 1.0, 0.0, TAU, trunc()).unwrap();

    // dims via the trait.
    assert_eq!(MetricProvider::dims(&coord), (l, l));

    // The trait's physical_gradient delegates to the inherent method (identical result).
    let u = coord.sample(|_xi, eta| (TAU * eta).sin()).unwrap();
    let (ax, ay) = MetricProvider::physical_gradient(&coord, &u).unwrap();
    let (bx, by) = coord.physical_gradient(&u).unwrap();
    let ax = dequantize_2d(&ax, l, l).unwrap();
    let bx = dequantize_2d(&bx, l, l).unwrap();
    let ay = dequantize_2d(&ay, l, l).unwrap();
    let by = dequantize_2d(&by, l, l).unwrap();
    for (p, q) in ax.as_slice().iter().zip(bx.as_slice()) {
        assert!((p - q).abs() < 1e-12);
    }
    for (p, q) in ay.as_slice().iter().zip(by.as_slice()) {
        assert!((p - q).abs() < 1e-12);
    }

    // Jacobian is reachable and non-empty through the trait.
    let jac = dequantize_2d(MetricProvider::jacobian(&coord), l, l).unwrap();
    assert!(jac.as_slice().iter().all(|v| *v > 0.0));
}

#[test]
fn generic_consumer_accepts_both_providers() {
    let l = 4usize;
    let cart = CartesianIdentity::<f64>::new(l, l, 1.0, 1.0, trunc()).unwrap();
    let fitted = BodyFittedCoordinate::<f64>::new(l, l, 1.0, 1.0, 0.0, TAU, trunc()).unwrap();
    let n = (1usize << l) * (1usize << l);
    assert_eq!(lattice_size(&cart), n);
    assert_eq!(lattice_size(&fitted), n);
}
