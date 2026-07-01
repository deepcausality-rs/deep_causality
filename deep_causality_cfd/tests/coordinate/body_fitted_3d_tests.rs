/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `BodyFittedCoordinate3d` (the spherical-shell fitted metric). The decisive check: the
//! physical gradient of a purely **radial** field equals the unit radial vector `r̂ = (sinθcosφ,
//! sinθsinφ, cosθ)` — i.e. the 3-D chain-rule inverse-Jacobian metric is correct — plus the Jacobian
//! volume element, `position`, the pole/parameter rejections, and static-dispatch usability.

use deep_causality_cfd::{BodyFittedCoordinate3d, MetricProvider3d, dequantize_3d};
use deep_causality_tensor::Truncation;

const PI: f64 = core::f64::consts::PI;
const TAU: f64 = core::f64::consts::TAU;

fn trunc() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

fn shell(l: usize) -> BodyFittedCoordinate3d<f64> {
    // r ∈ [1, 2], θ ∈ [0.3, 2.3] ⊂ (0, π), φ ∈ [0, 2π) (full revolution).
    BodyFittedCoordinate3d::<f64>::new(l, l, l, 1.0, 1.0, 0.3, 2.0, 0.0, TAU, trunc()).unwrap()
}

#[test]
fn radial_field_gradient_is_the_unit_radial_vector() {
    let l = 4usize;
    let n = 1usize << l;
    let coord = shell(l);
    // u(ξ, η, ζ) = r(ζ) = 1 + ζ·1 — a purely radial field; ∇u = r̂.
    let u = coord.sample(|_xi, _eta, zeta| 1.0 + zeta).unwrap();
    let (gx, gy, gz) = coord.physical_gradient(&u).unwrap();
    let (gx, gy, gz) = (
        dequantize_3d(&gx, l, l, l).unwrap(),
        dequantize_3d(&gy, l, l, l).unwrap(),
        dequantize_3d(&gz, l, l, l).unwrap(),
    );
    let idx = |i: usize, j: usize, k: usize| (i * n + j) * n + k;

    let mut max_err = 0.0f64;
    // Interior ζ (avoid the periodic radial boundary at k = 0, N-1); a spread of (ξ, η).
    for &i in &[0usize, n / 4, n / 2] {
        for &j in &[1usize, n / 3, n / 2, (2 * n) / 3] {
            for &k in &[1usize, n / 2, n - 2] {
                let phi = 0.0 + (i as f64 / n as f64) * TAU;
                let theta = 0.3 + (j as f64 / n as f64) * 2.0;
                let (rx, ry, rz) = (
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                );
                max_err = max_err
                    .max((gx.as_slice()[idx(i, j, k)] - rx).abs())
                    .max((gy.as_slice()[idx(i, j, k)] - ry).abs())
                    .max((gz.as_slice()[idx(i, j, k)] - rz).abs());
            }
        }
    }
    assert!(
        max_err < 1e-6,
        "physical gradient of r must be r̂; max err {max_err}"
    );
}

#[test]
fn jacobian_is_the_spherical_volume_element() {
    let l = 4usize;
    let n = 1usize << l;
    let coord = shell(l);
    let jac = dequantize_3d(coord.jacobian(), l, l, l).unwrap();
    let idx = |i: usize, j: usize, k: usize| (i * n + j) * n + k;
    // |J| = r²·sinθ·Δr·Δθ·Δφ, with Δr = 1, Δθ = 2, Δφ = 2π.
    let mut max_err = 0.0f64;
    for &i in &[0usize, n / 2] {
        for &j in &[0usize, n / 2, n - 1] {
            for &k in &[0usize, n / 2, n - 1] {
                let theta = 0.3 + (j as f64 / n as f64) * 2.0;
                let r = 1.0 + (k as f64 / n as f64) * 1.0;
                let expected = r * r * theta.sin() * 1.0 * 2.0 * TAU;
                max_err = max_err.max((jac.as_slice()[idx(i, j, k)] - expected).abs());
            }
        }
    }
    assert!(max_err < 1e-6, "Jacobian volume element error {max_err}");
}

#[test]
fn position_has_radius_r() {
    let coord = shell(4);
    for &(xi, eta, zeta) in &[(0.0, 0.0, 0.0), (0.25, 0.5, 0.5), (0.75, 0.9, 1.0)] {
        let (x, y, z) = coord.position(xi, eta, zeta);
        let r = (x * x + y * y + z * z).sqrt();
        let expected = 1.0 + zeta * 1.0;
        assert!(
            (r - expected).abs() < 1e-12,
            "|position| = r: {r} vs {expected}"
        );
    }
}

#[test]
fn rejects_poles_and_nonpositive_parameters() {
    let t = trunc();
    // Polar range touching a pole (θ0 = 0).
    assert!(BodyFittedCoordinate3d::<f64>::new(3, 3, 3, 1.0, 1.0, 0.0, 1.0, 0.0, TAU, t).is_err());
    // Polar range crossing π.
    assert!(BodyFittedCoordinate3d::<f64>::new(3, 3, 3, 1.0, 1.0, 0.3, PI, 0.0, TAU, t).is_err());
    // Non-positive radius / spans.
    assert!(BodyFittedCoordinate3d::<f64>::new(3, 3, 3, 0.0, 1.0, 0.3, 1.0, 0.0, TAU, t).is_err());
    assert!(BodyFittedCoordinate3d::<f64>::new(3, 3, 3, 1.0, -1.0, 0.3, 1.0, 0.0, TAU, t).is_err());
}

#[test]
fn usable_by_static_dispatch() {
    fn dims_of<M: MetricProvider3d<f64>>(m: &M) -> (usize, usize, usize) {
        m.dims()
    }
    let coord = shell(3);
    assert_eq!(dims_of(&coord), (3, 3, 3));
}
