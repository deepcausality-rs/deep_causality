/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EuclideanDomain;
use core::ops::{Div, Neg};

mod algebra;
mod arithmetic;
mod display;
mod identity;
mod ord;

/// The scalar a [`Rational`] is built over.
///
/// Mathematically, the field of fractions exists over any integral domain. This crate builds it
/// over a [`EuclideanDomain`], because reducing a fraction to lowest terms needs a `gcd`, and the
/// Euclidean algorithm is what provides one. The remaining bounds are the operations the fraction
/// arithmetic needs and that the tower does not already imply:
///
/// - `Div` — to divide numerator and denominator through by their gcd.
/// - `Neg` — to move a negative sign out of the denominator.
/// - `PartialOrd` — to detect that sign in the first place, and to order fractions.
/// - `Copy` — the arithmetic reads each component several times per operation.
///
/// Every signed integer width satisfies this: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`. The
/// unsigned types do not, and cannot: ℕ is not a ring, so it is not a `EuclideanDomain`, and a
/// field of fractions over it would have nothing to negate.
pub trait RationalScalar:
    EuclideanDomain + Div<Output = Self> + Neg<Output = Self> + PartialOrd + Copy
{
}

impl<T> RationalScalar for T where
    T: EuclideanDomain + Div<Output = T> + Neg<Output = T> + PartialOrd + Copy
{
}

/// A rational number `n/d` — an element of the field of fractions of `T`.
///
/// `Rational<T>` is **exact**. Unlike a floating-point value, `1/3` is represented as the pair
/// `(1, 3)` and never as an approximation of it, so `(1/3) * 3` is exactly `1`.
///
/// # Invariants
///
/// Every `Rational` is kept in canonical form, which is why the fields are private:
///
/// 1. The denominator is strictly positive. A sign lives in the numerator, never the denominator.
/// 2. The numerator and denominator are coprime — `gcd(n, d) == 1`.
/// 3. Zero is exactly `0/1`.
///
/// Canonical form makes equality structural: two `Rational`s are equal exactly when their
/// components match, so `PartialEq` needs no cross-multiplication.
///
/// # Where this sits in the tower
///
/// ℚ is a [`Field`](deep_causality_algebra::Field): every non-zero rational has a multiplicative
/// inverse, `(n/d)⁻¹ = d/n`. It is **not** a [`Real`](deep_causality_algebra::Real), and that is
/// not an omission. `Real` is the analytic axis — `sqrt`, `exp`, `ln`, `sin` — and ℚ is not
/// closed under any of them: `sqrt(2)` is irrational, which is the oldest theorem about this
/// gap. ℚ is arithmetically complete and analytically empty.
///
/// This mirrors ℤ, which reaches [`CommutativeRing`](deep_causality_algebra::CommutativeRing) and
/// stops short of `Field` for the opposite reason — ℤ has no inverses. Passing from ℤ to ℚ buys
/// division and gives up nothing; passing from ℚ to ℝ buys limits and gives up exactness.
///
/// # Overflow
///
/// This is the one sharp edge, and it is inherent to fixed-width rational arithmetic rather than
/// specific to this implementation. Adding `a/b + c/d` forms `a·d + c·b` over `b·d`, so
/// denominators grow multiplicatively. The implementation reduces aggressively — it cancels
/// common factors *before* multiplying, not after — which delays the problem substantially but
/// cannot remove it. A long chain of additions with coprime denominators will exhaust any fixed
/// width.
///
/// When it does, the behaviour is `T`'s: a panic in debug builds, wrapping in release. Choose
/// `T` with headroom (`i128` where the denominators are unpredictable), and prefer accumulating
/// over a common denominator where the problem admits one.
///
/// # Examples
///
/// ```
/// use deep_causality_num_rational::Rational;
///
/// let third = Rational::new(1_i64, 3);
/// let sum = third + third + third;
/// assert_eq!(sum, Rational::from_integer(1)); // exact, unlike 0.1 + 0.2 in binary floats
///
/// // Fractions are reduced on construction.
/// assert_eq!(Rational::new(6_i64, 8), Rational::new(3, 4));
///
/// // A sign never survives in the denominator.
/// let r = Rational::new(1_i64, -2);
/// assert_eq!(*r.numer(), -1);
/// assert_eq!(*r.denom(), 2);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Rational<T: RationalScalar> {
    /// The numerator. Carries the sign of the value.
    num: T,
    /// The denominator. Always strictly positive, never zero.
    den: T,
}

