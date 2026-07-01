/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 1: the body-fitted polar coordinate — chain-rule physical gradient to scheme order,
//! free-stream preservation, low-rank Jacobian, and the rank-lever gate (fitted `χ ~ O(10)`
//! resolution-independent vs Cartesian `χ ~ √side`).

use deep_causality_cfd::{BodyFittedCoordinate, dequantize_2d, quantize_2d};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;

fn full() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

fn smoothed_step(d: f64, w: f64) -> f64 {
    0.5 * (1.0 + (d / w).tanh())
}

#[test]
fn gradient_recovers_physical_derivative() {
    // Full annulus (ξ periodic), radius 0.6 → 2.2. u = x (physical): ∂x/∂x = 1, ∂x/∂y = 0.
    let (lx, ly) = (5usize, 5usize);
    let (r0, dr, th0, dth) = (0.6f64, 1.6, 0.0, TAU);
    let coord = BodyFittedCoordinate::<f64>::new(lx, ly, r0, dr, th0, dth, full()).unwrap();
    let u = coord
        .sample(|xi, eta| (r0 + eta * dr) * (th0 + xi * dth).cos())
        .unwrap();
    let (dudx, dudy) = coord.physical_gradient(&u).unwrap();
    let gx = dequantize_2d(&dudx, lx, ly).unwrap();
    let gy = dequantize_2d(&dudy, lx, ly).unwrap();
    let (nx, ny) = (1usize << lx, 1usize << ly);
    // Interior radial rows (the periodic operator wraps at the η boundary — a Stage-2 refinement).
    for i in 0..nx {
        for j in 1..ny - 1 {
            let dx = gx.as_slice()[i * ny + j];
            let dy = gy.as_slice()[i * ny + j];
            assert!((dx - 1.0).abs() < 0.02, "∂x/∂x at {i},{j} = {dx}");
            assert!(dy.abs() < 0.02, "∂x/∂y at {i},{j} = {dy}");
        }
    }
}

#[test]
fn gradient_converges_second_order() {
    // The interior error of the chain-rule gradient drops ~4× per refinement (2nd order).
    let err = |l: usize| {
        let (r0, dr) = (0.6f64, 1.6);
        let coord = BodyFittedCoordinate::<f64>::new(l, l, r0, dr, 0.0, TAU, full()).unwrap();
        let u = coord
            .sample(|xi, eta| (r0 + eta * dr) * (xi * TAU).cos())
            .unwrap();
        let (dudx, _) = coord.physical_gradient(&u).unwrap();
        let gx = dequantize_2d(&dudx, l, l).unwrap();
        let (nx, ny) = (1usize << l, 1usize << l);
        let mut m = 0.0f64;
        for i in 0..nx {
            for j in 1..ny - 1 {
                m = m.max((gx.as_slice()[i * ny + j] - 1.0).abs());
            }
        }
        m
    };
    let (e5, e6) = (err(5), err(6));
    assert!(e6 < e5 * 0.4, "not ~2nd order: e5={e5}, e6={e6}");
}

#[test]
fn free_stream_preserved() {
    // A uniform field has exactly zero physical gradient (the metric identity holds discretely).
    let (lx, ly) = (5usize, 5usize);
    let coord = BodyFittedCoordinate::<f64>::new(lx, ly, 0.6, 1.6, 0.0, TAU, full()).unwrap();
    let u = coord.sample(|_xi, _eta| 3.7).unwrap();
    let (dudx, dudy) = coord.physical_gradient(&u).unwrap();
    let gx = dequantize_2d(&dudx, lx, ly).unwrap();
    let gy = dequantize_2d(&dudy, lx, ly).unwrap();
    for v in gx.as_slice().iter().chain(gy.as_slice()) {
        assert!(v.abs() < 1e-9, "free-stream gradient nonzero: {v}");
    }
}

