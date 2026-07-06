/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Either.lean` (coproduct; Mac Lane, CWM §III.3).

use deep_causality_haft::Either;

/// THEOREM_MAP: haft.either.coproduct_universal
#[test]
fn test_either_coproduct_universal() {
    // The mediating morphism [f, g] is a match; both injection equations hold.
    // (Uniqueness is quantified over all h — provable only in Lean; the Rust witness
    // pins the two equations.)
    fn either_fold<L, R, C>(f: impl Fn(L) -> C, g: impl Fn(R) -> C, e: Either<L, R>) -> C {
        match e {
            Either::Left(l) => f(l),
            Either::Right(r) => g(r),
        }
    }
    let f = |l: i32| l * 2;
    let g = |r: &str| r.len() as i32;
    assert_eq!(either_fold(f, g, Either::Left(21)), f(21));
    assert_eq!(either_fold(f, g, Either::Right("hello")), g("hello"));
}
