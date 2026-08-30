/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Num/Cast.lean`.
//!
//! Pins the primitive-cast round-trip and injectivity laws to the crate's real `FromPrimitive`,
//! `ToPrimitive`, and `NumCast` traits. Lean proves the laws for all naturals/integers; these
//! tests check them empirically at representative inputs.

use deep_causality_num::{FromPrimitive, NumCast, ToPrimitive};

/// THEOREM_MAP: num.cast.nat_int_roundtrip
#[test]
fn test_cast_nat_int_roundtrip() {
    // Round-trip a small non-negative integer up through the signed cast and back:
    // n : usize -> i64 -> usize recovers the same value.
    for n in [0usize, 1, 7, 42, 1000] {
        let widened: i64 = i64::from_usize(n).expect("usize fits in i64");
        let recovered: usize = widened.to_usize().expect("non-negative i64 fits in usize");
        assert_eq!(recovered, n);
    }
}

/// THEOREM_MAP: num.cast.int_injective
#[test]
fn test_cast_int_injective() {
    // Distinct integers map to distinct casts in the characteristic-zero field f64:
    // the widening i64 -> f64 loses no information at these magnitudes, so no two
    // distinct inputs alias.
    let inputs = [-42i64, -1, 0, 1, 7, 42, 1000];
    for (i, &a) in inputs.iter().enumerate() {
        for &b in inputs.iter().skip(i + 1) {
            let fa: f64 = NumCast::from(a).expect("i64 casts to f64");
            let fb: f64 = NumCast::from(b).expect("i64 casts to f64");
            assert_ne!(a, b);
            assert_ne!(fa, fb, "distinct integers {a} and {b} aliased under cast");
        }
    }
}
