/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B Stage 6 gates: the 3-D compressible Euler marcher (`CompressibleMarcher3d`) — free-stream
//! preservation, the `Marcher` seam, and boundedness beyond the explicit acoustic-diffusion limit. Grids
//! are kept small (`l = 3`, 8³) because the 3-D ADI inverse is the heaviest per-step operator.

use deep_causality_cfd::{
    CompressibleMarcher3d, EulerState3d, EulerStateTt3d, Marcher, dequantize_3d, quantize_3d,
};
use deep_causality_physics::PhysicsErrorEnum;
use deep_causality_tensor::{CausalTensor, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const GAMMA: f64 = 1.4;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

#[test]
fn free_stream_is_a_fixed_point() {
    // A uniform state has zero flux divergence and a free-stream-exact implicit step, so it marches
    // unchanged in 3-D.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let (rho, u, v, w, p) = (1.2, 0.3, 0.1, 0.05, 0.8);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v + w * w);
    let state: EulerState3d<f64> = [
        vec![rho; n],
        vec![rho * u; n],
        vec![rho * v; n],
        vec![rho * w; n],
        vec![e; n],
    ];
    let marcher =
        CompressibleMarcher3d::<f64>::new((l, l, l), dx, GAMMA, 0.002, 1.3, tr()).unwrap();
    assert!((marcher.gamma() - GAMMA).abs() < 1e-15, "gamma getter");
    let (out, _) = marcher.run(&state, 4).unwrap();
    for &d in &out[0] {
        assert!((d - rho).abs() < 1e-9, "ρ drifted: {d}");
    }
    for &en in &out[4] {
        assert!((en - e).abs() < 1e-9, "ρE drifted: {en}");
    }
}

#[test]
fn marcher_trait_advance_matches_one_step_and_preserves_free_stream() {
    // Stage-6 / 5.1 analogue: `CompressibleMarcher3d` implements `Marcher`; `advance` is one IMEX step on
    // the tensor-train state, and a uniform state is a fixed point of it.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let q = |buf: Vec<f64>| {
        quantize_3d(
            &CausalTensor::new(buf, vec![1 << l, 1 << l, 1 << l]).unwrap(),
            &tr(),
        )
        .unwrap()
    };
    let (rho, u, v, w, p) = (1.0, 0.2, 0.1, 0.05, 0.7);
    let e = p / (GAMMA - 1.0) + 0.5 * rho * (u * u + v * v + w * w);
    let state: EulerStateTt3d<f64> = [
        q(vec![rho; n]),
        q(vec![rho * u; n]),
        q(vec![rho * v; n]),
        q(vec![rho * w; n]),
        q(vec![e; n]),
    ];
    let marcher =
        CompressibleMarcher3d::<f64>::new((l, l, l), dx, GAMMA, 0.001, 1.3, tr()).unwrap();
    let advanced = Marcher::advance(&marcher, &state, &()).unwrap();
    let stepped = marcher.step(&state).unwrap();
    let a0 = dequantize_3d(&advanced[0], l, l, l).unwrap();
    let s0 = dequantize_3d(&stepped[0], l, l, l).unwrap();
    for (&a, &s) in a0.as_slice().iter().zip(s0.as_slice()) {
        assert!((a - s).abs() < 1e-14, "advance must equal step");
    }
    for &d in a0.as_slice() {
        assert!(
            (d - rho).abs() < 1e-9,
            "advance must preserve free-stream ρ: {d}"
        );
    }
}

