/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier-B 0.5.2: the continuous body-fit blend `BlendedMap` (the `λ` dial, design D8 / Resolution 4).
//! `λ = 1` reproduces the body-fitted metric exactly; `λ = 0` is the Cartesian-capture rectangle; the
//! sweep is a genuine **rank dial** for a fixed physical shock (gate BM-B), free-stream-exact throughout.

use deep_causality_cfd::{
    BlendedMap, BlendedMapConfig, BodyFittedCoordinate, MetricProvider, dequantize_2d,
};
use deep_causality_tensor::Truncation;

const PI: f64 = core::f64::consts::PI;

fn tr() -> Truncation<f64> {
    Truncation::by_tol(1e-10).unwrap()
}

/// A blend config over the standard test fan `r ∈ [r0, r0+dr]`, `θ ∈ [−dθ/2, +dθ/2]` at `lambda`.
fn cfg(l: usize, r0: f64, dr: f64, dth: f64, lambda: f64) -> BlendedMapConfig<f64> {
    BlendedMapConfig::new(l, l, r0, dr, -dth / 2.0, dth, lambda)
}

fn smoothed_step(d: f64, w: f64) -> f64 {
    0.5 * (1.0 + (d / w).tanh())
}

#[test]
fn lambda_one_reproduces_the_body_fitted_metric() {
    // At λ = 1 the blended forward Jacobian is the polar fan's, so its inverse metric equals
    // `BodyFittedCoordinate`'s. Same gradient operators + same metric ⇒ identical physical gradients.
    let (lx, ly, r0, dr, dth) = (5usize, 5usize, 1.0f64, 1.0, PI / 2.0);
    // BlendedMap centers the fan at θ = 0 (θ ∈ [−dθ/2, +dθ/2]); the body-fitted partner must match.
    let th0 = -dth / 2.0;
    let blend = BlendedMap::new(cfg(lx, r0, dr, dth, 1.0), tr()).unwrap();
    let fitted = BodyFittedCoordinate::<f64>::new(lx, ly, r0, dr, th0, dth, tr()).unwrap();

    let f = |xi: f64, eta: f64| (r0 + eta * dr) * (th0 + xi * dth).cos() + 0.3 * (xi + eta);
    let ub = blend.sample(f).unwrap();
    let uf = fitted.sample(f).unwrap();
    let (bx, by) = blend.physical_gradient(&ub).unwrap();
    let (fx, fy) = fitted.physical_gradient(&uf).unwrap();
    let (bxd, byd) = (
        dequantize_2d(&bx, lx, ly).unwrap(),
        dequantize_2d(&by, lx, ly).unwrap(),
    );
    let (fxd, fyd) = (
        dequantize_2d(&fx, lx, ly).unwrap(),
        dequantize_2d(&fy, lx, ly).unwrap(),
    );
    for (&b, &f) in bxd.as_slice().iter().zip(fxd.as_slice()) {
        assert!((b - f).abs() < 1e-9, "∂/∂x mismatch at λ=1: {b} vs {f}");
    }
    for (&b, &f) in byd.as_slice().iter().zip(fyd.as_slice()) {
        assert!((b - f).abs() < 1e-9, "∂/∂y mismatch at λ=1: {b} vs {f}");
    }
}

#[test]
fn lambda_zero_is_the_cartesian_capture_rectangle() {
    // At λ = 0 the chart is the rectangle x = r0 + η·Δr, y = −span/2 + ξ·span, so u = x has ∂u/∂x = 1,
    // ∂u/∂y = 0 in the interior (the η operator wraps at the radial boundary).
    let (lx, ly, r0, dr) = (5usize, 5usize, 1.0f64, 1.0);
    let blend = BlendedMap::new(cfg(lx, r0, dr, PI / 2.0, 0.0), tr()).unwrap();
    let u = blend.sample(|_xi, eta| r0 + eta * dr).unwrap();
    let (dudx, dudy) = blend.physical_gradient(&u).unwrap();
    let gx = dequantize_2d(&dudx, lx, ly).unwrap();
    let gy = dequantize_2d(&dudy, lx, ly).unwrap();
    let (nx, ny) = (1usize << lx, 1usize << ly);
    for i in 0..nx {
        for j in 1..ny - 1 {
            assert!(
                (gx.as_slice()[i * ny + j] - 1.0).abs() < 0.02,
                "∂x/∂x at {i},{j}"
            );
            assert!(gy.as_slice()[i * ny + j].abs() < 0.02, "∂x/∂y at {i},{j}");
        }
    }
}

