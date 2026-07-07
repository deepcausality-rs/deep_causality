/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact 3-D KS propagator (`KsPropagator`) — the singularity-free, perturbation-ready B1 conformal
//! core (Gap-3 FS-1). Tests cover coast exactness (round-off) against the independent planar Kepler
//! propagator, period closure, the semigroup property, conservation, the Strang perturbation hook
//! (2nd order, linear-in-ε, zero-perturbation identity), and the rejections. f64 + Float106.

use deep_causality_num::Float106;
use deep_causality_physics::{EARTH_GM, KsPropagator, TwoBodyPropagator, ks_strang_step};

// A bound, inclined, eccentric orbit (genuinely 3-D).
fn state_3d() -> ([f64; 3], [f64; 3]) {
    ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3])
}

fn n3(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
fn energy(r: [f64; 3], v: [f64; 3]) -> f64 {
    let rr = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]) / 2.0 - EARTH_GM / rr
}
fn ang_mom(r: [f64; 3], v: [f64; 3]) -> [f64; 3] {
    [
        r[1] * v[2] - r[2] * v[1],
        r[2] * v[0] - r[0] * v[2],
        r[0] * v[1] - r[1] * v[0],
    ]
}

#[test]
fn coast_matches_independent_planar_kepler_to_round_off() {
    // A planar (z = 0) state: KS in 3-D must agree with the shipped 2-D TwoBodyPropagator (an
    // independent implementation) to round-off, and keep the out-of-plane component ~0.
    let (r0, v0) = ([7.0e6, 0.0, 0.0], [1.0e3, 7.5e3, 0.0]);
    let ks = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let planar = TwoBodyPropagator::from_state([r0[0], r0[1]], [v0[0], v0[1]], EARTH_GM).unwrap();

    for k in 1..=12 {
        let dt = 500.0 * k as f64;
        let (rk, vk) = ks.propagate(dt).unwrap();
        let (rp, vp) = planar.propagate(dt).unwrap();
        assert!(
            (rk[0] - rp[0]).abs() < 1e-6 && (rk[1] - rp[1]).abs() < 1e-6,
            "position drift vs planar Kepler at dt={dt}"
        );
        assert!((vk[0] - vp[0]).abs() < 1e-9 && (vk[1] - vp[1]).abs() < 1e-9);
        assert!(rk[2].abs() < 1e-3 && vk[2].abs() < 1e-6, "stays in plane");
    }
}

#[test]
fn one_period_closes_to_round_off() {
    let (r0, v0) = state_3d();
    let ks = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let (r1, v1) = ks.propagate(ks.period().unwrap()).unwrap();
    // Return to the epoch state after exactly one orbit. Scale-relative: |r| ~ 7e6, |v| ~ 7e3.
    assert!(n3(r0, r1) < 1e-3, "position closes: {}", n3(r0, r1));
    assert!(n3(v0, v1) < 1e-6, "velocity closes: {}", n3(v0, v1));
}

#[test]
fn semigroup_property() {
    let (r0, v0) = state_3d();
    let ks = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let (dt1, dt2) = (321.0, 654.0);
    // Direct dt1 + dt2.
    let (rd, vd) = ks.propagate(dt1 + dt2).unwrap();
    // Compose: propagate dt1, rebuild, propagate dt2.
    let (r1, v1) = ks.propagate(dt1).unwrap();
    let (rc, vc) = KsPropagator::from_state(r1, v1, EARTH_GM)
        .unwrap()
        .propagate(dt2)
        .unwrap();
    assert!(n3(rd, rc) < 1e-4, "semigroup position: {}", n3(rd, rc));
    assert!(n3(vd, vc) < 1e-7, "semigroup velocity: {}", n3(vd, vc));
}

#[test]
fn conserves_energy_and_angular_momentum() {
    let (r0, v0) = state_3d();
    let ks = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let e0 = energy(r0, v0);
    let l0 = ang_mom(r0, v0);
    for k in 1..=20 {
        let (r, v) = ks.propagate(137.0 * k as f64).unwrap();
        assert!(
            ((energy(r, v) - e0) / e0).abs() < 1e-12,
            "energy conserved at step {k}"
        );
        let l = ang_mom(r, v);
        assert!(n3(l, l0) / n3(l0, [0.0; 3]) < 1e-12, "|L| conserved");
    }
}

#[test]
fn kepler_third_law() {
    let (r0, v0) = state_3d();
    let ks = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let a = ks.semi_major_axis();
    // T² = 4π²a³/μ.
    let t2 = ks.period().unwrap().powi(2);
    let rhs = 4.0 * core::f64::consts::PI.powi(2) * a.powi(3) / EARTH_GM;
    assert!((t2 - rhs).abs() / rhs < 1e-12, "Kepler III");
}

#[test]
fn rejects_unbound_and_degenerate() {
    // Escape velocity => energy >= 0 => rejected.
    let vesc = (2.0 * EARTH_GM / 7.0e6).sqrt();
    assert!(
        KsPropagator::from_state([7.0e6, 0.0, 0.0], [0.0, vesc * 1.01, 0.0], EARTH_GM).is_err()
    );
    // Non-positive GM.
    assert!(KsPropagator::from_state([7.0e6, 0.0, 0.0], [0.0, 7.5e3, 0.0], -1.0).is_err());
    // Zero radius.
    assert!(KsPropagator::from_state([0.0, 0.0, 0.0], [0.0, 7.5e3, 0.0], EARTH_GM).is_err());
}

