/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    QttProjector2d, body_mask_2d, dequantize_2d, divergence_residual, drag_lift, kinetic_energy,
    max_bond, max_speed, penalization_heat_integral, quantize_2d,
};
use deep_causality_tensor::{CausalTensor, TensorTrain, Truncation};

const TAU: f64 = core::f64::consts::TAU;
const N: usize = 16;
const L: usize = 4;

fn field(dx: f64, f: impl Fn(f64, f64) -> f64) -> CausalTensor<f64> {
    let mut data = vec![0.0; N * N];
    for i in 0..N {
        for j in 0..N {
            data[i * N + j] = f(i as f64 * dx, j as f64 * dx);
        }
    }
    CausalTensor::new(data, vec![N, N]).unwrap()
}

fn tg_u(x: f64, y: f64) -> f64 {
    -(x.cos() * y.sin())
}
fn tg_v(x: f64, y: f64) -> f64 {
    x.sin() * y.cos()
}

#[test]
fn kinetic_energy_matches_dense() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let uf = field(dx, tg_u);
    let vf = field(dx, tg_v);
    let u = quantize_2d(&uf, &trunc).unwrap();
    let v = quantize_2d(&vf, &trunc).unwrap();

    // Dense reference: ½ Σ (u² + v²) over the grid coefficients (Frobenius/L2 energy).
    let dense: f64 = uf
        .as_slice()
        .iter()
        .zip(vf.as_slice())
        .map(|(a, b)| a * a + b * b)
        .sum::<f64>()
        * 0.5;

    let ke = kinetic_energy(&u, &v).unwrap();
    assert!((ke - dense).abs() <= 1e-9, "ke {ke} vs dense {dense}");
}

#[test]
fn divergence_residual_is_tt_native() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let u = quantize_2d(&field(dx, tg_u), &trunc).unwrap();
    let v = quantize_2d(&field(dx, tg_v), &trunc).unwrap();
    let projector = QttProjector2d::new(L, L, dx, dx, trunc).unwrap();

    // The Taylor–Green vortex is divergence-free, so the residual norm is ~ 0.
    let res = divergence_residual(&projector, &u, &v).unwrap();
    assert!(res <= 1e-9, "divergence residual {res} too large");

    // Agreement with the explicit divergence-train norm.
    let div_norm = projector.divergence(&u, &v).unwrap().norm().unwrap();
    assert!((res - div_norm).abs() <= 1e-12);
}

#[test]
fn max_bond_matches_cores() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let u = quantize_2d(&field(dx, tg_u), &trunc).unwrap();
    let v = quantize_2d(&field(dx, tg_v), &trunc).unwrap();

    let want = u
        .cores()
        .iter()
        .chain(v.cores().iter())
        .map(|c| c.shape()[2])
        .max()
        .unwrap();
    assert_eq!(max_bond(&u, &v), want);
}

#[test]
fn max_speed_matches_dense() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let uf = field(dx, tg_u);
    let vf = field(dx, tg_v);
    let u = quantize_2d(&uf, &trunc).unwrap();
    let v = quantize_2d(&vf, &trunc).unwrap();

    let dense = uf
        .as_slice()
        .iter()
        .zip(vf.as_slice())
        .map(|(a, b)| (a * a + b * b).sqrt())
        .fold(0.0f64, f64::max);

    let ms = max_speed(&u, &v, L, L).unwrap();
    assert!(
        (ms - dense).abs() <= 1e-9,
        "max_speed {ms} vs dense {dense}"
    );
}

#[test]
fn drag_lift_matches_the_dense_penalization_integral() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let (cx, cy, r) = (TAU * 0.5, TAU * 0.5, TAU * 0.18);
    let eta = 0.02;
    let mask = body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, 2.0 * dx, &trunc).unwrap();

    // A decelerated free-stream: u = 1 outside, braked inside (any field — we cross-check the integral).
    let uf = field(dx, |x, y| 1.0 - (x.cos() * y.sin()) * 0.3);
    let vf = field(dx, |x, y| (x.sin() * y.cos()) * 0.2);
    let u = quantize_2d(&uf, &trunc).unwrap();
    let v = quantize_2d(&vf, &trunc).unwrap();

    let (u_ref, d_ref) = (1.0, 2.0 * r);
    let (cd, cl) = drag_lift(&mask, &u, &v, 0.0, 0.0, eta, dx, dx, u_ref, d_ref).unwrap();

    // Dense reference: F = (1/η) Σ mask·(vel − u_body) dV, nondimensionalized.
    let md = dequantize_2d(&mask, L, L).unwrap();
    let (ms, us, vs) = (md.as_slice(), uf.as_slice(), vf.as_slice());
    let dv = dx * dx;
    let fx: f64 = ms.iter().zip(us).map(|(m, a)| m * a).sum::<f64>() * dv / eta;
    let fy: f64 = ms.iter().zip(vs).map(|(m, a)| m * a).sum::<f64>() * dv / eta;
    let denom = 0.5 * u_ref * u_ref * d_ref;
    assert!(
        (cd - fx / denom).abs() <= 1e-9,
        "C_d {cd} vs dense {}",
        fx / denom
    );
    assert!(
        (cl - fy / denom).abs() <= 1e-9,
        "C_l {cl} vs dense {}",
        fy / denom
    );
}