#[test]
fn free_stream_preserved_across_the_blend() {
    // A uniform field has exactly zero physical gradient at every λ (the metric identity holds discretely).
    for &lam in &[0.0, 0.5, 1.0] {
        let blend = BlendedMap::new(cfg(5, 1.0, 1.0, PI / 2.0, lam), tr()).unwrap();
        let u = blend.sample(|_xi, _eta| 2.4).unwrap();
        let (gx, gy) = blend.physical_gradient(&u).unwrap();
        let gxd = dequantize_2d(&gx, 5, 5).unwrap();
        let gyd = dequantize_2d(&gy, 5, 5).unwrap();
        for v in gxd.as_slice().iter().chain(gyd.as_slice()) {
            assert!(v.abs() < 1e-9, "λ={lam}: free-stream gradient {v} ≠ 0");
        }
    }
}

#[test]
fn lambda_is_a_rank_dial_for_a_fixed_physical_shock() {
    // Gate BM-B: a fixed physical curved shock (tanh at radius RSHOCK), sampled on the λ-blended lattice,
    // runs from high rank (λ=0, the curved front on a square grid) to O(10) (λ=1, the front on the radial
    // axis). So λ is a genuine rank knob.
    let (l, r0, dr, dth) = (7usize, 1.0f64, 1.0, PI / 2.0);
    let r_shock = 1.5;
    let w = 0.04;
    let bond = |lam: f64| -> usize {
        let blend = BlendedMap::new(cfg(l, r0, dr, dth, lam), tr()).unwrap();
        blend
            .sample(|xi, eta| {
                let (x, y) = blend.position(xi, eta);
                smoothed_step((x * x + y * y).sqrt() - r_shock, w)
            })
            .unwrap()
            .max_bond()
    };
    let b0 = bond(0.0);
    let bmid = bond(0.5);
    let b1 = bond(1.0);
    assert!(b1 <= 12, "fitted (λ=1) not O(10): {b1}");
    assert!(b0 > b1, "λ=0 capture {b0} should exceed fitted {b1}");
    assert!(
        bmid <= b0 + 1,
        "intermediate λ should not exceed capture: {bmid} vs {b0}"
    );
}

#[test]
fn metric_provider_seam_exposes_dims_jacobian_and_sample() {
    // Exercise the trait methods (static-dispatch seam the marcher consumes).
    let blend = BlendedMap::new(
        BlendedMapConfig::new(4, 5, 1.0, 1.0, 0.0, PI / 2.0, 0.6),
        tr(),
    )
    .unwrap();
    fn use_metric<M: MetricProvider<f64>>(m: &M) -> (usize, usize, usize) {
        let s = m.sample(|_xi, _eta| 1.0).unwrap();
        (
            m.dims().0,
            m.dims().1,
            m.jacobian().max_bond().max(s.max_bond()),
        )
    }
    let (dx, dy, _bond) = use_metric(&blend);
    assert_eq!((dx, dy), (4, 5), "dims");
    assert!((blend.lambda() - 0.6).abs() < 1e-15, "lambda getter");
    assert!(blend.jacobian().max_bond() <= 8, "jacobian low-rank");
}

#[test]
fn rejects_invalid_geometry_and_lambda() {
    let bad = |r0, dr, dth, lam| {
        BlendedMap::new(BlendedMapConfig::new(4, 4, r0, dr, 0.0, dth, lam), tr())
    };
    assert!(bad(0.0, 1.0, PI / 2.0, 0.5).is_err(), "r0 = 0");
    assert!(bad(1.0, 0.0, PI / 2.0, 0.5).is_err(), "dr = 0");
    assert!(bad(1.0, 1.0, 0.0, 0.5).is_err(), "dtheta = 0");
    assert!(bad(1.0, 1.0, PI / 2.0, -0.1).is_err(), "lambda < 0");
    assert!(bad(1.0, 1.0, PI / 2.0, 1.1).is_err(), "lambda > 1");
}

