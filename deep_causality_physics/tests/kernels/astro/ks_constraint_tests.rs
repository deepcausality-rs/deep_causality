/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! KS bilinear constraint projection (B2). Tests: idempotence on an on-surface state, restoration of a
//! perturbed velocity to the surface, the projection is the *nearest* point (the removed component is
//! along the constraint gradient), and the zero-position rejection.

use deep_causality_physics::{ks_bilinear_residual, ks_project_velocity};

// A KS state (u, w) that already satisfies b(u, w) = 0 by construction: pick u freely and pick w in the
// constraint's null space. g = (u4, −u3, u2, −u1); any w orthogonal to g satisfies b = 0. Take w = u
// (b(u, u) = u4u1 − u3u2 + u2u3 − u1u4 = 0 identically).
fn on_surface() -> ([f64; 4], [f64; 4]) {
    let u = [1.3, -0.7, 0.4, 0.9];
    (u, u)
}

fn n4(a: [f64; 4], b: [f64; 4]) -> f64 {
    (0..4).map(|i| (a[i] - b[i]).powi(2)).sum::<f64>().sqrt()
}

#[test]
fn residual_is_zero_on_surface_and_projection_is_identity() {
    let (u, w) = on_surface();
    assert!(
        ks_bilinear_residual(u, w).abs() < 1e-15,
        "b(u, u) is identically 0"
    );
    let w2 = ks_project_velocity(u, w).unwrap();
    assert!(
        n4(w2, w) < 1e-15,
        "projection of an on-surface state is identity"
    );
}

#[test]
fn projection_restores_a_perturbed_velocity() {
    let (u, w0) = on_surface();
    // Perturb w off the surface.
    let w = [w0[0] + 0.5, w0[1] - 0.3, w0[2] + 0.2, w0[3] + 0.1];
    assert!(
        ks_bilinear_residual(u, w).abs() > 1e-3,
        "perturbation leaves the surface"
    );

    let wp = ks_project_velocity(u, w).unwrap();
    assert!(
        ks_bilinear_residual(u, wp).abs() < 1e-13,
        "projected velocity is back on the surface: {}",
        ks_bilinear_residual(u, wp)
    );
}

#[test]
fn projection_is_idempotent() {
    let u = [2.0, 1.0, -0.5, 0.3];
    let w = [0.1, -0.4, 0.9, 1.2];
    let w1 = ks_project_velocity(u, w).unwrap();
    let w2 = ks_project_velocity(u, w1).unwrap();
    assert!(
        n4(w1, w2) < 1e-15,
        "projecting twice equals projecting once"
    );
}

#[test]
fn projection_is_the_nearest_point() {
    // The removed component (w − w') must be parallel to the constraint gradient g = (u4, −u3, u2, −u1),
    // and no smaller move restores the constraint (orthogonal projection).
    let u = [2.0, 1.0, -0.5, 0.3];
    let w = [0.1, -0.4, 0.9, 1.2];
    let wp = ks_project_velocity(u, w).unwrap();
    let delta = [w[0] - wp[0], w[1] - wp[1], w[2] - wp[2], w[3] - wp[3]];
    let g = [u[3], -u[2], u[1], -u[0]];
    // delta ∥ g  ⇔  delta = λ g  ⇔  cross-ratios equal ⇔ |delta·g| = |delta||g|.
    let dg: f64 = (0..4).map(|i| delta[i] * g[i]).sum();
    let dd: f64 = (0..4).map(|i| delta[i] * delta[i]).sum::<f64>().sqrt();
    let gg: f64 = (0..4).map(|i| g[i] * g[i]).sum::<f64>().sqrt();
    assert!(
        (dg.abs() - dd * gg).abs() < 1e-12,
        "correction is along the constraint gradient"
    );
}

#[test]
fn rejects_zero_position() {
    assert!(ks_project_velocity([0.0; 4], [1.0, 2.0, 3.0, 4.0]).is_err());
}
