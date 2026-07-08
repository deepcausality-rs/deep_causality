/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Group.lean` (Mathlib `Group`, `AddGroup`,
//! `AddCommGroup`). `f64` is the representative carrier.

/// THEOREM_MAP: algebra.group.mul_inv
#[test]
fn test_group_mul_inv() {
    // a * a⁻¹ = 1 for a ≠ 0
    for a in [2.0f64, -3.5, 7.25] {
        assert_eq!(a * (1.0 / a), 1.0);
    }
}

/// THEOREM_MAP: algebra.add_group.neg_cancel
#[test]
fn test_add_group_neg_cancel() {
    // -a + a = 0
    for a in [2.0f64, -3.5, 0.0, 7.25] {
        assert_eq!(-a + a, 0.0);
    }
}

/// THEOREM_MAP: algebra.abelian_group.add_comm
#[test]
fn test_abelian_group_add_comm() {
    // a + b = b + a
    let (a, b) = (1.5f64, -4.25f64);
    assert_eq!(a + b, b + a);
}
