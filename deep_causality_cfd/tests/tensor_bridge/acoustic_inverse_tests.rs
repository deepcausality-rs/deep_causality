/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Resolution-6 / D10 gates for the closed-form constant-coefficient acoustic-core inverse
//! (`AcousticCoreInverse`): the inverse satisfies `A₀·A₀⁻¹ = I` to round-off at bounded, resolution-stable
//! bond, with no iterative solve, and is **free-stream-exact** (`A₀⁻¹·const = const`) — the property an
//! AMEn-per-step solve loses to its residual tolerance.

use deep_causality_cfd::{
    AcousticCoreInverse, AcousticCoreInverse2d, AcousticCoreInverse3d, dequantize, dequantize_2d,
    dequantize_3d, laplacian_2d, laplacian_3d, quantize, quantize_2d, quantize_3d, shift_minus,
    shift_plus,
};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainOperator, TensorTrainOperator, Truncation,
};

const TAU: f64 = core::f64::consts::TAU;
/// Acoustic stiffness `s = β/Δx²` (the study's value; `> 1` ⇒ past the explicit acoustic CFL).
const S: f64 = 8.0;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-12).unwrap()
}

/// `A₀ = (1+2s)·I − s·(S₊ + S₋)`, the constant-coefficient acoustic core as an MPO.
fn build_a0(l: usize, s: f64) -> CausalTensorTrainOperator<f64> {
    let id = CausalTensorTrainOperator::<f64>::identity(&vec![2usize; l]);
    let off = shift_plus::<f64>(l)
        .unwrap()
        .add(&shift_minus::<f64>(l).unwrap())
        .unwrap()
        .scale(-s);
    id.scale(1.0 + 2.0 * s).add(&off).unwrap()
}

/// Solve `A₀ x = b` for a smooth `b` via the closed-form inverse; return `(‖A₀x − b‖/‖b‖, x.max_bond)`.
fn inverse_residual(l: usize, s: f64) -> (f64, usize) {
    let n = 1usize << l;
    let b_dense: Vec<f64> = (0..n)
        .map(|i| {
            let x = i as f64 / n as f64;
            (TAU * x).sin() + 0.5 * (2.0 * TAU * x).cos()
        })
        .collect();
    let b = quantize(&CausalTensor::new(b_dense.clone(), vec![n]).unwrap(), &tr()).unwrap();

    let inv = AcousticCoreInverse::new_1d(l, s, tr()).unwrap();
    let x = inv.apply(&b).unwrap();

    let ax = build_a0(l, s).apply(&x, &tr()).unwrap();
    let ax_dense = dequantize(&ax).unwrap();
    let num: f64 = ax_dense
        .as_slice()
        .iter()
        .zip(&b_dense)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f64>()
        .sqrt();
    let den: f64 = b_dense.iter().map(|v| v * v).sum::<f64>().sqrt();
    (num / (den + 1e-300), x.max_bond())
}

#[test]
fn closed_form_inverse_solves_to_roundoff_and_is_resolution_stable() {
    // Gate 1: A₀ A₀⁻¹ = I to round-off, at bounded bond, flat across resolution (L=8 vs L=10) — the
    // standard constant-coefficient QTT-inverse result, here built in closed form (no AMEn).
    let (res8, bond8) = inverse_residual(8, S);
    let (res10, bond10) = inverse_residual(10, S);
    assert!(res8 < 1e-9, "L=8 residual {res8:.2e} not at round-off");
    assert!(res10 < 1e-9, "L=10 residual {res10:.2e} not at round-off");
    assert!(bond8 <= 16, "L=8 inverse bond {bond8} not low-rank");
    assert!(bond10 <= 16, "L=10 inverse bond {bond10} not low-rank");
    assert!(
        bond10 <= bond8 + 4,
        "inverse bond not resolution-stable (L8={bond8}, L10={bond10})"
    );
}

#[test]
fn inverse_is_free_stream_exact() {
    // A₀·const = const (the off-diagonal −s·(S₊+S₋) cancels the +2s on the diagonal), so A₀⁻¹·const = const
    // to round-off. This is the property the AMEn-per-step swap broke (lost to the solve tolerance).
    let l = 7usize;
    let n = 1usize << l;
    let c = 2.75;
    let b = quantize(&CausalTensor::new(vec![c; n], vec![n]).unwrap(), &tr()).unwrap();
    let inv = AcousticCoreInverse::new_1d(l, S, tr()).unwrap();
    let x = dequantize(&inv.apply(&b).unwrap()).unwrap();
    for &v in x.as_slice() {
        assert!((v - c).abs() < 1e-10, "free-stream drifted: {v} vs {c}");
    }
}

