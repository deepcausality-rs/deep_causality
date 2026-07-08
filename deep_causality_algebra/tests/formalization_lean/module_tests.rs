/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Module.lean` (Mathlib `Module`, `Algebra`).
//! The scalar action is modelled by `f64` multiplication through the crate's `Module::scale`,
//! with `f64` acting as both the scalar ring and the module/algebra carrier.

use deep_causality_algebra::Module;

/// r • x, realised by the crate's `Module::scale` on `f64` (r • x = r * x).
fn smul(r: f64, x: f64) -> f64 {
    <f64 as Module<f64>>::scale(&x, r)
}

/// THEOREM_MAP: algebra.module.smul_add
#[test]
fn test_module_smul_add() {
    // r • (x + y) = r • x + r • y
    let (r, x, y) = (3.0f64, 2.0f64, 5.0f64);
    assert_eq!(smul(r, x + y), smul(r, x) + smul(r, y));
}

/// THEOREM_MAP: algebra.module.add_smul
#[test]
fn test_module_add_smul() {
    // (r + s) • x = r • x + s • x
    let (r, s, x) = (3.0f64, 4.0f64, 2.0f64);
    assert_eq!(smul(r + s, x), smul(r, x) + smul(s, x));
}

/// THEOREM_MAP: algebra.module.one_smul
#[test]
fn test_module_one_smul() {
    // 1 • x = x
    for x in [2.0f64, -3.5, 7.25] {
        assert_eq!(smul(1.0, x), x);
    }
}

/// THEOREM_MAP: algebra.module.mul_smul
#[test]
fn test_module_mul_smul() {
    // (r * s) • x = r • (s • x)
    let (r, s, x) = (3.0f64, 4.0f64, 2.0f64);
    assert_eq!(smul(r * s, x), smul(r, smul(s, x)));
}

/// THEOREM_MAP: algebra.algebra.smul_mul_assoc
#[test]
fn test_algebra_smul_mul_assoc() {
    // r • (a * b) = (r • a) * b
    let (r, a, b) = (3.0f64, 2.0f64, 5.0f64);
    assert_eq!(smul(r, a * b), smul(r, a) * b);
}

/// THEOREM_MAP: algebra.algebra.mul_smul_comm
#[test]
fn test_algebra_mul_smul_comm() {
    // r • (a * b) = a * (r • b)
    let (r, a, b) = (3.0f64, 2.0f64, 5.0f64);
    assert_eq!(smul(r, a * b), a * smul(r, b));
}
