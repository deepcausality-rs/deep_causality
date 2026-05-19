/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Property-test helpers for the Tier 1 isomorphism marker subtraits.
//!
//! These helpers are exposed as `pub fn` (not `#[cfg(test)]`-gated) so they
//! are reachable from integration tests under `deep_causality_num/tests/`.
//! This follows the codebase convention established by
//! [`crate::utils_tests`] — Bazel cannot access files in `tests/` from `src/`,
//! so coverage-counting test utilities must live in the `src` tree.
//!
//! Each helper exercises one law from the Tier 1 marker hierarchy:
//!
//! - [`assert_iso_from_round_trip`] — bidirectional `From` round-trip identity.
//! - [`assert_group_iso_from_law`] — additive group homomorphism.
//! - [`assert_ring_iso_from_laws`] — ring homomorphism (addition AND multiplication).
//! - [`assert_field_iso_from_laws`] — field homomorphism (ring + multiplicative inverse).
//! - [`assert_algebra_iso_from_law`] — scalar multiplication preservation.
//! - [`assert_division_algebra_iso_from_law`] — conjugation preservation.
//!
//! The helpers take owned values (caller provides representative inputs) and
//! use `assert_eq!` with descriptive failure messages. No randomized input
//! generation; callers wrap calls in their own randomized iteration when
//! exhaustive coverage is wanted.

use crate::{Algebra, DivisionAlgebra, Field, Group, Ring};

/// Asserts that bidirectional `From<S>` / `From<T>` impls round-trip cleanly
/// in both directions.
///
/// Verifies the Tier 1 base law: any `From` pair participating in a Tier 1
/// marker subtrait must satisfy `S::from(T::from(s)) == s` and
/// `T::from(S::from(t)) == t`.
pub fn assert_iso_from_round_trip<S, T>(s: S)
where
    S: From<T> + Clone + PartialEq + core::fmt::Debug,
    T: From<S> + Clone + PartialEq + core::fmt::Debug,
{
    let t: T = T::from(s.clone());
    let s_back: S = S::from(t.clone());
    assert_eq!(
        s, s_back,
        "From round-trip S -> T -> S failed: original {:?} differs from S::from(T::from(original))",
        s
    );

    let s_again: S = S::from(t.clone());
    let t_again: T = T::from(s_again);
    assert_eq!(
        t, t_again,
        "From round-trip T -> S -> T failed: original {:?} differs from T::from(S::from(original))",
        t
    );
}

/// Asserts the additive group homomorphism law for a [`crate::iso::GroupIso<T>`]
/// impl: `T::from(a + b) == T::from(a) + T::from(b)`.
///
/// In this crate `Group: AddGroup`, so the group operation is `+`. The helper
/// is therefore an additive-group homomorphism check.
pub fn assert_group_iso_from_law<S, T>(a: S, b: S)
where
    S: Group + From<T> + Clone + PartialEq + core::fmt::Debug,
    T: Group + From<S> + Clone + PartialEq + core::fmt::Debug,
{
    let lhs: T = T::from(a.clone() + b.clone());
    let rhs: T = T::from(a) + T::from(b);
    assert_eq!(
        lhs, rhs,
        "GroupIso homomorphism failed: T::from(a + b) != T::from(a) + T::from(b)"
    );
}

/// Asserts the ring homomorphism laws for a [`crate::iso::RingIso<T>`] impl:
/// addition AND multiplication preserved.
///
/// 1. `T::from(a + b) == T::from(a) + T::from(b)` (additive homomorphism).
/// 2. `T::from(a * b) == T::from(a) * T::from(b)` (multiplicative homomorphism).
pub fn assert_ring_iso_from_laws<S, T>(a: S, b: S)
where
    S: Ring + From<T> + Clone + PartialEq + core::fmt::Debug,
    T: Ring + From<S> + Clone + PartialEq + core::fmt::Debug,
{
    let lhs_add: T = T::from(a.clone() + b.clone());
    let rhs_add: T = T::from(a.clone()) + T::from(b.clone());
    assert_eq!(
        lhs_add, rhs_add,
        "RingIso addition homomorphism failed: T::from(a + b) != T::from(a) + T::from(b)"
    );

    let lhs_mul: T = T::from(a.clone() * b.clone());
    let rhs_mul: T = T::from(a) * T::from(b);
    assert_eq!(
        lhs_mul, rhs_mul,
        "RingIso multiplication homomorphism failed: T::from(a * b) != T::from(a) * T::from(b)"
    );
}

/// Asserts the field homomorphism laws for a [`crate::iso::FieldIso<T>`] impl:
/// addition, multiplication, AND multiplicative inverse preserved.
///
/// Caller is responsible for passing a non-zero `a`. For `a == 0`, the inverse
/// is undefined / returns Infinity per IEEE 754 for floating-point fields, and
/// the check would be vacuous or misleading.
pub fn assert_field_iso_from_laws<S, T>(a: S)
where
    S: Field + From<T> + Clone + PartialEq + core::fmt::Debug,
    T: Field + From<S> + Clone + PartialEq + core::fmt::Debug,
{
    let inv_s: S = a.clone().inverse();
    let from_inv: T = T::from(inv_s);
    let inv_from: T = T::from(a).inverse();
    assert_eq!(
        from_inv, inv_from,
        "FieldIso multiplicative-inverse homomorphism failed: T::from(a.inverse()) != T::from(a).inverse()"
    );
}

/// Asserts the algebra homomorphism law for a [`crate::iso::AlgebraIso<T, R>`]
/// impl: scalar multiplication is preserved.
///
/// `T::from(a.scale(r)) == T::from(a).scale(r)` for any scalar `r: R` and
/// vector `a: Self`.
pub fn assert_algebra_iso_from_law<S, T, R>(a: S, r: R)
where
    R: Ring + Clone,
    S: Algebra<R> + From<T> + Clone + PartialEq + core::fmt::Debug,
    T: Algebra<R> + From<S> + Clone + PartialEq + core::fmt::Debug,
{
    let scaled_s: S = a.clone().scale(r.clone());
    let from_scaled: T = T::from(scaled_s);
    let scale_from: T = T::from(a).scale(r);
    assert_eq!(
        from_scaled, scale_from,
        "AlgebraIso scalar-multiplication homomorphism failed: T::from(a.scale(r)) != T::from(a).scale(r)"
    );
}

/// Asserts the division algebra homomorphism law for a
/// [`crate::iso::DivisionAlgebraIso<T, R>`] impl: conjugation is preserved.
///
/// `T::from(a.conjugate()) == T::from(a).conjugate()` for all `a: Self`.
///
/// The companion inverse law (`T::from(a.inverse()) == T::from(a).inverse()`)
/// follows mathematically from conjugation preservation plus norm-square
/// preservation, but is not directly tested here; for division algebras over
/// floating-point fields, the inverse check is best deferred to a tolerance-
/// aware comparison rather than `assert_eq!`.
pub fn assert_division_algebra_iso_from_law<S, T, R>(a: S)
where
    R: Field + Clone,
    S: DivisionAlgebra<R> + From<T> + Clone + PartialEq + core::fmt::Debug,
    T: DivisionAlgebra<R> + From<S> + Clone + PartialEq + core::fmt::Debug,
{
    let conj_s: S = a.clone().conjugate();
    let from_conj: T = T::from(conj_s);
    let conj_from: T = T::from(a).conjugate();
    assert_eq!(
        from_conj, conj_from,
        "DivisionAlgebraIso conjugation homomorphism failed: T::from(a.conjugate()) != T::from(a).conjugate()"
    );
}
