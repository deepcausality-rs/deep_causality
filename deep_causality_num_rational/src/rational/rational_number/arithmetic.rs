/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Field arithmetic on `Rational<T>`.
//!
//! Every operation cancels common factors **before** multiplying rather than after. The naive
//! `a/b + c/d = (a·d + c·b)/(b·d)` overflows far sooner than it needs to, because `b·d` is formed
//! even when `b` and `d` share a factor. Reducing first costs one extra `gcd` and buys a large
//! amount of headroom, which is the difference between a rational type that is usable over `i64`
//! and one that is not.

use super::{Rational, RationalScalar};
use crate::{One, Zero};
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl<T: RationalScalar> Add for Rational<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        // With g = gcd(b, d), the least common denominator is b·(d/g) rather than b·d.
        let g = self.den.gcd(&rhs.den);
        let d_reduced = rhs.den / g;
        let num = self.num * d_reduced + rhs.num * (self.den / g);
        let den = self.den * d_reduced;
        Self::reduce(num, den)
    }
}

impl<T: RationalScalar> Sub for Rational<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl<T: RationalScalar> Mul for Rational<T> {
    type Output = Self;

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

    #[inline]
    fn neg(self) -> Self {
        // Negating the numerator preserves both invariants: the denominator is untouched and
        // still positive, and gcd(-n, d) == gcd(n, d) == 1.
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
