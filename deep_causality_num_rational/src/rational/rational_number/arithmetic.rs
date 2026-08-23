/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Field arithmetic on `Rational<T>`.
//!
//! Two devices keep fixed-width rationals usable, and both are about *not forming* a quantity
//! larger than the answer needs.
//!
//! The first is cancelling common factors **before** multiplying rather than after. The naive
//! `a/b + c/d = (a·d + c·b)/(b·d)` overflows far sooner than it needs to, because `b·d` is formed
//! even when `b` and `d` share a factor; multiplication has the mirror problem, forming `a·c/b·d`
//! when `a` and `d` share one. Reducing first costs one extra `gcd` per operation.
//!
//! The second applies to addition, and is about the numerator rather than the denominator.
//! Splitting each operand into an integer part and a proper fraction, `a/b = q + r/b`, means the
//! large part of each value is carried as an integer and never multiplied out: `MAX/2 + MAX/2` is
//! exactly `MAX`, where the direct numerator sum `MAX + MAX` does not fit. The fractional
//! numerators that remain are each smaller than their denominator, so what is added is bounded by
//! the least common denominator rather than by the operands.
//!
//! Neither device makes the arithmetic total — see the `Overflow` section on
//! [`Rational`](super::Rational) for exactly what is left. Negation is the one operation here that
//! *is* total, and it is total by construction rather than by cleverness: invariant 4 keeps
//! `T::MIN` out of the numerator.

use super::{Rational, RationalScalar};
use crate::{One, Zero};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl<T: RationalScalar> Add for Rational<T> {
    type Output = Self;

    /// Adds two rationals exactly, carrying the integer part separately from the fraction.
    ///
    /// # Overflow
    ///
    /// Overflows when `lcm(b, d)` does not fit in `T`, when the sum of the two integer parts does
    /// not, or when the combined fractional numerator — bounded by `2·lcm(b, d)` — does not.
    /// Denominators near `√T::MAX` with no common factor will still overflow; at that point the
    /// exact answer needs a denominator wider than `T`, and no arrangement of the arithmetic can
    /// recover it.
    #[inline]
    fn add(self, rhs: Self) -> Self {
        // Split each operand: a/b = q + r/b with 0 ≤ r < b. Both quotients are bounded in
        // magnitude by their numerators, and the denominators are positive, so neither division
        // can overflow.
        let q_lhs = self.num.div_euclid(self.den);
        let r_lhs = self.num.rem_euclid(self.den);
        let q_rhs = rhs.num.div_euclid(rhs.den);
        let r_rhs = rhs.num.rem_euclid(rhs.den);

        // Add the proper fractions over the least common denominator b·(d/g), never over b·d.
        let g = self.den.gcd(&rhs.den);
        let d_reduced = rhs.den / g;
        let mut frac_num = r_lhs * d_reduced + r_rhs * (self.den / g);
        let frac_den = self.den * d_reduced;

        // Each proper fraction is strictly below one, so their sum is below two and carries at
        // most a single unit into the integer part.
        let mut int_part = q_lhs + q_rhs;
        if frac_num >= frac_den {
            frac_num = frac_num - frac_den;
            int_part = int_part + T::one();
        }

        // Reduce the fraction *before* re-attaching the integer part, so the multiply below is by
        // the smallest denominator the result admits. `gcd(0, frac_den) == frac_den`, so a zero
        // fractional part collapses to `0/1` and the multiply disappears entirely — which is what
        // makes `MAX/2 + MAX/2` come out as `MAX/1`.
        let g_frac = frac_num.gcd(&frac_den);
        let frac_num = frac_num / g_frac;
        let frac_den = frac_den / g_frac;

        // n = int_part·frac_den + frac_num. Since 0 ≤ frac_num < frac_den, the product is on the
        // same side of `n` as `int_part`'s sign, so choosing the form by that sign keeps the
        // intermediate inside the range whenever `n` itself is.
        let num = if int_part < T::zero() {
            (int_part + T::one()) * frac_den - (frac_den - frac_num)
        } else {
            int_part * frac_den + frac_num
        };

        // `frac_den` is coprime to `num` by construction, so this reduction only enforces the
        // invariants; it has nothing left to cancel.
        Self::reduce(num, frac_den)
    }
}

impl<T: RationalScalar> Sub for Rational<T> {
    type Output = Self;

    /// Subtracts by adding the negation, which is total — see [`Neg`].
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl<T: RationalScalar> Mul for Rational<T> {
    type Output = Self;

    /// # Overflow
    ///
    /// Overflows when the product of the two cross-cancelled numerators, or of the two
    /// cross-cancelled denominators, does not fit in `T`.
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        // Cross-cancel: each numerator against the *other* denominator. Both products are then
        // formed from already-coprime factors.
        let g1 = self.num.gcd(&rhs.den);
        let g2 = rhs.num.gcd(&self.den);
        let num = (self.num / g1) * (rhs.num / g2);
        let den = (self.den / g2) * (rhs.den / g1);
        Self::reduce(num, den)
    }
}

impl<T: RationalScalar> Div for Rational<T> {
    type Output = Self;

    /// # Panics
    ///
    /// Panics if `rhs` is zero, which has no inverse in a field.
    ///
    /// # Overflow
    ///
    /// The same bound as [`Mul`], applied to `self · rhs⁻¹`.
    #[inline]
    fn div(self, rhs: Self) -> Self {
        if rhs.num.is_zero() {
            panic!("Rational division by zero");
        }
        // Multiply by the reciprocal, with the same cross-cancellation.
        let g1 = self.num.gcd(&rhs.num);
        let g2 = rhs.den.gcd(&self.den);
        let num = (self.num / g1) * (rhs.den / g2);
        let den = (self.den / g2) * (rhs.num / g1);
        Self::reduce(num, den)
    }
}

impl<T: RationalScalar> Neg for Rational<T> {
    type Output = Self;

    /// Negates, and cannot fail.
    ///
    /// Negating the numerator preserves every invariant: the denominator is untouched and still
    /// positive, `gcd(-n, d) == gcd(n, d) == 1`, and `-n` is never `T::MIN` because `n` never is.
    ///
    /// That last clause is the whole reason invariant 4 exists. The alternative was to leave
    /// `T::MIN` constructible and offer a `checked_neg` beside a `Neg` that panics on it — but
    /// `Neg` is not an isolated operation here. `Sub` is `self + (-rhs)`, `AbelianGroup` claims
    /// `a + (-a) = 0` for every `a`, and a partial negation would have made both of those partial
    /// too, in a way no signature admits. Refusing one value per width at construction, where
    /// there *is* a signature to say so, buys a negation that is total everywhere else.
    #[inline]
    fn neg(self) -> Self {
        Self {
            num: -self.num,
            den: self.den,
        }
    }
}

impl<T: RationalScalar> AddAssign for Rational<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: RationalScalar> SubAssign for Rational<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: RationalScalar> MulAssign for Rational<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: RationalScalar> DivAssign for Rational<T> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<T: RationalScalar> Sum for Rational<T> {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, x| acc + x)
    }
}

impl<T: RationalScalar> Product for Rational<T> {
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::one(), |acc, x| acc * x)
    }
}
