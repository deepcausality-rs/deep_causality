/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float;
use std::num::FpCategory;

// F32 Helper functions for integer_decode
fn integer_decode_f32(f: f32) -> (u64, i16, i8) {
    let bits: u32 = f.to_bits();
    let sign: i8 = if bits >> 31 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 23) & 0xff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0x7fffff) << 1
    } else {
        (bits & 0x7fffff) | 0x800000
    };
    // Exponent bias + mantissa shift
    exponent -= 127 + 23;
    (mantissa as u64, exponent, sign)
}

impl Float for f32 {
    #[inline]
    fn nan() -> Self {
        f32::NAN
    }
    #[inline]
    fn infinity() -> Self {
        f32::INFINITY
    }
    #[inline]
    fn neg_infinity() -> Self {
        f32::NEG_INFINITY
    }
    #[inline]
    fn neg_zero() -> Self {
        -0.0
    }
    #[inline]
    fn min_value() -> Self {
        f32::MIN
    }
    #[inline]
    fn min_positive_value() -> Self {
        f32::MIN_POSITIVE
    }
    #[inline]
    fn epsilon() -> Self {
        f32::EPSILON
    }
    #[inline]
    fn max_value() -> Self {
        f32::MAX
    }

    #[inline]
    fn is_nan(self) -> bool {
        f32::is_nan(self)
    }

    #[inline]
    fn is_infinite(self) -> bool {
        f32::is_infinite(self)
    }

    #[inline]
    fn is_finite(self) -> bool {
        f32::is_finite(self)
    }
    #[inline]
    fn is_normal(self) -> bool {
        f32::is_normal(self)
    }
    #[inline]
    fn is_subnormal(self) -> bool {
        f32::is_subnormal(self)
    }
    #[inline]
    fn classify(self) -> FpCategory {
        f32::classify(self)
    }
    #[inline]
    fn floor(self) -> Self {
        f32::floor(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        f32::ceil(self)
    }
    #[inline]
    fn round(self) -> Self {
        f32::round(self)
    }
    #[inline]
    fn trunc(self) -> Self {
        f32::trunc(self)
    }
    #[inline]
    fn fract(self) -> Self {
        f32::fract(self)
    }
    #[inline]
    fn abs(self) -> Self {
        f32::abs(self)
    }
    #[inline]
    fn signum(self) -> Self {
        f32::signum(self)
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        f32::is_sign_positive(self)
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        f32::is_sign_negative(self)
    }
    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        f32::mul_add(self, a, b)
    }
    #[inline]
    fn recip(self) -> Self {
        f32::recip(self)
    }
    #[inline]
    fn powi(self, n: i32) -> Self {
        f32::powi(self, n)
    }
    #[inline]
    fn powf(self, n: Self) -> Self {
        f32::powf(self, n)
    }
    #[inline]
    fn sqrt(self) -> Self {
        f32::sqrt(self)
    }
    #[inline]
    fn exp(self) -> Self {
        f32::exp(self)
    }
    #[inline]
    fn exp2(self) -> Self {
        f32::exp2(self)
    }
    #[inline]
    fn ln(self) -> Self {
        f32::ln(self)
    }
    #[inline]
    fn log(self, base: Self) -> Self {
        f32::log(self, base)
    }
    #[inline]
    fn log2(self) -> Self {
        f32::log2(self)
    }
    #[inline]
    fn log10(self) -> Self {
        f32::log10(self)
    }
    #[inline]
    fn to_degrees(self) -> Self {
        f32::to_degrees(self)
    }
    #[inline]
    fn to_radians(self) -> Self {
        f32::to_radians(self)
    }
    #[inline]
    fn max(self, other: Self) -> Self {
        f32::max(self, other)
    }
    #[inline]
    fn min(self, other: Self) -> Self {
        f32::min(self, other)
    }
    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        f32::clamp(self, min, max)
    }
    #[inline]
    fn cbrt(self) -> Self {
        f32::cbrt(self)
    }
    #[inline]
    fn hypot(self, other: Self) -> Self {
        f32::hypot(self, other)
    }
    #[inline]
    fn sin(self) -> Self {
        f32::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        f32::cos(self)
    }
    #[inline]
    fn tan(self) -> Self {
        f32::tan(self)
    }
    #[inline]
    fn asin(self) -> Self {
        f32::asin(self)
    }
    #[inline]
    fn acos(self) -> Self {
        f32::acos(self)
    }
    #[inline]
    fn atan(self) -> Self {
        f32::atan(self)
    }
    #[inline]
    fn atan2(self, other: Self) -> Self {
        f32::atan2(self, other)
    }
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        f32::sin_cos(self)
    }
    #[inline]
    fn exp_m1(self) -> Self {
        f32::exp_m1(self)
    }
    #[inline]
    fn ln_1p(self) -> Self {
        f32::ln_1p(self)
    }
    #[inline]
    fn sinh(self) -> Self {
        f32::sinh(self)
    }
    #[inline]
    fn cosh(self) -> Self {
        f32::cosh(self)
    }
    #[inline]
    fn tanh(self) -> Self {
        f32::tanh(self)
    }
    #[inline]
    fn asinh(self) -> Self {
        f32::asinh(self)
    }
    #[inline]
    fn acosh(self) -> Self {
        f32::acosh(self)
    }
    #[inline]
    fn atanh(self) -> Self {
        f32::atanh(self)
    }
    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        integer_decode_f32(self)
    }
    #[inline]
    fn copysign(self, sign: Self) -> Self {
        f32::copysign(self, sign)
    }
}
