/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic operators for `BFloat16`.
//!
//! Each operator widens both operands to `f32`, which is exact, computes there, and rounds the
//! result once with [`BFloat16::round_from_f32`]. The `f32` operation has already rounded to 24
//! bits, and that intermediate rounding cannot change the final 8-bit result for `+`, `-`, `*`
//! and `/` because `24 >= 2 · 8 + 2` (Figueroa, 1995; see the module documentation). The
//! remainder is exact in `f32` and representable in `BFloat16`: it is a multiple of the divisor's
//! ulp and smaller than the divisor in magnitude, so it fits in the divisor's own 8 bits.

use crate::BFloat16;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

// =============================================================================
// Negation
// =============================================================================

impl Neg for BFloat16 {
    type Output = Self;

    /// Flips the sign bit and nothing else, so `-NaN` is a NaN with the opposite sign and
    /// `-0.0` is the negative zero.
    #[inline]
    fn neg(self) -> Self::Output {
        Self::from_bits(self.to_bits() ^ 0x8000)
    }
}

// =============================================================================
// Addition
// =============================================================================

impl Add for BFloat16 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::round_from_f32(self.to_f32() + rhs.to_f32())
    }
}

impl AddAssign for BFloat16 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

// =============================================================================
// Subtraction
// =============================================================================

impl Sub for BFloat16 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self::round_from_f32(self.to_f32() - rhs.to_f32())
    }
}

impl SubAssign for BFloat16 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

// =============================================================================
// Multiplication
// =============================================================================

impl Mul for BFloat16 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::round_from_f32(self.to_f32() * rhs.to_f32())
    }
}

impl MulAssign for BFloat16 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

// =============================================================================
// Division
// =============================================================================

impl Div for BFloat16 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self::round_from_f32(self.to_f32() / rhs.to_f32())
    }
}

impl DivAssign for BFloat16 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

// =============================================================================
// Remainder
// =============================================================================

impl Rem for BFloat16 {
    type Output = Self;

    /// The IEEE remainder with the sign of the dividend, as `f32`'s `%`.
    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Self::round_from_f32(self.to_f32() % rhs.to_f32())
    }
}

impl RemAssign for BFloat16 {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

// =============================================================================
// Reference operations
// =============================================================================

impl Add<&BFloat16> for BFloat16 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &BFloat16) -> Self::Output {
        self + *rhs
    }
}

impl Add<BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn add(self, rhs: BFloat16) -> Self::Output {
        *self + rhs
    }
}

impl Add<&BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn add(self, rhs: &BFloat16) -> Self::Output {
        *self + *rhs
    }
}

impl Sub<&BFloat16> for BFloat16 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &BFloat16) -> Self::Output {
        self - *rhs
    }
}

impl Sub<BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn sub(self, rhs: BFloat16) -> Self::Output {
        *self - rhs
    }
}

impl Sub<&BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn sub(self, rhs: &BFloat16) -> Self::Output {
        *self - *rhs
    }
}

impl Mul<&BFloat16> for BFloat16 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &BFloat16) -> Self::Output {
        self * *rhs
    }
}

impl Mul<BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn mul(self, rhs: BFloat16) -> Self::Output {
        *self * rhs
    }
}

impl Mul<&BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn mul(self, rhs: &BFloat16) -> Self::Output {
        *self * *rhs
    }
}

impl Div<&BFloat16> for BFloat16 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &BFloat16) -> Self::Output {
        self / *rhs
    }
}

impl Div<BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn div(self, rhs: BFloat16) -> Self::Output {
        *self / rhs
    }
}

impl Div<&BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn div(self, rhs: &BFloat16) -> Self::Output {
        *self / *rhs
    }
}

impl Rem<&BFloat16> for BFloat16 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: &BFloat16) -> Self::Output {
        self % *rhs
    }
}

impl Rem<BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn rem(self, rhs: BFloat16) -> Self::Output {
        *self % rhs
    }
}

impl Rem<&BFloat16> for &BFloat16 {
    type Output = BFloat16;

    #[inline]
    fn rem(self, rhs: &BFloat16) -> Self::Output {
        *self % *rhs
    }
}
