/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{One, UnsignedInt, Zero};

/// The natural numbers **ℕ** = {0, 1, 2, …}.
///
/// This is the set-named entry point for ℕ, the counterpart of [`Integer`](crate::Integer) for ℤ,
/// [`Real`](https://docs.rs/deep_causality_algebra) for ℝ, `Rational` for ℚ, and `Complex` for ℂ.
/// Its algebraic counterpart is `CommutativeSemiring` in `deep_causality_algebra`; this trait is
/// where the *operations* of ℕ live, stated in ℕ's own vocabulary.
///
/// # What ℕ is
///
/// ℕ is a **commutative semiring** and nothing above it: two commutative monoids on one carrier
/// (`+` with identity `0`, `·` with identity `1`), multiplication distributing over addition, and
/// `0` annihilating under multiplication. It is *not* a ring, because it has no additive
/// inverses — and it is that single absence which shapes everything below.
///
/// # Subtraction is partial, and this trait says so twice
///
/// `a - b` exists in ℕ exactly when `b ≤ a`. Equivalently, the natural order *is* the
/// subtraction: `a ≤ b` iff there is some `c ∈ ℕ` with `a + c = b`. Two total functions stand in
/// for the partial one, and they answer different questions:
///
/// - [`checked_difference`](Self::checked_difference) returns `None` when the difference does not
///   exist. Use it when the absence is meaningful.
/// - [`monus`](Self::monus) — truncated subtraction, written `a ∸ b` — returns `0` instead. This
///   is the standard *total* operation on ℕ, and it is what makes `(ℕ, ∸)` well defined.
///
/// The `Sub` operator is deliberately not part of this trait. `u64` has one, but `3u64 - 5u64`
/// panics in debug and wraps in release, so it does not compute ℕ's difference in either mode.
///
/// # Division and gcd
///
/// Division with remainder is partial at zero, and [`div_rem`](Self::div_rem) reflects that.
/// Over ℕ the quotient is unambiguous: truncating, flooring, and Euclidean division all coincide,
/// because there are no negative operands to disagree about.
///
/// [`gcd`](Self::gcd) is the reason this trait carries operations rather than only a name. ℕ has
/// a perfectly good greatest common divisor — cleaner than ℤ's, since there is no sign and so no
/// choice of associate to make — but ℕ is not a ring, so it cannot reach
/// `EuclideanDomain`, which is where the signed gcd lives. Without this trait there is no generic
/// gcd for unsigned types at all.
///
/// # What `u64` actually is
///
/// Honesty about the carrier: `u64`'s value set is an initial segment of ℕ, not ℕ itself. It is
/// not closed under `+` or `·`, and under wrapping arithmetic it is the ring `ℤ/2⁶⁴ℤ` — which
/// *has* additive inverses and *has* zero divisors (`2⁶³ · 2 = 0`), two things ℕ has neither of.
/// So the methods here that can leave the representable range say so in their return type
/// ([`succ`](Self::succ), [`checked_difference`](Self::checked_difference),
/// [`lcm`](Self::lcm)) rather than silently wrapping. [`pred`](Self::pred) is the one whose
/// `None` is genuine ℕ partiality rather than a representation limit: `0` has no predecessor in
/// ℕ, at any width.
pub trait NaturalNumber: UnsignedInt + Zero + One {
    /// The successor `n + 1`, or `None` if it is not representable.
    ///
    /// The Peano successor. Its `None` is a representation limit, not a property of ℕ — every
    /// natural number has a successor.
    #[inline]
    fn succ(self) -> Option<Self> {
        self.checked_add(Self::one())
    }

    /// The predecessor `n - 1`, or `None` for zero.
    ///
    /// Unlike [`succ`](Self::succ), this `None` is genuine: `0` has no predecessor in ℕ at any
    /// width. This is the Peano base case, and it is why induction on ℕ needs one.
    #[inline]
    fn pred(self) -> Option<Self> {
        self.checked_sub(Self::one())
    }

    /// Truncated subtraction, `a ∸ b` — the difference, or `0` when `b > a`.
    ///
    /// The standard *total* operation standing in for ℕ's partial subtraction. `(ℕ, ∸)` is well
    /// defined for every pair, which ordinary subtraction is not.
    #[inline]
    fn monus(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }

    /// The difference `a - b`, or `None` when `b > a` and it does not exist in ℕ.
    ///
    /// The partial subtraction stated honestly. Prefer this over the `Sub` operator, which
    /// panics in debug builds and wraps in release rather than reporting the absence.
    #[inline]
    fn checked_difference(self, rhs: Self) -> Option<Self> {
        self.checked_sub(rhs)
    }

    /// Division with remainder: `(q, r)` with `a = b·q + r` and `r < b`, or `None` when `b` is
    /// zero.
    ///
    /// Over ℕ the pair is unique and unambiguous — truncating, flooring, and Euclidean division
    /// coincide, because no operand can be negative.
    #[inline]
    fn div_rem(self, rhs: Self) -> Option<(Self, Self)> {
        match (self.checked_div(rhs), self.checked_rem(rhs)) {
            (Some(q), Some(r)) => Some((q, r)),
            _ => None,
        }
    }

    /// The greatest common divisor, by the Euclidean algorithm.
    ///
    /// Cleaner than the signed case: with no sign there is no choice of associate, so no
    /// normalization step is needed and the result is already canonical.
    ///
    /// - `gcd(a, 0) = a`
    /// - `gcd(a, b) = gcd(b, a mod b)`
    /// - `gcd(a, b)` divides both `a` and `b`
    #[inline]
    fn gcd(self, rhs: Self) -> Self {
        let mut a = self;
        let mut b = rhs;
        while !b.is_zero() {
            let r = a.rem_euclid(b);
            a = b;
            b = r;
        }
        a
    }

    /// The least common multiple, or `None` if it is not representable.
    ///
    /// Divides before multiplying: forming `a·b` first overflows whenever the product exceeds the
    /// width even though the answer would fit.
    ///
    /// Returns `Some(0)` when either argument is zero, since `0` is a multiple of everything.
    #[inline]
    fn lcm(self, rhs: Self) -> Option<Self> {
        if self.is_zero() || rhs.is_zero() {
            return Some(Self::zero());
        }
        let g = self.gcd(rhs);
        // `g` divides `self` exactly, so this division is exact and loses nothing.
        self.div_euclid(g).checked_mul(rhs)
    }
}

// Every unsigned primitive is a natural number. All methods are defaulted in terms of the
// `Integer` surface, so a new unsigned width needs nothing beyond `UnsignedInt + Zero + One`.
impl<T> NaturalNumber for T where T: UnsignedInt + Zero + One {}
