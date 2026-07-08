/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/DivisionAlgebra.lean` (Mathlib `DivisionRing`).
//! `f64` via the crate's `DivisionAlgebra` trait is the representative carrier.

use deep_causality_algebra::DivisionAlgebra;

/// THEOREM_MAP: algebra.division_algebra.mul_inv
#[test]
fn test_division_algebra_mul_inv() {
    // a ≠ 0 → a * a⁻¹ = 1, with the inverse taken through `DivisionAlgebra::inverse`.
    for a in [2.0f64, -3.5, 7.25] {
        assert_ne!(a, 0.0);
        assert_eq!(a * a.inverse(), 1.0);
    }
}