#[test]
fn imex_marches_a_smooth_field_stably_in_3d() {
    // A smooth 3-D structure marches stably (density positive and finite) past the explicit acoustic-
    // diffusion limit — the implicit dissipation via the closed-form 3-D inverse carries the stiffness.
    let l = 3usize;
    let side = 1usize << l;
    let n = side * side * side;
    let dx = 1.0 / side as f64;
    let mut state: EulerState3d<f64> = [
        vec![0.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![0.0; n],
    ];
    for ix in 0..side {
        for iy in 0..side {
            for iz in 0..side {
                let x = ix as f64 / side as f64;
                let y = iy as f64 / side as f64;
                let z = iz as f64 / side as f64;
                let bump = 0.08 * (TAU * x).sin() * (TAU * y).sin() * (TAU * z).sin();
                let idx = (ix * side + iy) * side + iz;
                state[0][idx] = 1.0 + bump;
                state[4][idx] = (1.0 + bump) / (GAMMA - 1.0); // p = 1 + bump, u = v = w = 0
            }
        }
    }
    // s = ½·c·Δt/Δx ≈ ½·1.18·Δt·8 ≈ 4.7·Δt; Δt = 0.06 ⇒ s ≈ 0.28 (> the explicit 3-D limit).
    let marcher = CompressibleMarcher3d::<f64>::new((l, l, l), dx, GAMMA, 0.06, 1.2, tr()).unwrap();
    let (out, _peak) = marcher.run(&state, 10).unwrap();
    assert!(
        out[0].iter().all(|&d| d > 0.0 && d.is_finite()),
        "density must stay positive and finite under the 3-D march"
    );
    assert!(
        out[4].iter().all(|&en| en.is_finite()),
        "energy must stay finite"
    );
}

#[test]
fn new_rejects_non_positive_reference_speed() {
    let dx = 1.0 / 8.0;
    assert!(CompressibleMarcher3d::<f64>::new((3, 3, 3), dx, GAMMA, 0.001, 0.0, tr()).is_err());
    assert!(CompressibleMarcher3d::<f64>::new((3, 3, 3), dx, GAMMA, 0.001, -1.0, tr()).is_err());
}

#[test]
fn run_rejects_wrong_length_state() {
    let dx = 1.0 / 8.0;
    let marcher =
        CompressibleMarcher3d::<f64>::new((3, 3, 3), dx, GAMMA, 0.001, 1.3, tr()).unwrap();
    let bad: EulerState3d<f64> = [
        vec![1.0; 7],
        vec![0.0; 7],
        vec![0.0; 7],
        vec![0.0; 7],
        vec![1.0; 7],
    ];
    assert!(marcher.run(&bad, 1).is_err());
}

#[test]
fn run_rejects_non_positive_density() {
    // The pointwise 3-D flux/EOS enforces positivity: a negative-density cell must surface a
    // `PhysicalInvariantBroken` on the first flux evaluation.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let marcher =
        CompressibleMarcher3d::<f64>::new((l, l, l), dx, GAMMA, 0.001, 1.3, tr()).unwrap();
    let mut rho = vec![1.0; n];
    rho[9] = -0.4; // non-physical density
    let state: EulerState3d<f64> = [rho, vec![0.0; n], vec![0.0; n], vec![0.0; n], vec![2.5; n]];
    let err = marcher.run(&state, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "non-positive density must break the positivity invariant: {err:?}"
    );
}

#[test]
fn non_positive_pressure_is_rejected() {
    // Item 12, 3-D: a cell with E < ½|m|²/ρ is refused at the shared pressure guard.
    let l = 3usize;
    let n = 1usize << (3 * l);
    let dx = 1.0 / (1usize << l) as f64;
    let marcher =
        CompressibleMarcher3d::<f64>::new((l, l, l), dx, GAMMA, 0.002, 1.3, tr()).unwrap();
    let mut state: EulerState3d<f64> = [
        vec![1.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![0.0; n],
        vec![2.5; n],
    ];
    state[1][n / 2] = 2.0;
    state[4][n / 2] = 1.0;
    let err = marcher.run(&state, 1).unwrap_err();
    assert!(
        matches!(err.0, PhysicsErrorEnum::PhysicalInvariantBroken(_)),
        "a non-hyperbolic cell must be refused in 3-D: {err:?}"
    );
}
