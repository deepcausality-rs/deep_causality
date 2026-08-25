/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::gf2::Gf2;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Addition in 𝔽₂ is exclusive-or: `0+0 = 0`, `0+1 = 1`, `1+0 = 1`, `1+1 = 0`.
///
/// Written as `!=` rather than `^`. The two are the same function on `bool` — the sum is `1`
/// exactly when the operands differ — and `!=` reads as the definition rather than as a bit trick.
impl Add for Gf2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        Gf2(self.0 != rhs.0)
    }
}

impl AddAssign for Gf2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 != rhs.0;
    }
}

/// Subtraction coincides with addition, because every element is its own additive inverse.
impl Sub for Gf2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Gf2(self.0 != rhs.0)
    }
}

impl SubAssign for Gf2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 != rhs.0;
    }
}

/// Negation is the identity: `-0 = 0` and `-1 = 1`, since `1 + 1 = 0`.
impl Neg for Gf2 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        self
    }
}

/// Multiplication in 𝔽₂ is conjunction: the product is `1` exactly when both factors are.
impl Mul for Gf2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Gf2(self.0 && rhs.0)
    }
}

impl MulAssign for Gf2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0 && rhs.0;
    }
}

/// Division by the only unit, `1`, is the identity.
///
/// # Panics
///
/// Dividing by `0` panics, as it does for every other scalar in this workspace. 𝔽₂ has exactly one
/// non-zero element, so this is the whole of the partiality: `a / 1 = a`, and `a / 0` has no value.
impl Div for Gf2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        assert!(rhs.0, "division by zero in GF(2)");
        self
    }
}

impl DivAssign for Gf2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        assert!(rhs.0, "division by zero in GF(2)");
    }
}
