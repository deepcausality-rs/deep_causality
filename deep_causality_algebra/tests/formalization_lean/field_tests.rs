/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Field.lean` (Mathlib `Field`,
//! `LinearOrderedField`). `f64` is the representative carrier.

/// THEOREM_MAP: algebra.field.mul_inv_cancel
#[test]
fn test_field_mul_inv_cancel() {
    // a ≠ 0 → a * a⁻¹ = 1
    for a in [2.0f64, -3.5, 7.25] {
        assert_ne!(a, 0.0);
        assert_eq!(a * a.recip(), 1.0);
    }
}

/// THEOREM_MAP: algebra.field.inv_mul_cancel
#[test]
fn test_field_inv_mul_cancel() {
    // a ≠ 0 → a⁻¹ * a = 1
    for a in [2.0f64, -3.5, 7.25] {
        assert_ne!(a, 0.0);
        assert_eq!(a.recip() * a, 1.0);
    }
}

/// THEOREM_MAP: algebra.real_field.mul_pos
#[test]
fn test_real_field_mul_pos() {
    // 0 < a → 0 < b → 0 < a * b
    for (a, b) in [(2.0f64, 3.0f64), (0.25, 0.5), (7.0, 11.0)] {
        assert!(a > 0.0 && b > 0.0);
        assert!(a * b > 0.0);
    }
}
