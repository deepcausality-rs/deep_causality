/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{body_mask_2d, dequantize_2d, plume_mask_2d, quantize_2d};
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

// --- The [0, 1] mask invariant (close-qtt-solver-envelope, item 14) -------------------------------
//
// The analytic smoothed indicator is in [0, 1], but tensor-train rounding drives the quantized mask
// out of range — a negative χ inverts the penalization forcing sign. `mask_from_fn` clamps after
// quantization and rejects a gross excursion. A lossy train cannot hold pointwise [0, 1] exactly, so
// the guarantee is: the bulk excursion is removed, the residual is bounded truncation noise, and a
// mask wrong by more than a fraction of the range is refused.
//
// These tests run on this file's 32² grid (const L = 5), where the truncation excursion is small and
// bond 16+ is exactly non-negative. That is NOT the shipped L=8 (256²) cylinder ladder, whose η sweep
// runs at bond cap 48 with min χ ≈ −7e-7 (non-negative only to truncation tolerance) — the excursion
// grows with grid resolution at a fixed cap. So these assert the L=5 property; the residual-at-the-
// operating-cap behaviour on the shipped grid is documented on `clamp_mask_to_unit_interval` itself.

#[test]
fn the_clamp_strictly_reduces_the_coarse_cap_excursion() {
    // This must bite on the clamp, not pass on the raw mask. So it compares the clamped mask
    // (body_mask_2d, which clamps) against the RAW quantized mask (quantize_2d of the same analytic
    // field, no clamp) at bond 4, where the raw excursion is min χ = −1.78e-3 across 188 cells.
    // The clamp must make the residual STRICTLY less negative (measured −1.78e-3 → −1.21e-3) — an
    // assertion that fails the moment the clamp is removed, unlike a bare `min > −0.05` bound that
    // the raw mask already satisfies.
    let n = 1usize << L;
    let dx = TAU / n as f64;
    let (cx, cy, r) = (TAU * 0.5, TAU * 0.5, TAU * 0.15);
    let tr = Truncation::<f64>::by_bond(4).unwrap();
    let smoothing = 2.0 * dx;

    // Raw: quantize the analytic smoothed indicator directly, bypassing the clamp.
    let half = 0.5f64;
    let mut field = vec![0.0f64; n * n];
    for i in 0..n {
        let x = i as f64 * dx;
        for j in 0..n {
            let y = j as f64 * dx;
            let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r;
            field[i * n + j] = half * (1.0 - (dist / smoothing).tanh());
        }
    }
    let raw = quantize_2d(
        &deep_causality_tensor::CausalTensor::new(field, vec![n, n]).unwrap(),
        &tr,
    )
    .unwrap();
    let raw_min = dequantize_2d(&raw, L, L)
        .unwrap()
        .as_slice()
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    // Clamped: body_mask_2d routes through clamp_mask_to_unit_interval.
    let clamped_min = dequantize_2d(
        &body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, smoothing, &tr).unwrap(),
        L,
        L,
    )
    .unwrap()
    .as_slice()
    .iter()
    .cloned()
    .fold(f64::INFINITY, f64::min);

    assert!(
        raw_min < -1e-4,
        "the raw mask must be meaningfully negative here, got {raw_min}"
    );
    assert!(
        clamped_min > raw_min + 1e-6,
        "the clamp must strictly reduce the excursion: raw {raw_min}, clamped {clamped_min}"
    );
    assert!(
        clamped_min > -0.05,
        "and the residual stays inside the truncation-noise band"
    );
}

#[test]
fn on_the_32_grid_the_mask_is_non_negative_at_bond_16_and_up() {
    // On this file's 32² grid the truncation excursion is small enough that bond 16 and 24 give an
    // exactly non-negative mask. (On the shipped 256² ladder the same caps are under-resolved and the
    // operating cap 48 still carries a −7e-7 residual — see the clamp fn's doc; this is the L=5 case.)
    let n = 1usize << L;
    let dx = TAU / n as f64;
    let (cx, cy, r) = (TAU * 0.5, TAU * 0.5, TAU * 0.15);
    for cap in [16usize, 24] {
        let tr = Truncation::<f64>::by_bond(cap).unwrap();
        let m = body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, 2.0 * dx, &tr).unwrap();
        let dense = dequantize_2d(&m, L, L).unwrap();
        let min = dense
            .as_slice()
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        assert!(
            min >= 0.0,
            "at bond {cap} the mask must be non-negative, got min χ {min}"
        );
    }
}

#[test]
fn a_grossly_out_of_range_mask_is_rejected() {
    // The rejection route: a mask fn returning a value far outside [0, 1] is a modelling error, not
    // rounding noise, and must fail construction rather than be silently clamped.
    let tr = Truncation::<f64>::by_bond(8).unwrap();
    let too_negative =
        deep_causality_cfd::mask_from_fn::<f64, _>(4, 4, 1.0, 1.0, |_x, _y| -0.5, &tr);
    assert!(too_negative.is_err(), "a χ = −0.5 mask must be refused");
    let too_large = deep_causality_cfd::mask_from_fn::<f64, _>(4, 4, 1.0, 1.0, |_x, _y| 1.6, &tr);
    assert!(too_large.is_err(), "a χ = 1.6 mask must be refused");
    // A valid in-range mask still constructs.
    let ok = deep_causality_cfd::mask_from_fn::<f64, _>(4, 4, 1.0, 1.0, |_x, _y| 0.5, &tr);
    assert!(ok.is_ok(), "an in-range mask must construct");
}
