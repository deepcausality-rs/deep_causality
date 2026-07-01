/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the 3-D `MetricProvider3d` seam: `CartesianIdentity3d` correctness (physical gradient vs
//! analytic, Jacobian = cell volume) and a generic `over M: MetricProvider3d` consumer (static dispatch).

use deep_causality_cfd::{CartesianIdentity3d, MetricProvider3d, dequantize_3d};
use deep_causality_tensor::Truncation;

const TAU: f64 = core::f64::consts::TAU;

fn trunc() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

/// A generic consumer: proves the provider satisfies the bound and is usable by static dispatch.
fn lattice_size<M: MetricProvider3d<f64>>(m: &M) -> usize {
    let (lx, ly, lz) = m.dims();
    (1usize << lx) * (1usize << ly) * (1usize << lz)
}

#[test]
fn cartesian_identity_3d_physical_gradient_matches_analytic() {
    // Identity chart on [0,1)³: x = ξ, so u = sin(2πξ) = sin(2πx); ∂u/∂x = 2π cos(2πx), ∂u/∂y = ∂u/∂z = 0.
    let l = 5usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let m = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, trunc()).unwrap();
    let u = m.sample(|xi, _eta, _zeta| (TAU * xi).sin()).unwrap();

    let (dudx, dudy, dudz) = m.physical_gradient(&u).unwrap();
    let gx = dequantize_3d(&dudx, l, l, l).unwrap();
    let gy = dequantize_3d(&dudy, l, l, l).unwrap();
    let gz = dequantize_3d(&dudz, l, l, l).unwrap();
    let (gxs, gys, gzs) = (gx.as_slice(), gy.as_slice(), gz.as_slice());

    // Row-major [nx, ny, nz]: index (i*ny + j)*nz + k; the (i, 0, 0) column carries the x-gradient.
    let idx = |i: usize, j: usize, k: usize| (i * n + j) * n + k;
    let mut max_err_x = 0.0f64;
    let mut max_abs_yz = 0.0f64;
    for i in 0..n {
        let x = i as f64 / n as f64;
        let exact = TAU * (TAU * x).cos();
        max_err_x = max_err_x.max((gxs[idx(i, 0, 0)] - exact).abs());
        max_abs_yz = max_abs_yz
            .max(gys[idx(i, 0, 0)].abs())
            .max(gzs[idx(i, 0, 0)].abs());
    }
    // 2nd-order central difference: error ~ (2π)³/6·dx² ≈ 0.04 at N=32.
    assert!(max_err_x < 0.1, "∂/∂x error too large: {max_err_x}");
    assert!(
        max_abs_yz < 1e-9,
        "∂/∂y and ∂/∂z of an x-only field should vanish: {max_abs_yz}"
    );
}

#[test]
fn cartesian_identity_3d_gradient_picks_each_axis() {
    // A separable field varying only in z: u = sin(2πζ); ∂u/∂z ≠ 0, ∂u/∂x = ∂u/∂y = 0.
    let l = 5usize;
    let n = 1usize << l;
    let dx = 1.0 / n as f64;
    let m = CartesianIdentity3d::<f64>::new(l, l, l, dx, dx, dx, trunc()).unwrap();
    let u = m.sample(|_xi, _eta, zeta| (TAU * zeta).sin()).unwrap();

    let (dudx, dudy, dudz) = m.physical_gradient(&u).unwrap();
    let gx = dequantize_3d(&dudx, l, l, l).unwrap();
    let gy = dequantize_3d(&dudy, l, l, l).unwrap();
    let gz = dequantize_3d(&dudz, l, l, l).unwrap();
    let idx = |i: usize, j: usize, k: usize| (i * n + j) * n + k;

    let mut max_err_z = 0.0f64;
    let mut max_abs_xy = 0.0f64;
    for k in 0..n {
        let z = k as f64 / n as f64;
        let exact = TAU * (TAU * z).cos();
        max_err_z = max_err_z.max((gz.as_slice()[idx(0, 0, k)] - exact).abs());
        max_abs_xy = max_abs_xy
            .max(gx.as_slice()[idx(0, 0, k)].abs())
            .max(gy.as_slice()[idx(0, 0, k)].abs());
    }
    assert!(max_err_z < 0.1, "∂/∂z error too large: {max_err_z}");
    assert!(
        max_abs_xy < 1e-9,
        "∂/∂x and ∂/∂y should vanish: {max_abs_xy}"
    );
}

#[test]
fn cartesian_identity_3d_jacobian_is_cell_volume() {
    let l = 4usize;
    let n = 1usize << l;
    let (dx, dy, dz) = (1.0 / n as f64, 2.0 / n as f64, 0.5 / n as f64);
    let m = CartesianIdentity3d::<f64>::new(l, l, l, dx, dy, dz, trunc()).unwrap();
    let jac = dequantize_3d(m.jacobian(), l, l, l).unwrap();
    let cell = dx * dy * dz;
    for v in jac.as_slice() {
        assert!(
            (v - cell).abs() < 1e-14,
            "jacobian should be the constant cell volume {cell}, got {v}"
        );
    }
}

#[test]
fn generic_consumer_accepts_the_3d_provider() {
    let l = 3usize;
    let cart = CartesianIdentity3d::<f64>::new(l, l, l, 1.0, 1.0, 1.0, trunc()).unwrap();
    let n = (1usize << l) * (1usize << l) * (1usize << l);
    assert_eq!(lattice_size(&cart), n);
    assert_eq!(MetricProvider3d::dims(&cart), (l, l, l));
}
