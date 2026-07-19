/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{body_mask_2d, dequantize_2d, plume_mask_2d};
use deep_causality_tensor::Truncation;

const TAU: f64 = core::f64::consts::TAU;
const L: usize = 5; // 32 x 32

fn max_bond(train: &deep_causality_tensor::CausalTensorTrain<f64>) -> usize {
    train
        .cores()
        .iter()
        .map(|c| c.shape()[2])
        .max()
        .unwrap_or(1)
}

#[test]
fn cylinder_mask_round_trips_and_is_bounded_rank() {
    let n = 1usize << L;
    let dx = TAU / n as f64;
    let (cx, cy, r) = (TAU * 0.5, TAU * 0.5, TAU * 0.15);
    let smoothing = 2.0 * dx;
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();

    let mask = body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, smoothing, &trunc).unwrap();

    // Bounded rank: far below the dense element count (1024).
    let bond = max_bond(&mask);
    assert!(bond < n, "mask bond {bond} should be well below {n}");

    // Round-trips to the smoothed volume fraction: ~1 at the center, ~0 at a far corner, in [0,1].
    let dense = dequantize_2d(&mask, L, L).unwrap();
    let v = dense.as_slice();
    for &x in v {
        assert!(
            (-1e-6..=1.0 + 1e-6).contains(&x),
            "mask value {x} out of [0,1]"
        );
    }
    let center = v[(n / 2) * n + n / 2];
    let corner = v[0];
    assert!(
        center > 0.99,
        "center should be inside the body, got {center}"
    );
    assert!(
        corner < 0.01,
        "corner should be outside the body, got {corner}"
    );
}

#[test]
fn sharper_mask_costs_more_rank() {
    // The rank/accuracy trade-off, in its resolution-robust form: at a *fixed bond cap*, a sharper
    // (smaller δ) mask loses more accuracy than a smoother one — i.e. a sharper body needs more rank
    // to represent at the same fidelity. (Comparing bonds at a fixed tolerance is noisy once both
    // saturate the grid's rank ceiling; the fixed-bond error is the honest, literature-standard form.)
    let n = 1usize << L;
    let dx = TAU / n as f64;
    let (cx, cy, r) = (TAU * 0.5, TAU * 0.5, TAU * 0.15);
    let accurate = Truncation::<f64>::by_tol(1e-12).unwrap();
    let capped = Truncation::<f64>::by_bond(6).unwrap();

    let truncation_error = |smoothing: f64| -> f64 {
        let truth = body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, smoothing, &accurate).unwrap();
        let lowrank = body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, smoothing, &capped).unwrap();
        let t = dequantize_2d(&truth, L, L).unwrap();
        let c = dequantize_2d(&lowrank, L, L).unwrap();
        t.as_slice()
            .iter()
            .zip(c.as_slice())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f64, f64::max)
    };

    let err_smooth = truncation_error(4.0 * dx);
    let err_sharp = truncation_error(0.5 * dx);
    assert!(
        err_sharp > err_smooth,
        "sharper mask should lose more at a fixed bond cap: sharp {err_sharp:.3e} vs smooth {err_smooth:.3e}"
    );
}

#[test]
fn plume_mask_round_trips_and_is_bounded_rank() {
    let n = 1usize << L;
    let dx = 1.0 / n as f64;
    // A retro-plume-shaped ellipse: long along x (the jet axis), narrower across.
    let (cx, cy) = (0.4, 0.5);
    let (half_length, max_radius) = (0.18, 0.10);
    let smoothing = 0.5 * dx;
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();

    let mask = plume_mask_2d::<f64>(
        L,
        L,
        dx,
        dx,
        cx,
        cy,
        half_length,
        max_radius,
        smoothing,
        &trunc,
    )
    .unwrap();

    // Bounded rank: far below the dense element count.
    let bond = max_bond(&mask);
    assert!(bond < n, "plume mask bond {bond} should be well below {n}");

    // Round-trips to the smoothed volume fraction, in [0, 1]: ~1 at the ellipse center, ~0 far
    // outside, and anisotropic — a point inside along the major axis but outside the minor one.
    let dense = dequantize_2d(&mask, L, L).unwrap();
    let v = dense.as_slice();
    for &x in v {
        assert!(
            (-1e-6..=1.0 + 1e-6).contains(&x),
            "mask value {x} out of [0,1]"
        );
    }
    let at = |x: f64, y: f64| v[(x / dx).round() as usize * n + (y / dx).round() as usize];
    assert!(at(cx, cy) > 0.99, "center inside: {}", at(cx, cy));
    assert!(at(0.95, 0.95) < 0.01, "far corner outside");
    // (cx + 0.075, cy) is well inside the 0.18 semi-axis; (cx, cy + 0.15) is outside the 0.10 one.
    assert!(at(cx + 0.075, cy) > 0.9, "major axis point inside");
    assert!(at(cx, cy + 0.15) < 0.1, "minor axis point outside");
}

#[test]
fn different_plume_geometries_produce_different_masks() {
    // Two thrust settings → two analytic plume geometries → measurably different masks (the
    // imprint follows the commanded throttle through the geometry, before any physics).
    let n = 1usize << L;
    let dx = 1.0 / n as f64;
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();
    let low = plume_mask_2d::<f64>(L, L, dx, dx, 0.4, 0.5, 0.10, 0.06, 1.5 * dx, &trunc).unwrap();
    let high = plume_mask_2d::<f64>(L, L, dx, dx, 0.4, 0.5, 0.22, 0.12, 1.5 * dx, &trunc).unwrap();
    let a = dequantize_2d(&low, L, L).unwrap();
    let b = dequantize_2d(&high, L, L).unwrap();
    let diff: f64 = a
        .as_slice()
        .iter()
        .zip(b.as_slice())
        .map(|(x, y)| (x - y).abs())
        .sum();
    assert!(diff > 1.0, "geometries should differ, L1 diff {diff}");
}
