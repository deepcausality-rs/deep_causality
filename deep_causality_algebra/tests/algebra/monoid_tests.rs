/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the Num-layer monoid laws proved in Lean.
//!
//! Lean source of truth: `lean/DeepCausalityFormal/Algebra/Monoid.lean`.
//! Bound via `lean/THEOREM_MAP.md`. Each test carries the shared `THEOREM_MAP` id.

use deep_causality_num::Zero;

/// Witness for `THEOREM_MAP: algebra.add_monoid.assoc`.
///
/// Exercises the associativity law `(a + b) + c = a + (b + c)` that the Lean theorem
/// `add_monoid_assoc` proves.
#[test]
fn test_add_monoid_associativity() {
    let (a, b, c) = (7i64, 11i64, 13i64);
    assert_eq!((a + b) + c, a + (b + c));
}

/// Witness for `THEOREM_MAP: algebra.add_monoid.identity`.
///
/// Exercises the two-sided identity law `a + 0 = a` and `0 + a = a` that the Lean theorem
/// `add_monoid_identity` proves, with `0` obtained from the `Zero` trait.
#[test]
fn test_add_monoid_identity() {
    let a = 42i64;
    assert_eq!(a + i64::zero(), a);
    assert_eq!(i64::zero() + a, a);
}
