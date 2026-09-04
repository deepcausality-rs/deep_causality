/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Float` for `BFloat16`.
//!
//! Operations that IEEE 754 requires to be correctly rounded — `sqrt` and the reciprocal — run
//! in `f32`, where a double rounding is harmless (`24 >= 2 · 8 + 2`, Figueroa 1995). The
//! integer-rounding functions run in `f32` as well: every `BFloat16` at or above 2⁷ is already an
//! integer, and below it the integer part has at most 8 bits, so their results are exact. The
//! transcendental functions run in `f64` through that type's `Float` implementation, which routes
//! to `std` or `libm` as the crate features decide, and round once at the end. Fused multiply-add
//! is the one operation whose correct rounding needs more than a wide intermediate; see
//! [`Float::mul_add`] below.

use crate::float_106::two_sum;
use crate::{BFloat16, Float};
use core::num::FpCategory;

impl Float for BFloat16 {
    #[inline]
    fn nan() -> Self {
        Self::NAN
    }

    #[inline]
    fn infinity() -> Self {
        Self::INFINITY
    }

    #[inline]
    fn neg_infinity() -> Self {
        Self::NEG_INFINITY
    }

    #[inline]
    fn neg_zero() -> Self {
        Self::NEG_ZERO
    }

    #[inline]
    fn min_value() -> Self {
        Self::MIN
    }

    #[inline]
    fn min_positive_value() -> Self {
        Self::MIN_POSITIVE
    }

    #[inline]
    fn epsilon() -> Self {
        Self::EPSILON
    }

    #[inline]
    fn pi() -> Self {
        Self::PI
    }

    #[inline]
    fn e() -> Self {
        Self::E
    }

    #[inline]
    fn max_value() -> Self {
        Self::MAX
    }

    #[inline]
    fn is_nan(self) -> bool {
        BFloat16::is_nan(self)
    }

    #[inline]
    fn is_infinite(self) -> bool {
        BFloat16::is_infinite(self)
    }

    #[inline]
    fn is_finite(self) -> bool {
        BFloat16::is_finite(self)
    }

    #[inline]
    fn is_normal(self) -> bool {
        self.to_f32().is_normal()
    }

    #[inline]
    fn is_subnormal(self) -> bool {
        self.to_f32().is_subnormal()
    }

    #[inline]
    fn classify(self) -> FpCategory {
        self.to_f32().classify()
    }

    #[inline]
    fn floor(self) -> Self {
        Self::round_from_f32(Float::floor(self.to_f32()))
    }

    #[inline]
    fn ceil(self) -> Self {
        Self::round_from_f32(Float::ceil(self.to_f32()))
    }

    #[inline]
    fn round(self) -> Self {
        Self::round_from_f32(Float::round(self.to_f32()))
    }

    #[inline]
    fn trunc(self) -> Self {
        Self::round_from_f32(Float::trunc(self.to_f32()))
    }

    #[inline]
    fn fract(self) -> Self {
        Self::round_from_f32(Float::fract(self.to_f32()))
    }

    /// Clears the sign bit, so a NaN stays a NaN.
    #[inline]
    fn abs(self) -> Self {
        Self::from_bits(self.to_bits() & 0x7FFF)
    }

    #[inline]
    fn signum(self) -> Self {
        Self::round_from_f32(Float::signum(self.to_f32()))
    }

    #[inline]
    fn is_sign_positive(self) -> bool {
        BFloat16::is_sign_positive(self)
    }

    #[inline]
    fn is_sign_negative(self) -> bool {
        BFloat16::is_sign_negative(self)
    }

    /// `self · a + b` with one rounding.
    ///
    /// The product of two 8-bit significands has 16 and is exact in `f64`. Knuth's `two_sum` then
    /// gives the `f64` sum and its exact error term. When the error is not zero, the sum is
    /// nudged to the neighbouring `f64` in the error's direction if its significand is even,
    /// which is round to odd; the final `round_from_f64` performs its own round to odd into `f32`
    /// and rounds to nearest even from there. Round to odd composes, and rounding to nearest
    /// after it is correct with two spare bits (Boldo and Melquiond, 2008), so the result is the
    /// correctly rounded exact `self · a + b`. Computing in `f32` or `f64` and rounding would
    /// not be: a product that lands exactly on a `bf16` tie, with an addend too small to survive
    /// the intermediate, would take the tie rule instead of the addend's direction.
    fn mul_add(self, a: Self, b: Self) -> Self {
        let product = self.to_f64() * a.to_f64();
        let (sum, error) = two_sum(product, b.to_f64());
        if !sum.is_finite() || error == 0.0 {
            return Self::round_from_f64(sum);
        }
        let bits = sum.to_bits();
        if bits & 1 == 1 {
            return Self::round_from_f64(sum);
        }
        // `error != 0` rules out `sum == 0`, so the sign comparison is meaningful.
        let toward_larger_magnitude = (error > 0.0) == (sum > 0.0);
        let odd = if toward_larger_magnitude {
            f64::from_bits(bits + 1)
        } else {
            f64::from_bits(bits - 1)
        };
        Self::round_from_f64(odd)
    }

    #[inline]
    fn recip(self) -> Self {
        Self::round_from_f32(1.0 / self.to_f32())
    }

    #[inline]
    fn powi(self, n: i32) -> Self {
        Self::round_from_f64(Float::powi(self.to_f64(), n))
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        Self::round_from_f64(Float::powf(self.to_f64(), n.to_f64()))
    }

    #[inline]
    fn sqrt(self) -> Self {
        Self::round_from_f32(Float::sqrt(self.to_f32()))
    }

    #[inline]
    fn exp(self) -> Self {
        Self::round_from_f64(Float::exp(self.to_f64()))
    }

    #[inline]
    fn exp2(self) -> Self {
        Self::round_from_f64(Float::exp2(self.to_f64()))
    }

    #[inline]
    fn ln(self) -> Self {
        Self::round_from_f64(Float::ln(self.to_f64()))
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        Self::round_from_f64(Float::log(self.to_f64(), base.to_f64()))
    }

    #[inline]
    fn log2(self) -> Self {
        Self::round_from_f64(Float::log2(self.to_f64()))
    }

    #[inline]
    fn log10(self) -> Self {
        Self::round_from_f64(Float::log10(self.to_f64()))
    }

    #[inline]
    fn to_degrees(self) -> Self {
        Self::round_from_f64(Float::to_degrees(self.to_f64()))
    }

    #[inline]
    fn to_radians(self) -> Self {
        Self::round_from_f64(Float::to_radians(self.to_f64()))
    }

    /// IEEE 754-2008 `maxNum`: when exactly one operand is a NaN, the other is returned.
    #[inline]
    fn max(self, other: Self) -> Self {
        Self::round_from_f32(Float::max(self.to_f32(), other.to_f32()))
    }

    /// IEEE 754-2008 `minNum`: when exactly one operand is a NaN, the other is returned.
    #[inline]
    fn min(self, other: Self) -> Self {
        Self::round_from_f32(Float::min(self.to_f32(), other.to_f32()))
    }

    /// A NaN is returned unchanged, since it is neither below `min` nor above `max`.
    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        debug_assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    #[inline]
    fn cbrt(self) -> Self {
        Self::round_from_f64(Float::cbrt(self.to_f64()))
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        Self::round_from_f64(Float::hypot(self.to_f64(), other.to_f64()))
    }

    #[inline]
    fn sin(self) -> Self {
        Self::round_from_f64(Float::sin(self.to_f64()))
    }

    #[inline]
    fn cos(self) -> Self {
        Self::round_from_f64(Float::cos(self.to_f64()))
    }

    #[inline]
    fn tan(self) -> Self {
        Self::round_from_f64(Float::tan(self.to_f64()))
    }

    #[inline]
    fn asin(self) -> Self {
        Self::round_from_f64(Float::asin(self.to_f64()))
    }

    #[inline]
    fn acos(self) -> Self {
        Self::round_from_f64(Float::acos(self.to_f64()))
    }

    #[inline]
    fn atan(self) -> Self {
        Self::round_from_f64(Float::atan(self.to_f64()))
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        Self::round_from_f64(Float::atan2(self.to_f64(), other.to_f64()))
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        let (s, c) = Float::sin_cos(self.to_f64());
        (Self::round_from_f64(s), Self::round_from_f64(c))
    }

    #[inline]
    fn exp_m1(self) -> Self {
        Self::round_from_f64(Float::exp_m1(self.to_f64()))
    }

    #[inline]
    fn ln_1p(self) -> Self {
        Self::round_from_f64(Float::ln_1p(self.to_f64()))
    }

    #[inline]
    fn sinh(self) -> Self {
        Self::round_from_f64(Float::sinh(self.to_f64()))
    }

    #[inline]
    fn cosh(self) -> Self {
        Self::round_from_f64(Float::cosh(self.to_f64()))
    }

    #[inline]
    fn tanh(self) -> Self {
        Self::round_from_f64(Float::tanh(self.to_f64()))
    }

    #[inline]
    fn asinh(self) -> Self {
        Self::round_from_f64(Float::asinh(self.to_f64()))
    }

    #[inline]
    fn acosh(self) -> Self {
        Self::round_from_f64(Float::acosh(self.to_f64()))
    }

    #[inline]
    fn atanh(self) -> Self {
        Self::round_from_f64(Float::atanh(self.to_f64()))
    }

    /// The `f32` convention at this width: the significand carries the implicit bit for a normal
    /// value and is shifted up one place for a subnormal, and the exponent is biased by
    /// `127 + 7` so that `sign · mantissa · 2^exponent` is the value.
    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        let bits = self.to_bits();
        let sign: i8 = if bits & 0x8000 == 0 { 1 } else { -1 };
        let mut exponent: i16 = ((bits >> 7) & 0xFF) as i16;
        let mantissa = if exponent == 0 {
            (bits & 0x7F) << 1
        } else {
            (bits & 0x7F) | 0x80
        };
        exponent -= 127 + 7;
        (mantissa as u64, exponent, sign)
    }

    /// The magnitude of `self` with the sign bit of `sign`; a NaN keeps its payload.
    #[inline]
    fn copysign(self, sign: Self) -> Self {
        Self::from_bits((self.to_bits() & 0x7FFF) | (sign.to_bits() & 0x8000))
    }
}
