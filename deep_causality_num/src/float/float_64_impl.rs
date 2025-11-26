/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Float, One, Zero};
use core::num::FpCategory;

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
        let f = self.fract();
        if f.is_nan() || f.is_zero() {
            self
        } else if self < Self::zero() {
            self - f - Self::one()
        } else {
            self - f
        }
    }

    #[inline]
    fn ceil(self) -> Self {
        let f = self.fract();
        if f.is_nan() || f.is_zero() {
            self
        } else if self > Self::zero() {
            self - f + Self::one()
        } else {
            self - f
        }
    }

    #[inline]
    fn round(self) -> Self {
        (self + self.signum() * 0.5).trunc()
    }

    #[inline]
    fn trunc(self) -> Self {
        if self > 0.0 {
            self.floor()
        } else {
            self.ceil()
        }
    }

    #[cfg(feature = "std")]
    #[inline]
    fn fract(self) -> Self {
        <f64>::fract(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn fract(self) -> Self {
        self - libm::trunc(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn fract(self) -> Self {
        panic!("Floating point function 'fract' not available in no-std without libm_math feature")
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

    #[cfg(feature = "std")]
    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        <f64>::mul_add(self, a, b)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        libm::fma(self, a, b)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn mul_add(self, _a: Self, _b: Self) -> Self {
        panic!(
            "Floating point function 'mul_add' not available in no-std without libm_math feature"
        )
    }

    #[inline]
    fn recip(self) -> Self {
        f64::recip(self)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn powi(self, n: i32) -> Self {
        <f64>::powi(self, n)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn powi(self, n: i32) -> Self {
        libm::pow(self, n as f64)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn powi(self, _n: i32) -> Self {
        panic!("Floating point function 'powi' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn powf(self, n: Self) -> Self {
        <f64>::powf(self, n)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn powf(self, n: Self) -> Self {
        libm::pow(self, n)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn powf(self, _n: Self) -> Self {
        panic!("Floating point function 'powf' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sqrt(self) -> Self {
        <f64>::sqrt(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sqrt(self) -> Self {
        libm::sqrt(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sqrt(self) -> Self {
        panic!("Floating point function 'sqrt' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn exp(self) -> Self {
        <f64>::exp(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn exp(self) -> Self {
        libm::exp(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn exp(self) -> Self {
        panic!("Floating point function 'exp' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn exp2(self) -> Self {
        <f64>::exp2(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn exp2(self) -> Self {
        libm::exp2(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn exp2(self) -> Self {
        panic!("Floating point function 'exp2' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn ln(self) -> Self {
        <f64>::ln(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn ln(self) -> Self {
        libm::log(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn ln(self) -> Self {
        panic!("Floating point function 'ln' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn log(self, base: Self) -> Self {
        <f64>::log(self, base)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn log(self, base: Self) -> Self {
        libm::log(self) / libm::log(base)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn log(self, _base: Self) -> Self {
        panic!("Floating point function 'log' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn log2(self) -> Self {
        <f64>::log2(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn log2(self) -> Self {
        libm::log2(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn log2(self) -> Self {
        panic!("Floating point function 'log2' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn log10(self) -> Self {
        <f64>::log10(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn log10(self) -> Self {
        libm::log10(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn log10(self) -> Self {
        panic!("Floating point function 'log10' not available in no-std without libm_math feature")
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

    #[cfg(feature = "std")]
    #[inline]
    fn cbrt(self) -> Self {
        <f64>::cbrt(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn cbrt(self) -> Self {
        libm::cbrt(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn cbrt(self) -> Self {
        panic!("Floating point function 'cbrt' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn hypot(self, other: Self) -> Self {
        <f64>::hypot(self, other)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn hypot(self, other: Self) -> Self {
        libm::hypot(self, other)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn hypot(self, _other: Self) -> Self {
        panic!("Floating point function 'hypot' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sin(self) -> Self {
        <f64>::sin(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sin(self) -> Self {
        libm::sin(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sin(self) -> Self {
        panic!("Floating point function 'sin' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn cos(self) -> Self {
        <f64>::cos(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn cos(self) -> Self {
        libm::cos(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn cos(self) -> Self {
        panic!("Floating point function 'cos' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn tan(self) -> Self {
        <f64>::tan(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn tan(self) -> Self {
        libm::tan(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn tan(self) -> Self {
        panic!("Floating point function 'tan' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn asin(self) -> Self {
        <f64>::asin(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn asin(self) -> Self {
        libm::asin(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn asin(self) -> Self {
        panic!("Floating point function 'asin' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn acos(self) -> Self {
        <f64>::acos(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn acos(self) -> Self {
        libm::acos(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn acos(self) -> Self {
        panic!("Floating point function 'acos' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn atan(self) -> Self {
        <f64>::atan(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn atan(self) -> Self {
        libm::atan(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn atan(self) -> Self {
        panic!("Floating point function 'atan' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn atan2(self, other: Self) -> Self {
        <f64>::atan2(self, other)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2(self, other)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn atan2(self, _other: Self) -> Self {
        panic!("Floating point function 'atan2' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        <f64>::sin_cos(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (libm::sin(self), libm::cos(self))
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        panic!(
            "Floating point function 'sin_cos' not available in no-std without libm_math feature"
        )
    }

    #[cfg(feature = "std")]
    #[inline]
    fn exp_m1(self) -> Self {
        <f64>::exp_m1(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn exp_m1(self) -> Self {
        libm::expm1(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn exp_m1(self) -> Self {
        panic!("Floating point function 'exp_m1' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn ln_1p(self) -> Self {
        <f64>::ln_1p(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn ln_1p(self) -> Self {
        libm::log1p(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn ln_1p(self) -> Self {
        panic!("Floating point function 'ln_1p' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sinh(self) -> Self {
        <f64>::sinh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sinh(self) -> Self {
        libm::sinh(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sinh(self) -> Self {
        panic!("Floating point function 'sinh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn cosh(self) -> Self {
        <f64>::cosh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn cosh(self) -> Self {
        libm::cosh(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn cosh(self) -> Self {
        panic!("Floating point function 'cosh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn tanh(self) -> Self {
        <f64>::tanh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn tanh(self) -> Self {
        libm::tanh(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn tanh(self) -> Self {
        panic!("Floating point function 'tanh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn asinh(self) -> Self {
        <f64>::asinh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn asinh(self) -> Self {
        libm::asinh(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn asinh(self) -> Self {
        panic!("Floating point function 'asinh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn acosh(self) -> Self {
        <f64>::acosh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn acosh(self) -> Self {
        libm::acosh(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn acosh(self) -> Self {
        panic!("Floating point function 'acosh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn atanh(self) -> Self {
        <f64>::atanh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn atanh(self) -> Self {
        libm::atanh(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn atanh(self) -> Self {
        panic!("Floating point function 'atanh' not available in no-std without libm_math feature")
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
