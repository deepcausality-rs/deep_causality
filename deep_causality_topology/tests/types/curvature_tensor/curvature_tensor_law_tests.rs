/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Riemann-curvature laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Topology/RiemannCurvature.lean`
//! (do Carmo, *Riemannian Geometry*, Ch. 4). Bound via `lean/THEOREM_MAP.md`.
//!
//! The Lean model proves the laws for the canonical constant-curvature operator
//! `R(u,v)w = K(g(v,w)·u − g(u,w)·v)`; these tests instantiate the SAME form on the concrete
//! `CurvatureTensor` — components `R^d_abc = K(δ^d_a g_bc − δ^d_b g_ac)` with the Euclidean
//! `g = δ` — and check the same statements through `contract` / `check_bianchi_identity`.
//! Linearity is additionally checked on arbitrary (asymmetric) components, since it holds
//! for every component array, not just lawful Riemann tensors.

use deep_causality_metric::Metric;
use deep_causality_topology::{CurvatureSymmetry, CurvatureTensor};

const DIM: usize = 3;
const K: f64 = 2.0;

/// Constant-curvature components `R^d_abc = K(δ^d_a δ_bc − δ^d_b δ_ac)` — the maximally
/// symmetric space with Euclidean g (do Carmo Ch. 4, Lemma 3.4).
fn constant_curvature() -> CurvatureTensor<f64, f64, f64, f64, f64> {
    let delta = |i: usize, j: usize| if i == j { 1.0 } else { 0.0 };
    CurvatureTensor::from_generator(
        DIM,
        Metric::Euclidean(DIM),
        CurvatureSymmetry::Riemann,
        |d, a, b, c| K * (delta(d, a) * delta(b, c) - delta(d, b) * delta(a, c)),
    )
}

/// Arbitrary asymmetric components — linearity of `contract` must hold for these too.
fn arbitrary_components() -> CurvatureTensor<f64, f64, f64, f64, f64> {
    CurvatureTensor::from_generator(
        DIM,
        Metric::Euclidean(DIM),
        CurvatureSymmetry::None,
        |d, a, b, c| ((d * 7 + a * 5 + b * 3 + c) % 11) as f64 - 5.0,
    )
}

fn assert_vec_eq(lhs: &[f64], rhs: &[f64]) {
    assert_eq!(lhs.len(), rhs.len());
    for (l, r) in lhs.iter().zip(rhs.iter()) {
        assert!((l - r).abs() < 1e-12, "expected {l} == {r}");
    }
}

/// THEOREM_MAP: topology.curvature.antisymmetry
#[test]
fn test_curvature_antisymmetry() {
    // R(u,v)w = -R(v,u)w — swapping the loop directions negates the holonomy.
    let tensor = constant_curvature();
    let u = [1.0, 2.0, -1.0];
    let v = [0.5, -3.0, 2.0];
    let w = [4.0, 0.0, 1.0];

    let r_uv = tensor.contract(&u, &v, &w);
    let r_vu = tensor.contract(&v, &u, &w);
    let neg_r_vu: Vec<f64> = r_vu.iter().map(|x| -x).collect();

    assert_vec_eq(&r_uv, &neg_r_vu);
    // Sanity: the tensor is not flat, so the law is not vacuous.
    assert!(!tensor.is_flat());
    assert!(r_uv.iter().any(|x| x.abs() > 1e-9));
}

/// THEOREM_MAP: topology.curvature.bianchi_first
#[test]
fn test_curvature_bianchi_first() {
    // Cyclic sum R(u,v)w + R(v,w)u + R(w,u)v = 0 — needs exactly the symmetry of g.
    let tensor = constant_curvature();

    // Component-level check via the crate's own detector.
    assert!(tensor.check_bianchi_identity() < 1e-12);

    // Operator-level check via contraction.
    let u = [1.0, 2.0, -1.0];
    let v = [0.5, -3.0, 2.0];
    let w = [4.0, 0.0, 1.0];
    let r1 = tensor.contract(&u, &v, &w);
    let r2 = tensor.contract(&v, &w, &u);
    let r3 = tensor.contract(&w, &u, &v);
    for d in 0..DIM {
        assert!((r1[d] + r2[d] + r3[d]).abs() < 1e-12);
    }

    // The detector detects: a tensor violating the cyclic identity reports a violation.
    let broken: CurvatureTensor<f64, f64, f64, f64, f64> = CurvatureTensor::from_generator(
        DIM,
        Metric::Euclidean(DIM),
        CurvatureSymmetry::None,
        |d, a, b, c| {
            if d == 0 && a == 0 && b == 1 && c == 2 {
                1.0
            } else {
                0.0
            }
        },
    );
    assert!(broken.check_bianchi_identity() > 0.5);
}

/// THEOREM_MAP: topology.curvature.linearity
#[test]
fn test_curvature_linearity() {
    // contract is multilinear: additivity and homogeneity in each slot. Checked in the
    // transported slot w (as in the Lean theorems) and, since contract is a symmetric
    // triple sum, in u as well. Holds for ARBITRARY components.
    let tensor = arbitrary_components();
    let u = [1.0, 2.0, -1.0];
    let v = [0.5, -3.0, 2.0];
    let w1 = [4.0, 0.0, 1.0];
    let w2 = [-1.0, 2.5, 3.0];
    let k = -2.5;

    // Additivity in w: R(u,v)(w1 + w2) = R(u,v)w1 + R(u,v)w2
    let w_sum: Vec<f64> = w1.iter().zip(w2.iter()).map(|(a, b)| a + b).collect();
    let lhs = tensor.contract(&u, &v, &w_sum);
    let r1 = tensor.contract(&u, &v, &w1);
    let r2 = tensor.contract(&u, &v, &w2);
    let rhs: Vec<f64> = r1.iter().zip(r2.iter()).map(|(a, b)| a + b).collect();
    assert_vec_eq(&lhs, &rhs);

    // Homogeneity in w: R(u,v)(k·w) = k·R(u,v)w
    let kw: Vec<f64> = w1.iter().map(|x| k * x).collect();
    let lhs = tensor.contract(&u, &v, &kw);
    let rhs: Vec<f64> = r1.iter().map(|x| k * x).collect();
    assert_vec_eq(&lhs, &rhs);

    // Additivity in u: R(u1 + u2, v)w = R(u1,v)w + R(u2,v)w
    let u2 = [0.0, 1.0, 5.0];
    let u_sum: Vec<f64> = u.iter().zip(u2.iter()).map(|(a, b)| a + b).collect();
    let lhs = tensor.contract(&u_sum, &v, &w1);
    let ra = tensor.contract(&u, &v, &w1);
    let rb = tensor.contract(&u2, &v, &w1);
    let rhs: Vec<f64> = ra.iter().zip(rb.iter()).map(|(a, b)| a + b).collect();
    assert_vec_eq(&lhs, &rhs);
}
