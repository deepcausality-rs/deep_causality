/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/Monoid.lean` (Mathlib `AddMonoid`).

/// THEOREM_MAP: algebra.add_monoid.assoc
#[test]
fn test_add_monoid_assoc() {
    // (a + b) + c = a + (b + c)
    let (a, b, c) = (7i64, 11i64, 13i64);
    assert_eq!((a + b) + c, a + (b + c));
}

/// THEOREM_MAP: algebra.add_monoid.identity
#[test]
fn test_add_monoid_identity() {
    // a + 0 = a and 0 + a = a
    let a = 42i64;
    assert_eq!(a + 0i64, a);
    assert_eq!(0i64 + a, a);
}