#[test]
fn position_maps_computational_to_physical_polar_point() {
    // (x, y) = (r·cosθ, r·sinθ) with θ = θ0 + ξ·Δθ, r = r0 + η·Δr; |position| = r exactly.
    let (r0, dr, th0, dth) = (0.6f64, 1.6, 0.2, TAU);
    let coord = BodyFittedCoordinate::<f64>::new(5, 5, r0, dr, th0, dth, full()).unwrap();
    for &(xi, eta) in &[(0.0, 0.0), (0.25, 0.5), (0.5, 1.0), (0.75, 0.3)] {
        let (x, y) = coord.position(xi, eta);
        let theta = th0 + xi * dth;
        let r = r0 + eta * dr;
        assert!((x - r * theta.cos()).abs() < 1e-12, "x at {xi},{eta}");
        assert!((y - r * theta.sin()).abs() < 1e-12, "y at {xi},{eta}");
        // Radius invariant: |position| = r regardless of the angle.
        assert!(((x * x + y * y).sqrt() - r).abs() < 1e-12, "|position| = r");
    }
}

#[test]
fn rejects_nonpositive_geometry() {
    let t = full();
    let bad = |r0, dr, dth| BodyFittedCoordinate::<f64>::new(4, 4, r0, dr, 0.0, dth, t);
    // r0 ≤ 0. (BodyFittedCoordinate is not Debug, so match the error without unwrap_err.)
    match bad(0.0, 1.6, TAU) {
        Err(e) => assert!(
            matches!(e.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
            "r0 = 0 must be PhysicalInvariantBroken"
        ),
        Ok(_) => panic!("r0 = 0 must be rejected"),
    }
    // dr ≤ 0.
    assert!(bad(0.6, 0.0, TAU).is_err(), "dr = 0");
    assert!(bad(0.6, -1.0, TAU).is_err(), "dr < 0");
    // dtheta ≤ 0.
    assert!(bad(0.6, 1.6, 0.0).is_err(), "dtheta = 0");
    assert!(bad(0.6, 1.6, -1.0).is_err(), "dtheta < 0");
}

#[test]
fn jacobian_is_low_rank() {
    let coord = BodyFittedCoordinate::<f64>::new(6, 6, 0.6, 1.6, 0.0, TAU, full()).unwrap();
    let bond = coord.jacobian().max_bond();
    assert!(bond <= 4, "Jacobian should be low-rank, got bond {bond}");
}

#[test]
fn rank_lever_fitted_bounded_cartesian_grows() {
    let (r_shock, w) = (1.4f64, 0.08f64);

    // Fitted: the curved shock is a constant-radius surface → a step in η, constant in ξ → low-rank.
    let fitted_bond = |l: usize| -> usize {
        let (r0, dr) = (0.6f64, 1.6);
        let coord = BodyFittedCoordinate::<f64>::new(l, l, r0, dr, 0.0, TAU, full()).unwrap();
        coord
            .sample(|_xi, eta| smoothed_step((r0 + eta * dr) - r_shock, w))
            .unwrap()
            .max_bond()
    };

    // Cartesian: the same curved shock captured on an [-2,2]² grid → rank grows with resolution.
    let cart_bond = |l: usize| -> usize {
        let n = 1usize << l;
        let mut data = vec![0.0f64; n * n];
        for i in 0..n {
            for j in 0..n {
                let x = -2.0 + 4.0 * (i as f64) / (n as f64);
                let y = -2.0 + 4.0 * (j as f64) / (n as f64);
                data[i * n + j] = smoothed_step((x * x + y * y).sqrt() - r_shock, w);
            }
        }
        let field = CausalTensor::new(data, vec![n, n]).unwrap();
        quantize_2d(&field, &full()).unwrap().max_bond()
    };

    let (f4, f7) = (fitted_bond(4), fitted_bond(7));
    let (c4, c7) = (cart_bond(4), cart_bond(7));

    // Fitted bond stays O(10) and bounded across the resolution sweep (it does not grow like √side).
    assert!(f4 <= 12 && f7 <= 12, "fitted bond not O(10): {f4} -> {f7}");
    // Cartesian capture grows with resolution...
    assert!(c7 > c4, "cartesian bond did not grow: {c4} -> {c7}");
    // ...and overtakes the fitted bond at high resolution — the measured rank lever.
    assert!(
        c7 > f7,
        "no rank lever: cartesian {c7} should exceed fitted {f7}"
    );
}
