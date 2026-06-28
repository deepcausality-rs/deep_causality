/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{body_mask_2d, dequantize_2d};
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
