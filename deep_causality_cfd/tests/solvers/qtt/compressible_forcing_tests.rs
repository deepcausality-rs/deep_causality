/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    EulerStateTt2d, ForcingRegion, dequantize_2d, plume_mask_2d, quantize_2d,
};
use deep_causality_tensor::{CausalTensor, Truncation};

const L: usize = 5; // 32 x 32
const GAMMA: f64 = 1.4;

fn trunc() -> Truncation<f64> {
    Truncation::<f64>::by_tol(1e-10).unwrap()
}

/// A uniform conserved state `[ρ, ρu, ρv, ρE]` on the 2^L × 2^L grid, as tensor trains.
fn uniform_state(rho: f64, u: f64, p: f64) -> EulerStateTt2d<f64> {
    let n = 1usize << L;
    let e = p / (GAMMA - 1.0) + 0.5 * rho * u * u;
    let field = |v: f64| {
        quantize_2d(
            &CausalTensor::new(vec![v; n * n], vec![n, n]).unwrap(),
            &trunc(),
        )
        .unwrap()
    };
    [field(rho), field(rho * u), field(0.0), field(e)]
}

/// A plume-shaped mask centered at (0.4, 0.5) on the unit square (a sharp half-cell skirt, so
/// the interior sits within ~1e-4 of χ = 1 and the blend assertions can be tight).
fn plume_mask() -> deep_causality_tensor::CausalTensorTrain<f64> {
    let n = 1usize << L;
    let dx = 1.0 / n as f64;
    plume_mask_2d::<f64>(L, L, dx, dx, 0.4, 0.5, 0.15, 0.09, 0.5 * dx, &trunc()).unwrap()
}

#[test]
fn constructor_rejects_bad_inputs() {
    let mask = plume_mask();
    let target = [1.0, -2.0, 0.0, 5.0];
    // eta must be finite and positive.
    assert!(ForcingRegion::new(mask.clone(), target, 0.0).is_err());
    assert!(ForcingRegion::new(mask.clone(), target, -1.0).is_err());
    assert!(ForcingRegion::new(mask.clone(), target, f64::NAN).is_err());
    // Every target component must be finite.
    assert!(ForcingRegion::new(mask.clone(), [1.0, f64::INFINITY, 0.0, 5.0], 0.01).is_err());
    // The target density must be positive (the marcher rejects a non-positive density).
    assert!(ForcingRegion::new(mask.clone(), [0.0, 0.0, 0.0, 5.0], 0.01).is_err());
    // A valid region reports its pieces.
    let region = ForcingRegion::new(mask, target, 0.01).unwrap();
    assert_eq!(region.target(), target);
    assert_eq!(region.eta(), 0.01);
}

#[test]
fn hard_forcing_drives_the_interior_to_the_target_and_leaves_the_exterior() {
    // η = dt → w = 1: one application pins the masked interior at the target while the exterior
    // (χ → 0) stays at the freestream — the "interior driven, exterior free" contract.
    let n = 1usize << L;
    let dt = 1.0e-3;
    let freestream = uniform_state(1.0, 2.0, 1.0 / GAMMA);
    let target = [0.3, -0.9, 0.0, 2.5];
    let region = ForcingRegion::new(plume_mask(), target, dt).unwrap();

    let forced = region.apply(&freestream, dt, &trunc()).unwrap();

    let dx = 1.0 / n as f64;
    let at = |train: &deep_causality_tensor::CausalTensorTrain<f64>, x: f64, y: f64| {
        let d = dequantize_2d(train, L, L).unwrap();
        d.as_slice()[(x / dx).round() as usize * n + (y / dx).round() as usize]
    };
    // Interior (mask center): every component at its target, to the smoothed skirt's residual.
    for (k, &t) in target.iter().enumerate() {
        let got = at(&forced[k], 0.4, 0.5);
        assert!(
            (got - t).abs() < 1e-3,
            "component {k} interior: got {got}, target {t}"
        );
    }
    // Exterior (far corner): every component still the freestream.
    let e_inf = (1.0 / GAMMA) / (GAMMA - 1.0) + 0.5 * 4.0;
    for (k, &f) in [1.0, 2.0, 0.0, e_inf].iter().enumerate() {
        let got = at(&forced[k], 0.95, 0.95);
        assert!(
            (got - f).abs() < 1e-3,
            "component {k} exterior: got {got}, freestream {f}"
        );
    }
}

#[test]
fn soft_forcing_blends_by_the_penalization_weight() {
    // η = 4·dt → w = 0.25: one application moves the interior a quarter of the way.
    let dt = 1.0e-3;
    let freestream = uniform_state(1.0, 0.0, 1.0);
    let target = [2.0, 0.0, 0.0, 4.0];
    let region = ForcingRegion::new(plume_mask(), target, 4.0 * dt).unwrap();

    let forced = region.apply(&freestream, dt, &trunc()).unwrap();
    let n = 1usize << L;
    let rho = dequantize_2d(&forced[0], L, L).unwrap();
    let center = rho.as_slice()[(0.4 * n as f64).round() as usize * n + n / 2];
    // ρ: 1.0 + 0.25·(2.0 − 1.0) = 1.25 at χ ≈ 1 (to the smoothed skirt's residual).
    assert!(
        (center - 1.25).abs() < 1e-3,
        "quarter blend at the center: {center}"
    );
}

#[test]
fn forced_state_stays_under_a_bond_cap() {
    // Under a by-bond round policy the forced state respects the cap — the imprint cannot
    // silently blow the compression contract.
    let cap = 8usize;
    let capped = Truncation::<f64>::by_bond(cap).unwrap();
    let dt = 1.0e-3;
    let freestream = uniform_state(1.0, 2.0, 1.0 / GAMMA);
    let region = ForcingRegion::new(plume_mask(), [0.5, -0.4, 0.0, 3.0], dt).unwrap();

    let forced = region.apply(&freestream, dt, &capped).unwrap();
    for (k, train) in forced.iter().enumerate() {
        let bond = train.cores().iter().map(|c| c.shape()[2]).max().unwrap();
        assert!(bond <= cap, "component {k} bond {bond} exceeds cap {cap}");
    }
}
