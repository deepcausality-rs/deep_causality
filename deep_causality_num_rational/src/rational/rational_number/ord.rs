/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Equality and ordering.
//!
//! ℚ is a totally ordered field. Because every `Rational` is held in canonical form, equality is
//! structural — no cross-multiplication is needed, and `PartialEq` cannot disagree with `Ord`.
//!
//! Ordering is exact and cannot overflow. It is computed by the continued-fraction descent
//! described on [`Ord::cmp`], which compares floor quotients and recurses on the remainders, and
//! so multiplies nothing at any point.

use super::{Rational, RationalScalar};
use core::cmp::Ordering;

impl<T: RationalScalar> PartialEq for Rational<T> {
    /// Canonical form makes this exact: `2/4` and `1/2` are both stored as `1/2`.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.numer() == other.numer() && self.denom() == other.denom()
    }
}

impl<T: RationalScalar> Eq for Rational<T> {}

impl<T: RationalScalar> PartialOrd for Rational<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: RationalScalar> Ord for Rational<T> {
    /// Compares two rationals exactly, by continued fraction, without ever multiplying.
    ///
    /// The obvious implementation compares `a/b` against `c/d` as `a·d` against `c·b`. It is
    /// wrong for large scalars, and wrong in the worst way: `Rational::new(i64::MAX, 1)` against
    /// `Rational::new(1, 2)` forms `i64::MAX · 2`, which panics in debug and, in release, wraps to
    /// `-2` and **reverses the answer** — reporting that `i64::MAX` is the smaller of the two.
    /// A comparison that silently inverts corrupts every `sort`, `max`, and binary search built
    /// on it, and `Ord::cmp` returns an `Ordering` with nowhere to report a failure.
    ///
    /// So this compares the way Euclid would. Write each side as an integer part plus a proper
    /// fraction:
    ///
    /// ```text
    /// a/b = q₁ + r₁/b     with 0 ≤ r₁ < b
    /// c/d = q₂ + r₂/d     with 0 ≤ r₂ < d
    /// ```
    ///
    /// If the floor quotients differ, they settle the comparison — and each is no larger in
    /// magnitude than the numerator it came from, so forming them is safe. If they agree, the
    /// comparison reduces to `r₁/b` against `r₂/d`, and inverting both proper fractions turns
    /// that into `d/r₂` against `b/r₁`, which is the same shape one rung down. Each step strictly
    /// decreases both denominators, so the descent terminates.
    ///
    /// Both denominators are strictly positive throughout — invariant 1 for the first step, and
    /// the range of a Euclidean remainder for every step after — which is what lets the quotients
    /// be compared directly.
    ///
    /// The cost is `O(log)` steps of division rather than two multiplications. For the common
    /// case of a shared denominator there is a fast path that compares numerators directly.
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        // Equal denominators — including the two-integers case, where both are one.
        if self.den == other.den {
            return self.num.cmp(&other.num);
        }

        let (mut a, mut b) = (self.num, self.den);
        let (mut c, mut d) = (other.num, other.den);

        loop {
            // `b` and `d` are strictly positive, so neither division can overflow.
            let q_lhs = a.div_euclid(b);
            let q_rhs = c.div_euclid(d);
            if q_lhs != q_rhs {
                return q_lhs.cmp(&q_rhs);
            }

            // Same integer part; the proper fractions decide. Euclidean remainders are
            // non-negative, and strictly smaller than the divisor.
            let r_lhs = a.rem_euclid(b);
            let r_rhs = c.rem_euclid(d);
            if r_lhs.is_zero() {
                return if r_rhs.is_zero() {
                    Ordering::Equal
                } else {
                    Ordering::Less
                };
            }
            if r_rhs.is_zero() {
                return Ordering::Greater;
            }

            // r₁/b ⋛ r₂/d  ⟺  d/r₂ ⋛ b/r₁, and both new denominators have strictly decreased.
            let (next_a, next_b, next_c, next_d) = (d, r_rhs, b, r_lhs);
            a = next_a;
            b = next_b;
            c = next_c;
            d = next_d;
        }
    }
}
