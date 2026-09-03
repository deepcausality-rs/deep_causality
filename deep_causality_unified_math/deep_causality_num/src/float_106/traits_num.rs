/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Numeric trait implementations for `DoubleFloat`.

use crate::Float106;
use crate::{FromPrimitive, NumCast, ToPrimitive};

// =============================================================================
// ToPrimitive
// =============================================================================

impl Float106 {
    /// The integer part of the value as an `i128`, or `None` beyond `i128`'s range.
    ///
    /// The value is truncated as a whole before its halves are read, so a value such as
    /// `3 − 10⁻¹⁹`, held as `hi = 3` and a negative `lo`, truncates to 2 and not to 3. The low
    /// component is what makes the conversion exact past 2⁵³, where an `f64` alone would round.
    fn integer_part_i128(&self) -> Option<i128> {
        let min = Self::from_i128_exact(i128::MIN);
        let max = Self::from_i128_exact(i128::MAX);
        if *self < min || *self > max {
            return None;
        }
        let t = <Self as crate::Float>::trunc(*self);
        if t == min {
            return Some(i128::MIN);
        }
        if t == max {
            return Some(i128::MAX);
        }
        t.hi.to_i128()?.checked_add(t.lo.to_i128()?)
    }

    fn integer_part_u128(&self) -> Option<u128> {
        let max = Self::from_u128_exact(u128::MAX);
        if *self < <Self as From<f64>>::from(0.0) || *self > max {
            return None;
        }
        let t = <Self as crate::Float>::trunc(*self);
        if t == max {
            return Some(u128::MAX);
        }
        let integer = t.hi.to_i128()?.checked_add(t.lo.to_i128()?)?;
        u128::try_from(integer).ok()
    }

    /// An integer as a `Float106`, exact while the remainder past the high half fits an `f64`,
    /// which every `u64` and `i64` does and every `u128` below 2¹⁰⁶ does.
    fn from_u128_exact(n: u128) -> Self {
        let hi = n as f64;
        // `hi` is `n` rounded to the nearest double; the remainder is small and may be negative.
        // Rounding up to 2¹²⁸ leaves nothing to subtract modulo 2¹²⁸, which is what the wrapping
        // subtraction wants, so that case reads as zero.
        let hi_mod = if hi >= 3.402_823_669_209_385e38 {
            0u128
        } else {
            hi as u128
        };
        let remainder = n.wrapping_sub(hi_mod) as i128;
        Self {
            hi,
            lo: remainder as f64,
        }
    }

    fn from_i128_exact(n: i128) -> Self {
        if n >= 0 {
            Self::from_u128_exact(n as u128)
        } else {
            -Self::from_u128_exact(n.unsigned_abs())
        }
    }
}

impl ToPrimitive for Float106 {
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        isize::try_from(self.integer_part_i128()?).ok()
    }

    #[inline]
    fn to_i8(&self) -> Option<i8> {
        i8::try_from(self.integer_part_i128()?).ok()
    }

    #[inline]
    fn to_i16(&self) -> Option<i16> {
        i16::try_from(self.integer_part_i128()?).ok()
    }

    #[inline]
    fn to_i32(&self) -> Option<i32> {
        i32::try_from(self.integer_part_i128()?).ok()
    }

    #[inline]
    fn to_i64(&self) -> Option<i64> {
        i64::try_from(self.integer_part_i128()?).ok()
    }

    #[inline]
    fn to_i128(&self) -> Option<i128> {
        self.integer_part_i128()
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        usize::try_from(self.integer_part_u128()?).ok()
    }

    #[inline]
    fn to_u8(&self) -> Option<u8> {
        u8::try_from(self.integer_part_u128()?).ok()
    }

    #[inline]
    fn to_u16(&self) -> Option<u16> {
        u16::try_from(self.integer_part_u128()?).ok()
    }

    #[inline]
    fn to_u32(&self) -> Option<u32> {
        u32::try_from(self.integer_part_u128()?).ok()
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        u64::try_from(self.integer_part_u128()?).ok()
    }

    #[inline]
    fn to_u128(&self) -> Option<u128> {
        self.integer_part_u128()
    }

    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some((self.hi + self.lo) as f32)
    }

    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(self.hi + self.lo)
    }
}

// =============================================================================
// FromPrimitive
// =============================================================================

impl FromPrimitive for Float106 {
    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        Some(Self::from_i128_exact(n as i128))
    }

    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        Some(Self::from_u128_exact(n as u128))
    }

    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        Some(Self::from_i128_exact(n as i128))
    }

    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        Some(Self::from_i128_exact(n as i128))
    }

    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        Some(Self::from_i128_exact(n as i128))
    }

    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        Some(Self::from_i128_exact(n as i128))
    }

    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        Some(Self::from_i128_exact(n))
    }

    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        Some(Self::from_u128_exact(n as u128))
    }

    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        Some(Self::from_u128_exact(n as u128))
    }

    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        Some(Self::from_u128_exact(n as u128))
    }

    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        Some(Self::from_u128_exact(n as u128))
    }

    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        Some(Self::from_u128_exact(n))
    }

    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        Some(<Self as From<f64>>::from(n as f64))
    }

    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        Some(<Self as From<f64>>::from(n))
    }
}

// =============================================================================
// NumCast
// =============================================================================

impl NumCast for Float106 {
    #[inline]
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        n.to_f64().map(<Self as From<f64>>::from)
    }
}

// =============================================================================
// Sum and Product
// =============================================================================

use core::iter::{Product, Sum};

impl Sum for Float106 {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(<Self as From<f64>>::from(0.0), |acc, x| acc + x)
    }
}

impl<'a> Sum<&'a Float106> for Float106 {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(<Self as From<f64>>::from(0.0), |acc, x| acc + *x)
    }
}

impl Product for Float106 {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(<Self as From<f64>>::from(1.0), |acc, x| acc * x)
    }
}

impl<'a> Product<&'a Float106> for Float106 {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(<Self as From<f64>>::from(1.0), |acc, x| acc * *x)
    }
}