impl<T: RationalScalar> Rational<T> {
    /// Constructs `num/den`, reduced to canonical form.
    ///
    /// # Panics
    ///
    /// Panics if `den` is zero. Division by zero has no value in ℚ, so there is nothing this
    /// could return. This matches the behaviour of `/` on the integers themselves. Use
    /// [`try_new`](Self::try_new) where the denominator is not known to be non-zero.
    #[inline]
    pub fn new(num: T, den: T) -> Self {
        match Self::try_new(num, den) {
            Some(r) => r,
            None => panic!("Rational::new called with a zero denominator"),
        }
    }

    /// Constructs `num/den`, reduced to canonical form, or `None` if `den` is zero.
    ///
    /// The total counterpart of [`new`](Self::new): every rational with a non-zero denominator
    /// exists, and no other does.
    #[inline]
    pub fn try_new(num: T, den: T) -> Option<Self> {
        if den.is_zero() {
            return None;
        }
        Some(Self::reduce(num, den))
    }

    /// Constructs the rational `n/1` from an integer.
    ///
    /// This is the canonical embedding ℤ ↪ ℚ, and it is injective: distinct integers give
    /// distinct rationals.
    #[inline]
    pub fn from_integer(n: T) -> Self {
        Self {
            num: n,
            den: T::one(),
        }
    }

    /// Returns the numerator, which carries the sign.
    #[inline]
    pub fn numer(&self) -> &T {
        &self.num
    }

    /// Returns the denominator, which is always strictly positive.
    #[inline]
    pub fn denom(&self) -> &T {
        &self.den
    }

    /// Returns `true` if the denominator is one, so the value is an integer.
    #[inline]
    pub fn is_integer(&self) -> bool {
        self.den == T::one()
    }

    /// Returns the multiplicative inverse `d/n`, or `None` for zero.
    ///
    /// Zero is the one element of a field with no inverse, so `None` is the honest answer rather
    /// than a panic.
    #[inline]
    pub fn checked_recip(&self) -> Option<Self> {
        if self.num.is_zero() {
            None
        } else {
            // `self.den` is positive and coprime to `self.num`, so swapping preserves invariant
            // 2 and needs only the sign fixed up, which `reduce` does.
            Some(Self::reduce(self.den, self.num))
        }
    }

    /// Returns the multiplicative inverse `d/n`.
    ///
    /// # Panics
    ///
    /// Panics if the value is zero.
    #[inline]
    pub fn recip(&self) -> Self {
        match self.checked_recip() {
            Some(r) => r,
            None => panic!("Rational::recip called on zero"),
        }
    }

    /// Reduces `num/den` to canonical form. The caller guarantees `den != 0`.
    ///
    /// Two steps: move any sign out of the denominator, then divide both components by their
    /// greatest common divisor.
    #[inline]
    pub(crate) fn reduce(num: T, den: T) -> Self {
        let (num, den) = if den < T::zero() {
            // Negating both components leaves the value unchanged and fixes invariant 1.
            (-num, -den)
        } else {
            (num, den)
        };

        // `gcd` is non-negative, and is non-zero here because `den` is non-zero. When the
        // numerator is zero this yields `gcd(0, den) == den`, so zero reduces to `0/1` and
        // invariant 3 falls out of the same step rather than needing a special case.
        let g = num.gcd(&den);
        Self {
            num: num / g,
            den: den / g,
        }
    }
}