#[test]
fn rho_matches_the_analytic_contracting_root() {
    // ρ = (1 + 2s − √(1+4s)) / (2s); for s = 8 that is (17 − √33)/16 ≈ 0.7034, and 0 < ρ < 1.
    let inv = AcousticCoreInverse::new_1d(6, S, tr()).unwrap();
    let expected = (1.0 + 2.0 * S - (1.0 + 4.0 * S).sqrt()) / (2.0 * S);
    assert!((inv.rho() - expected).abs() < 1e-12, "rho = {}", inv.rho());
    assert!(inv.rho() > 0.0 && inv.rho() < 1.0, "rho out of (0,1)");
}

#[test]
fn inverse_works_at_small_stiffness() {
    // As s → 0, A₀ → I and the inverse → I (ρ → 0); the solve must still hit round-off.
    let (res, _bond) = inverse_residual(6, 0.05);
    assert!(res < 1e-9, "small-stiffness residual {res:.2e}");
}

#[test]
fn new_1d_rejects_zero_modes() {
    assert!(AcousticCoreInverse::<f64>::new_1d(0, S, tr()).is_err());
}

#[test]
fn rejects_non_positive_or_non_finite_stiffness() {
    assert!(AcousticCoreInverse::<f64>::new_1d(5, 0.0, tr()).is_err());
    assert!(AcousticCoreInverse::<f64>::new_1d(5, -1.0, tr()).is_err());
    assert!(AcousticCoreInverse::<f64>::new_1d(5, f64::NAN, tr()).is_err());
}

/// `A₀ = I − β·∇²` on a `2^lx × 2^ly` grid (cells `dx = dy = h`), as an MPO.
fn build_a0_2d(lx: usize, ly: usize, h: f64, beta: f64) -> CausalTensorTrainOperator<f64> {
    let id = CausalTensorTrainOperator::<f64>::identity(&vec![2usize; lx + ly]);
    let lap = laplacian_2d::<f64>(lx, ly, h, h, &tr()).unwrap();
    id.add(&lap.scale(-beta)).unwrap()
}

#[test]
fn adi_inverse_is_free_stream_exact() {
    // The ADI split (I−β∂ₓ²)⁻¹(I−β∂ᵧ²)⁻¹ maps a uniform field to itself exactly (each 1-D factor does),
    // independent of the O(β²) cross-term splitting error — the property the marcher relies on.
    let (lx, ly) = (4usize, 4usize);
    let n = (1usize << lx) * (1usize << ly);
    let h = 1.0 / (1usize << lx) as f64;
    let c = 1.7;
    let b = quantize_2d(
        &CausalTensor::new(vec![c; n], vec![1 << lx, 1 << ly]).unwrap(),
        &tr(),
    )
    .unwrap();
    let inv = AcousticCoreInverse2d::new(lx, ly, h, h, 0.5 * h * h, tr()).unwrap();
    let x = dequantize_2d(&inv.apply(&b).unwrap(), lx, ly).unwrap();
    for &v in x.as_slice() {
        // Roundoff-exact: the drift is pure floating-point accumulation through the 2·(lx+ly) rounding
        // steps of the ADI composition, not a tolerance-bearing solver residual.
        assert!((v - c).abs() < 1e-8, "2-D free-stream drifted: {v} vs {c}");
    }
}

#[test]
fn adi_inverse_approximates_the_2d_solve() {
    // For a smooth field at a modest per-axis stiffness, the ADI inverse is an accurate approximate
    // solve: ‖A₀·(A₀⁻¹b) − b‖/‖b‖ is at the O(β²) splitting error, and the inverse stays low-rank.
    let (lx, ly) = (5usize, 5usize);
    let (nx, ny) = (1usize << lx, 1usize << ly);
    let h = 1.0 / nx as f64;
    let beta = 0.5 * h * h; // per-axis stiffness s = 0.5
    let mut b_dense = vec![0.0f64; nx * ny];
    for ix in 0..nx {
        for iy in 0..ny {
            let x = ix as f64 / nx as f64;
            let y = iy as f64 / ny as f64;
            b_dense[ix * ny + iy] = (TAU * x).sin() * (TAU * y).cos() + 0.3;
        }
    }
    let b = quantize_2d(
        &CausalTensor::new(b_dense.clone(), vec![nx, ny]).unwrap(),
        &tr(),
    )
    .unwrap();
    let inv = AcousticCoreInverse2d::new(lx, ly, h, h, beta, tr()).unwrap();
    let x: CausalTensorTrain<f64> = inv.apply(&b).unwrap();
    let ax = build_a0_2d(lx, ly, h, beta).apply(&x, &tr()).unwrap();
    let ax_dense = dequantize_2d(&ax, lx, ly).unwrap();
    let num: f64 = ax_dense
        .as_slice()
        .iter()
        .zip(&b_dense)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f64>()
        .sqrt();
    let den: f64 = b_dense.iter().map(|v| v * v).sum::<f64>().sqrt();
    assert!(num / den < 1e-2, "ADI residual {:.2e} too large", num / den);
    assert!(
        x.max_bond() <= 24,
        "2-D inverse bond {} not low-rank",
        x.max_bond()
    );
}