// ── Strang perturbation hook (FS-2) ─────────────────────────────────────────────────────────────

/// RK4 truth for the full perturbed EOM `ẍ = −μ x/|x|³ − k v` over `horizon` with tiny steps.
fn rk4_truth(r0: [f64; 3], v0: [f64; 3], k: f64, horizon: f64, n: usize) -> ([f64; 3], [f64; 3]) {
    let h = horizon / n as f64;
    let deriv = |r: [f64; 3], v: [f64; 3]| -> ([f64; 3], [f64; 3]) {
        let rr = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        let c = -EARTH_GM / (rr * rr * rr);
        let a = [
            c * r[0] - k * v[0],
            c * r[1] - k * v[1],
            c * r[2] - k * v[2],
        ];
        (v, a)
    };
    let (mut r, mut v) = (r0, v0);
    let add =
        |a: [f64; 3], b: [f64; 3], s: f64| [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s];
    for _ in 0..n {
        let (k1r, k1v) = deriv(r, v);
        let (k2r, k2v) = deriv(add(r, k1r, h / 2.0), add(v, k1v, h / 2.0));
        let (k3r, k3v) = deriv(add(r, k2r, h / 2.0), add(v, k2v, h / 2.0));
        let (k4r, k4v) = deriv(add(r, k3r, h), add(v, k3v, h));
        for i in 0..3 {
            r[i] += h / 6.0 * (k1r[i] + 2.0 * k2r[i] + 2.0 * k3r[i] + k4r[i]);
            v[i] += h / 6.0 * (k1v[i] + 2.0 * k2v[i] + 2.0 * k3v[i] + k4v[i]);
        }
    }
    (r, v)
}

/// March `ks_strang_step` over `horizon` with `n` macro-steps; return the final position error vs truth.
fn strang_error(r0: [f64; 3], v0: [f64; 3], k: f64, horizon: f64, n: usize) -> f64 {
    let h = horizon / n as f64;
    let accel = move |_r: [f64; 3], v: [f64; 3]| [-k * v[0], -k * v[1], -k * v[2]];
    let (mut r, mut v) = (r0, v0);
    for _ in 0..n {
        let (rn, vn) = ks_strang_step(r, v, EARTH_GM, h, accel).unwrap();
        r = rn;
        v = vn;
    }
    let (rt, _) = rk4_truth(r0, v0, k, horizon, n * 200);
    n3(r, rt)
}

#[test]
fn strang_split_is_second_order() {
    let (r0, v0) = state_3d();
    // k = 1e-6 keeps the drag ratio ε ≈ a_drag/a_grav ~ 9e-4 (FS-2's regime), where the leading O(H²)
    // Strang term dominates; a stronger perturbation contaminates the measured order with ε²H² terms.
    let (k, horizon) = (1.0e-6, 600.0);
    let e_h = strang_error(r0, v0, k, horizon, 20);
    let e_h2 = strang_error(r0, v0, k, horizon, 40);
    let order = (e_h / e_h2).log2();
    assert!(
        (1.8..=2.2).contains(&order),
        "observed order {order} (err_H={e_h}, err_H/2={e_h2})"
    );
}

#[test]
fn strang_error_shrinks_with_perturbation() {
    let (r0, v0) = state_3d();
    let horizon = 600.0;
    let e_big = strang_error(r0, v0, 1.0e-4, horizon, 20);
    let e_small = strang_error(r0, v0, 1.0e-5, horizon, 20);
    // Split error is ~linear in the perturbation ratio: 10× weaker perturbation ⇒ ~10× smaller error.
    assert!(
        e_small < e_big / 5.0,
        "error should shrink with ε: big={e_big}, small={e_small}"
    );
}

#[test]
#[cfg_attr(miri, ignore)] // Miri's soft-float emulation drifts the two propagation paths past the 1e-9/1e-12 tolerance; test is correct under normal CI.
fn zero_perturbation_equals_exact_core() {
    let (r0, v0) = state_3d();
    let dt = 400.0;
    let zero = |_r: [f64; 3], _v: [f64; 3]| [0.0; 3];
    let (rs, vs) = ks_strang_step(r0, v0, EARTH_GM, dt, zero).unwrap();
    let (rc, vc) = KsPropagator::from_state(r0, v0, EARTH_GM)
        .unwrap()
        .propagate(dt)
        .unwrap();
    assert!(
        n3(rs, rc) < 1e-9 && n3(vs, vc) < 1e-12,
        "zero kick == exact drift"
    );
}

// ── Float106 precision ──────────────────────────────────────────────────────────────────────────

fn fp(x: f64) -> Float106 {
    Float106::from_f64(x)
}
fn fp3(a: [f64; 3]) -> [Float106; 3] {
    [fp(a[0]), fp(a[1]), fp(a[2])]
}

#[test]
fn float106_period_closes_tighter_than_f64() {
    let (r0, v0) = state_3d();
    let ks = KsPropagator::from_state(fp3(r0), fp3(v0), fp(EARTH_GM)).unwrap();
    let (r1, _v1) = ks.propagate(ks.period().unwrap()).unwrap();
    // Float106 carries ~30 digits: the closure is far tighter than the f64 gate. Compare in f64 space.
    let d = |a: Float106, b: f64| (a.to_f64() - b).abs();
    let dr = d(r1[0], r0[0]) + d(r1[1], r0[1]) + d(r1[2], r0[2]);
    assert!(dr < 1e-6, "Float106 one-period closure: {dr}");
}
