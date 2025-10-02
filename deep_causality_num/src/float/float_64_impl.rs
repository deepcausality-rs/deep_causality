/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float;
use std::num::FpCategory;

fn integer_decode_f64(f: f64) -> (u64, i16, i8) {
    let bits: u64 = f.to_bits();
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };
    // Exponent bias + mantissa shift
    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

impl Float for f64 {
    #[inline]
    fn nan() -> Self {
        f64::NAN
    }
    #[inline]
    fn infinity() -> Self {
        f64::INFINITY
    }
    #[inline]
    fn neg_infinity() -> Self {
        f64::NEG_INFINITY
    }
    #[inline]
    fn neg_zero() -> Self {
        -0.0
    }
    #[inline]
    fn min_value() -> Self {
        f64::MIN
    }
    #[inline]
    fn min_positive_value() -> Self {
        f64::MIN_POSITIVE
    }
    #[inline]
    fn epsilon() -> Self {
        f64::EPSILON
    }
    #[inline]
    fn max_value() -> Self {
        f64::MAX
    }

    #[inline]
    fn is_nan(self) -> bool {
        f64::is_nan(self)
    }

    #[inline]
    fn is_infinite(self) -> bool {
        f64::is_infinite(self)
    }

    #[inline]
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
    #[inline]
    fn is_normal(self) -> bool {
        f64::is_normal(self)
    }
    #[inline]
    fn is_subnormal(self) -> bool {
        f64::is_subnormal(self)
    }
    #[inline]
    fn classify(self) -> FpCategory {
        f64::classify(self)
    }
    #[inline]
    fn floor(self) -> Self {
        f64::floor(self)
    }
    #[inline]
    fn ceil(self) -> Self {
        f64::ceil(self)
    }
    #[inline]
    fn round(self) -> Self {
        f64::round(self)
    }
    #[inline]
    fn trunc(self) -> Self {
        f64::trunc(self)
    }
    #[inline]
    fn fract(self) -> Self {
        f64::fract(self)
    }
    #[inline]
    fn abs(self) -> Self {
        f64::abs(self)
    }
    #[inline]
    fn signum(self) -> Self {
        f64::signum(self)
    }
    #[inline]
    fn is_sign_positive(self) -> bool {
        f64::is_sign_positive(self)
    }
    #[inline]
    fn is_sign_negative(self) -> bool {
        f64::is_sign_negative(self)
    }
    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        f64::mul_add(self, a, b)
    }
    #[inline]
    fn recip(self) -> Self {
        f64::recip(self)
    }
    #[inline]
    fn powi(self, n: i32) -> Self {
        f64::powi(self, n)
    }
    #[inline]
    fn powf(self, n: Self) -> Self {
        f64::powf(self, n)
    }
    #[inline]
    fn sqrt(self) -> Self {
        f64::sqrt(self)
    }
    #[inline]
    fn exp(self) -> Self {
        f64::exp(self)
    }
    #[inline]
    fn exp2(self) -> Self {
        f64::exp2(self)
    }
    #[inline]
    fn ln(self) -> Self {
        f64::ln(self)
    }
    #[inline]
    fn log(self, base: Self) -> Self {
        f64::log(self, base)
    }
    #[inline]
    fn log2(self) -> Self {
        f64::log2(self)
    }
    #[inline]
    fn log10(self) -> Self {
        f64::log10(self)
    }
    #[inline]
    fn to_degrees(self) -> Self {
        f64::to_degrees(self)
    }
    #[inline]
    fn to_radians(self) -> Self {
        f64::to_radians(self)
    }
    #[inline]
    fn max(self, other: Self) -> Self {
        f64::max(self, other)
    }
    #[inline]
    fn min(self, other: Self) -> Self {
        f64::min(self, other)
    }
    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        f64::clamp(self, min, max)
    }
    #[inline]
    fn cbrt(self) -> Self {
        f64::cbrt(self)
    }
    #[inline]
    fn hypot(self, other: Self) -> Self {
        f64::hypot(self, other)
    }
    #[inline]
    fn sin(self) -> Self {
        f64::sin(self)
    }
    #[inline]
    fn cos(self) -> Self {
        f64::cos(self)
    }
    #[inline]
    fn tan(self) -> Self {
        f64::tan(self)
    }
    #[inline]
    fn asin(self) -> Self {
        f64::asin(self)
    }
    #[inline]
    fn acos(self) -> Self {
        f64::acos(self)
    }
    #[inline]
    fn atan(self) -> Self {
        f64::atan(self)
    }
    #[inline]
    fn atan2(self, other: Self) -> Self {
        f64::atan2(self, other)
    }
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        f64::sin_cos(self)
    }
    #[inline]
    fn exp_m1(self) -> Self {
        f64::exp_m1(self)
    }
    #[inline]
    fn ln_1p(self) -> Self {
        f64::ln_1p(self)
    }
    #[inline]
    fn sinh(self) -> Self {
        f64::sinh(self)
    }
    #[inline]
    fn cosh(self) -> Self {
        f64::cosh(self)
    }
    #[inline]
    fn tanh(self) -> Self {
        f64::tanh(self)
    }
    #[inline]
    fn asinh(self) -> Self {
        f64::asinh(self)
    }
    #[inline]
    fn acosh(self) -> Self {
        f64::acosh(self)
    }
    #[inline]
    fn atanh(self) -> Self {
        f64::atanh(self)
    }
    #[inline]
    fn integer_decode(self) -> (u64, i16, i8) {
        integer_decode_f64(self)
    }
    #[inline]
    fn copysign(self, sign: Self) -> Self {
        f64::copysign(self, sign)
    }
}