#[test]
fn metric_provider_physical_gradient_delegates_to_inherent() {
    // The trait's `physical_gradient` forwards to the inherent method (identical result) — the static
    // dispatch seam a marcher consumes.
    let (lx, ly) = (5usize, 5usize);
    let blend = BlendedMap::new(cfg(lx, 1.0, 1.0, PI / 2.0, 0.4), tr()).unwrap();
    let u = blend.sample(|xi, eta| (xi * PI).cos() + 0.3 * eta).unwrap();

    let (ax, ay) = MetricProvider::physical_gradient(&blend, &u).unwrap();
    let (bx, by) = blend.physical_gradient(&u).unwrap();
    let ax = dequantize_2d(&ax, lx, ly).unwrap();
    let bx = dequantize_2d(&bx, lx, ly).unwrap();
    let ay = dequantize_2d(&ay, lx, ly).unwrap();
    let by = dequantize_2d(&by, lx, ly).unwrap();
    for (p, q) in ax.as_slice().iter().zip(bx.as_slice()) {
        assert!((p - q).abs() < 1e-12, "∂/∂x trait vs inherent: {p} vs {q}");
    }
    for (p, q) in ay.as_slice().iter().zip(by.as_slice()) {
        assert!((p - q).abs() < 1e-12, "∂/∂y trait vs inherent: {p} vs {q}");
    }
}

// --- Enforced invertibility (item 8, blocker B-4) -------------------------------------------------
//
// The constructor's validity guarantee has to be falsifiable, so these pin the two ways `det J_λ`
// can fail. All three share the same wide fan — `r0 = 0.1`, `dr = 5`, `dθ = 5` — and differ only in
// `λ`, so a rejection is attributable to the blend and not to the geometry being exotic.
//
// The fold and the floor are reached at different `λ` and are checked separately: at `λ = 0.2` the
// determinant changes sign while `min|det J|` is still ~10⁴× the floor, so only the sign test can
// fire; the near-singular `λ` sits just under the fold threshold, where `det` is one-signed at 0.51×
// the floor. Neither case can be passing for the other's reason.

/// `λ` just below the fold threshold for the wide test fan, where `det J_λ` is still one-signed but
/// has fallen under the floor. Obtained by bisecting the threshold on `max(det)` — `0.142856850670…`
/// — and stepping back `1.1e-7` into the rejecting band. The band is ~2.2e-7 wide in `λ`, which is
/// narrow against `λ ∈ [0, 1]` but nine orders above float noise, so the constant is stable.
const LAMBDA_NEAR_SINGULAR: f64 = 0.142_856_740_671;

#[test]
fn a_folded_map_is_rejected_and_names_the_fold() {
    // λ = 0.2 on the wide fan reverses the sign of det J_λ, so the chart is not invertible over the
    // computational domain and the inverse metric would be meaningless rather than merely large.
    let Err(err) = BlendedMap::new(cfg(5, 0.1, 5.0, 5.0, 0.2), tr()) else {
        panic!("a folded map must not construct");
    };
    let msg = format!("{err}");
    assert!(msg.contains("folds"), "the fold must be named, got: {msg}");
}

#[test]
fn a_near_singular_map_is_rejected_and_names_the_floor() {
    // One-signed but under the floor: the inverse metric (cofactor / det) would be unbounded.
    let Err(err) = BlendedMap::new(cfg(5, 0.1, 5.0, 5.0, LAMBDA_NEAR_SINGULAR), tr()) else {
        panic!("a near-singular map must not construct");
    };
    let msg = format!("{err}");
    assert!(
        msg.contains("near-singular"),
        "the floor must be named, got: {msg}"
    );
}

#[test]
fn the_same_fan_constructs_at_admissible_blends() {
    // The control the two rejections need: the wide fan itself is fine, so the refusals above are
    // about det J_λ and not about the constructor disliking a large dθ. Both ends of the sweep sit
    // five or more orders above the floor.
    for lambda in [0.0, 0.05, 0.9, 1.0] {
        assert!(
            BlendedMap::new(cfg(5, 0.1, 5.0, 5.0, lambda), tr()).is_ok(),
            "the wide fan must construct at λ = {lambda}"
        );
    }
}

#[test]
fn the_validity_scan_covers_the_closed_domain() {
    // The near-singular map degenerates at the (ξ, η) = (1, 1) corner, which the metric trains never
    // sample (`sample_grid` forms ξ = i/nx for i in 0..nx). If the scan tracked only the sampled
    // lattice this map would construct, and the sampled minimum falls off only as ~1/nx, so refining
    // would not save it — the rejection has to come from closing the domain. Same verdict at every
    // level is the observable consequence.
    for l in [4usize, 5, 6] {
        assert!(
            BlendedMap::new(cfg(l, 0.1, 5.0, 5.0, LAMBDA_NEAR_SINGULAR), tr()).is_err(),
            "the near-singular map must be refused at lx = ly = {l}"
        );
    }
}
