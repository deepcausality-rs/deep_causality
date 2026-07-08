/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The verdict carrier: a bounded lattice with complement — the output type of `Collection`
//! aggregation.

/// A **verdict**: a bounded lattice with complement (a Boolean algebra for `bool`; an MV-algebra
/// for the probability carrier, where `complement(p) = 1 − p`).
///
/// The aggregation output type is a `Verdict`: `All` folds `meet`, `Any` folds `join`, and `None`
/// is `Any` post-composed with `complement`. `bottom`/`top` are the lattice bounds (`false`/`true`
/// for `bool`).
///
/// # Laws
/// - bounded-lattice: `meet`/`join` are associative, commutative, absorptive; `bottom`/`top` are
///   the identities of `join`/`meet`.
/// - complement: involution `complement(complement(x)) == x` and (for the Boolean class) De Morgan.
pub trait Verdict: Sized {
    /// The lattice bottom (identity of `join`).
    fn bottom() -> Self;
    /// The lattice top (identity of `meet`).
    fn top() -> Self;
    /// Greatest lower bound (`∧` for `bool`).
    fn meet(self, other: Self) -> Self;
    /// Least upper bound (`∨` for `bool`).
    fn join(self, other: Self) -> Self;
    /// Complement (`!` for `bool`; `1 − p` for the probability MV-algebra).
    fn complement(self) -> Self;
}

impl Verdict for bool {
    #[inline]
    fn bottom() -> Self {
        false
    }
    #[inline]
    fn top() -> Self {
        true
    }
    #[inline]
    fn meet(self, other: Self) -> Self {
        self && other
    }
    #[inline]
    fn join(self, other: Self) -> Self {
        self || other
    }
    #[inline]
    fn complement(self) -> Self {
        !self
    }
}