#[test]
fn penalization_heat_integral_responds_to_a_hot_wall() {
    let dx = TAU / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-12).unwrap();
    let (cx, cy, r) = (TAU * 0.5, TAU * 0.5, TAU * 0.18);
    let eta = 0.02;
    let mask = body_mask_2d::<f64>(L, L, dx, dx, cx, cy, r, 2.0 * dx, &trunc).unwrap();

    // Cold flow (T = 0), hot wall (T_w = 1): heat flows into the fluid → Q > 0.
    let temp = quantize_2d(&field(dx, |_, _| 0.0), &trunc).unwrap();
    let q_hot = penalization_heat_integral(&mask, &temp, 1.0, eta, dx, dx).unwrap();
    assert!(q_hot > 0.0, "hot wall should source heat, got {q_hot}");

    // Matched temperature (T_w = 0) → no flux.
    let q_none = penalization_heat_integral(&mask, &temp, 0.0, eta, dx, dx).unwrap();
    assert!(
        q_none.abs() <= 1e-12,
        "no temperature gap should give ~0 flux, got {q_none}"
    );

    // Dense reference for the hot wall.
    let md = dequantize_2d(&mask, L, L).unwrap();
    let dense: f64 = md.as_slice().iter().map(|m| m * (1.0 - 0.0)).sum::<f64>() * dx * dx / eta;
    assert!((q_hot - dense).abs() <= 1e-9, "Q {q_hot} vs dense {dense}");
}

#[test]
fn strip_pressure_force_matches_the_dense_integral_on_a_uniform_field() {
    // A uniform conserved state has a uniform pressure p0, so the strip contraction must equal
    // p0 · Σχ · dV — computable densely to round-off.
    let gamma = 1.4;
    let dx = 1.0 / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();
    let (rho0, u0, p0) = (1.0, 2.0, 0.75);
    let e0 = p0 / (gamma - 1.0) + 0.5 * rho0 * u0 * u0;
    let enc = |v: f64| quantize_2d(&field(dx, |_, _| v), &trunc).unwrap();
    let state = [enc(rho0), enc(rho0 * u0), enc(0.0), enc(e0)];

    // A forebody-strip mask: a thin vertical band (uniform field → the sharp indicator is exact).
    let strip = deep_causality_cfd::mask_from_fn::<f64, _>(
        L,
        L,
        dx,
        dx,
        |x, _| if (x - 0.6).abs() < 0.06 { 1.0 } else { 0.0 },
        &trunc,
    )
    .unwrap();

    let force =
        deep_causality_cfd::strip_pressure_force(&strip, &state, gamma, L, L, dx, dx, &trunc)
            .unwrap();
    let sd = dequantize_2d(&strip, L, L).unwrap();
    let dense: f64 = sd.as_slice().iter().sum::<f64>() * p0 * dx * dx;
    assert!(
        (force - dense).abs() < 1e-9,
        "contracted {force} vs dense {dense}"
    );
}

#[test]
fn strip_pressure_force_rejects_a_nonpositive_density() {
    let gamma = 1.4;
    let dx = 1.0 / N as f64;
    let trunc = Truncation::<f64>::by_tol(1e-10).unwrap();
    // One cell of exactly zero density breaks the positive cone.
    let rho = quantize_2d(
        &field(dx, |x, y| if x == 0.0 && y == 0.0 { 0.0 } else { 1.0 }),
        &trunc,
    )
    .unwrap();
    let zero = quantize_2d(&field(dx, |_, _| 0.0), &trunc).unwrap();
    let e = quantize_2d(&field(dx, |_, _| 2.5), &trunc).unwrap();
    let strip = quantize_2d(&field(dx, |_, _| 1.0), &trunc).unwrap();
    let state = [rho, zero.clone(), zero, e];
    assert!(
        deep_causality_cfd::strip_pressure_force(&strip, &state, gamma, L, L, dx, dx, &trunc)
            .is_err()
    );
}

#[test]
fn preserved_drag_fraction_is_a_guarded_ratio() {
    // The powered/unpowered ratio, with the baseline guarded: an unpowered run's own fraction is
    // one, and a vanishing or non-finite baseline is a singularity, not a quiet infinity.
    let f: f64 = deep_causality_cfd::preserved_drag_fraction(0.3_f64, 1.2).unwrap();
    assert!((f - 0.25).abs() < 1e-15);
    let unity: f64 = deep_causality_cfd::preserved_drag_fraction(1.2_f64, 1.2).unwrap();
    assert!((unity - 1.0).abs() < 1e-15);
    assert!(deep_causality_cfd::preserved_drag_fraction(0.3, 0.0).is_err());
    assert!(deep_causality_cfd::preserved_drag_fraction(0.3, f64::NAN).is_err());
}