#[test]
fn adi_inverse_rejects_zero_modes() {
    assert!(AcousticCoreInverse2d::<f64>::new(0, 3, 0.1, 0.1, 0.01, tr()).is_err());
    assert!(AcousticCoreInverse2d::<f64>::new(3, 0, 0.1, 0.1, 0.01, tr()).is_err());
}

#[test]
fn adi_3d_inverse_is_free_stream_exact() {
    // The 3-D ADI split maps a uniform field to itself (each 1-D factor does), to round-off.
    let (lx, ly, lz) = (3usize, 3usize, 3usize);
    let (nx, ny, nz) = (1 << lx, 1 << ly, 1 << lz);
    let h = 1.0 / nx as f64;
    let c = 0.9;
    let b = quantize_3d(
        &CausalTensor::new(vec![c; nx * ny * nz], vec![nx, ny, nz]).unwrap(),
        &tr(),
    )
    .unwrap();
    let inv = AcousticCoreInverse3d::new((lx, ly, lz), (h, h, h), 0.5 * h * h, tr()).unwrap();
    let x = dequantize_3d(&inv.apply(&b).unwrap(), lx, ly, lz).unwrap();
    for &v in x.as_slice() {
        assert!((v - c).abs() < 1e-8, "3-D free-stream drifted: {v} vs {c}");
    }
}

#[test]
fn adi_3d_inverse_approximates_the_solve() {
    // A smooth 3-D field at modest stiffness: ‖A₀·(A₀⁻¹b) − b‖/‖b‖ at the O(β²) splitting error, low-rank.
    let (lx, ly, lz) = (4usize, 4usize, 4usize);
    let (nx, ny, nz) = (1 << lx, 1 << ly, 1 << lz);
    let h = 1.0 / nx as f64;
    let beta = 0.5 * h * h;
    let mut b_dense = vec![0.0f64; nx * ny * nz];
    for ix in 0..nx {
        for iy in 0..ny {
            for iz in 0..nz {
                let x = ix as f64 / nx as f64;
                let y = iy as f64 / ny as f64;
                let z = iz as f64 / nz as f64;
                b_dense[(ix * ny + iy) * nz + iz] =
                    (TAU * x).sin() * (TAU * y).cos() * (TAU * z).cos() + 0.4;
            }
        }
    }
    let b = quantize_3d(
        &CausalTensor::new(b_dense.clone(), vec![nx, ny, nz]).unwrap(),
        &tr(),
    )
    .unwrap();
    let inv = AcousticCoreInverse3d::new((lx, ly, lz), (h, h, h), beta, tr()).unwrap();
    let x = inv.apply(&b).unwrap();
    let id = CausalTensorTrainOperator::<f64>::identity(&vec![2usize; lx + ly + lz]);
    let lap = laplacian_3d::<f64>(lx, ly, lz, h, h, h, &tr()).unwrap();
    let a0 = id.add(&lap.scale(-beta)).unwrap();
    let ax = dequantize_3d(&a0.apply(&x, &tr()).unwrap(), lx, ly, lz).unwrap();
    let num: f64 = ax
        .as_slice()
        .iter()
        .zip(&b_dense)
        .map(|(a, b)| (a - b) * (a - b))
        .sum::<f64>()
        .sqrt();
    let den: f64 = b_dense.iter().map(|v| v * v).sum::<f64>().sqrt();
    assert!(
        num / den < 2e-2,
        "3-D ADI residual {:.2e} too large",
        num / den
    );
    assert!(
        x.max_bond() <= 32,
        "3-D inverse bond {} not low-rank",
        x.max_bond()
    );
}

#[test]
fn adi_3d_inverse_rejects_zero_modes() {
    assert!(AcousticCoreInverse3d::<f64>::new((0, 2, 2), (0.1, 0.1, 0.1), 0.01, tr()).is_err());
    assert!(AcousticCoreInverse3d::<f64>::new((2, 0, 2), (0.1, 0.1, 0.1), 0.01, tr()).is_err());
    assert!(AcousticCoreInverse3d::<f64>::new((2, 2, 0), (0.1, 0.1, 0.1), 0.01, tr()).is_err());
}

#[test]
fn from_shift_pows_rejects_empty_or_mismatched() {
    let sp = vec![shift_plus::<f64>(4).unwrap()];
    let empty: Vec<CausalTensorTrainOperator<f64>> = vec![];
    assert!(AcousticCoreInverse::from_shift_pows(S, empty.clone(), empty, tr()).is_err());
    let sm = vec![
        shift_minus::<f64>(4).unwrap(),
        shift_minus::<f64>(4).unwrap(),
    ];
    assert!(AcousticCoreInverse::from_shift_pows(S, sp, sm, tr()).is_err());
}
