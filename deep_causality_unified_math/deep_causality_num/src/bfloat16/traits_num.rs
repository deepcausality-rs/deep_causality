/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `ToPrimitive`, `FromPrimitive`, `NumCast`, `Sum` and `Product` for `BFloat16`.

use crate::BFloat16;
use crate::{FromPrimitive, NumCast, ToPrimitive};
use core::iter::{Product, Sum};

// =============================================================================
// Integers in
// =============================================================================

impl BFloat16 {
    /// An integer magnitude with a sign, rounded once to the nearest representable value, ties
    /// to even, directly from its bits.
    ///
    /// Going through `f64` would round twice for magnitudes past 2⁵³. Here the top eight bits
    /// of the magnitude are the significand, the bits below them decide the rounding by an exact
    /// comparison with half a unit, and a carry out of the eight bits moves the exponent up. A
    /// magnitude that rounds to 2¹²⁸ is beyond `MAX` and becomes the infinity of its sign.
    fn from_integer(magnitude: u128, negative: bool) -> Self {
        let sign = if negative { 0x8000 } else { 0x0000 };
        if magnitude == 0 {
            return Self::from_bits(sign);
        }
        let width = 128 - magnitude.leading_zeros();
        let mut exponent = width - 1;
        let mut significand = if width <= 8 {
            (magnitude as u16) << (8 - width)
        } else {
            let shift = width - 8;
            let kept = (magnitude >> shift) as u16;
            let dropped = magnitude & ((1u128 << shift) - 1);
            let half = 1u128 << (shift - 1);
            let round_up = dropped > half || (dropped == half && kept & 1 == 1);
            kept + (round_up as u16)
        };
        if significand == 0x100 {
            significand = 0x80;
            exponent += 1;
        }
        if exponent > 127 {
            return Self::from_bits(sign | 0x7F80);
        }
        let biased = (exponent + 127) as u16;
        Self::from_bits(sign | (biased << 7) | (significand & 0x7F))
    }
}

impl FromPrimitive for BFloat16 {
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        Some(Self::from_integer(n.unsigned_abs() as u128, n < 0))
    }

    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        Some(Self::from_integer(n.unsigned_abs() as u128, n < 0))
    }

    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        Some(Self::from_integer(n.unsigned_abs() as u128, n < 0))
    }

    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        Some(Self::from_integer(n.unsigned_abs() as u128, n < 0))
    }

    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        Some(Self::from_integer(n.unsigned_abs() as u128, n < 0))
    }

    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        Some(Self::from_integer(n.unsigned_abs(), n < 0))
    }

    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        Some(Self::from_integer(n as u128, false))
    }

    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        Some(Self::from_integer(n as u128, false))
    }

    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        Some(Self::from_integer(n as u128, false))
    }

    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        Some(Self::from_integer(n as u128, false))
    }

    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        Some(Self::from_integer(n as u128, false))
    }

    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        Some(Self::from_integer(n, false))
    }

    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        Some(Self::round_from_f32(n))
    }

    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        Some(Self::round_from_f64(n))
    }
}

// =============================================================================
// Primitives out
// =============================================================================

/// Every conversion widens to `f32`, which is exact, and applies `f32`'s own range checks and
/// truncation toward zero.
impl ToPrimitive for BFloat16 {
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        BFloat16::to_f32(*self).to_isize()
    }

    #[inline]
    fn to_i8(&self) -> Option<i8> {
        BFloat16::to_f32(*self).to_i8()
    }

    #[inline]
    fn to_i16(&self) -> Option<i16> {
        BFloat16::to_f32(*self).to_i16()
    }

    #[inline]
    fn to_i32(&self) -> Option<i32> {
        BFloat16::to_f32(*self).to_i32()
    }

    #[inline]
    fn to_i64(&self) -> Option<i64> {
        BFloat16::to_f32(*self).to_i64()
    }

    #[inline]
    fn to_i128(&self) -> Option<i128> {
        BFloat16::to_f32(*self).to_i128()
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        BFloat16::to_f32(*self).to_usize()
    }

    #[inline]
    fn to_u8(&self) -> Option<u8> {
        BFloat16::to_f32(*self).to_u8()
    }

    #[inline]
    fn to_u16(&self) -> Option<u16> {
        BFloat16::to_f32(*self).to_u16()
    }

    #[inline]
    fn to_u32(&self) -> Option<u32> {
        BFloat16::to_f32(*self).to_u32()
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        BFloat16::to_f32(*self).to_u64()
    }

    #[inline]
    fn to_u128(&self) -> Option<u128> {
        BFloat16::to_f32(*self).to_u128()
    }

    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some(BFloat16::to_f32(*self))
    }

    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(BFloat16::to_f64(*self))
    }
}

// =============================================================================
// NumCast
// =============================================================================

/// Goes through `f64`, the widest crossing `ToPrimitive` offers. An integer source above 2⁵³ is
/// rounded by that step before the single rounding here; `FromPrimitive` has the exact path for
/// callers that know their source is an integer.
impl NumCast for BFloat16 {
    #[inline]
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        n.to_f64().map(Self::round_from_f64)
    }
}

// =============================================================================
// Sum and Product
// =============================================================================

impl Sum for BFloat16 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + x)
    }
}

impl<'a> Sum<&'a BFloat16> for BFloat16 {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |acc, x| acc + *x)
    }
}

impl Product for BFloat16 {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |acc, x| acc * x)
    }
}

impl<'a> Product<&'a BFloat16> for BFloat16 {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |acc, x| acc * *x)
    }
}
