/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Float, One, Zero};
use core::num::FpCategory;

#[cfg(all(not(feature = "std"), feature = "libm_math"))]
use libm;

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
        <f32>::fract(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn fract(self) -> Self {
        self - libm::truncf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn fract(self) -> Self {
        panic!("Floating point function 'fract' not available in no-std without libm_math feature")
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

    #[cfg(feature = "std")]
    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        <f32>::mul_add(self, a, b)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn mul_add(self, a: Self, b: Self) -> Self {
        libm::fmaf(self, a, b)
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
        f32::recip(self)
    }

    #[cfg(feature = "std")]
    #[inline]
    fn powi(self, n: i32) -> Self {
        <f32>::powi(self, n)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn powi(self, n: i32) -> Self {
        libm::powf(self, n as f32)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn powi(self, _n: i32) -> Self {
        panic!("Floating point function 'powi' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn powf(self, n: Self) -> Self {
        <f32>::powf(self, n)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn powf(self, n: Self) -> Self {
        libm::powf(self, n)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn powf(self, _n: Self) -> Self {
        panic!("Floating point function 'powf' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sqrt(self) -> Self {
        <f32>::sqrt(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sqrt(self) -> Self {
        libm::sqrtf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sqrt(self) -> Self {
        panic!("Floating point function 'sqrt' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn exp(self) -> Self {
        <f32>::exp(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn exp(self) -> Self {
        libm::expf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn exp(self) -> Self {
        panic!("Floating point function 'exp' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn exp2(self) -> Self {
        <f32>::exp2(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn exp2(self) -> Self {
        libm::exp2f(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn exp2(self) -> Self {
        panic!("Floating point function 'exp2' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn ln(self) -> Self {
        <f32>::ln(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn ln(self) -> Self {
        libm::logf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn ln(self) -> Self {
        panic!("Floating point function 'ln' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn log(self, base: Self) -> Self {
        <f32>::log(self, base)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn log(self, base: Self) -> Self {
        libm::logf(self) / libm::logf(base)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn log(self, _base: Self) -> Self {
        panic!("Floating point function 'log' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn log2(self) -> Self {
        <f32>::log2(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn log2(self) -> Self {
        libm::log2f(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn log2(self) -> Self {
        panic!("Floating point function 'log2' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn log10(self) -> Self {
        <f32>::log10(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn log10(self) -> Self {
        libm::log10f(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn log10(self) -> Self {
        panic!("Floating point function 'log10' not available in no-std without libm_math feature")
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

    #[cfg(feature = "std")]
    #[inline]
    fn cbrt(self) -> Self {
        <f32>::cbrt(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn cbrt(self) -> Self {
        libm::cbrtf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn cbrt(self) -> Self {
        panic!("Floating point function 'cbrt' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn hypot(self, other: Self) -> Self {
        <f32>::hypot(self, other)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn hypot(self, other: Self) -> Self {
        libm::hypotf(self, other)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn hypot(self, _other: Self) -> Self {
        panic!("Floating point function 'hypot' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sin(self) -> Self {
        <f32>::sin(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sin(self) -> Self {
        libm::sinf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sin(self) -> Self {
        panic!("Floating point function 'sin' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn cos(self) -> Self {
        <f32>::cos(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn cos(self) -> Self {
        libm::cosf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn cos(self) -> Self {
        panic!("Floating point function 'cos' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn tan(self) -> Self {
        <f32>::tan(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn tan(self) -> Self {
        libm::tanf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn tan(self) -> Self {
        panic!("Floating point function 'tan' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn asin(self) -> Self {
        <f32>::asin(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn asin(self) -> Self {
        libm::asinf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn asin(self) -> Self {
        panic!("Floating point function 'asin' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn acos(self) -> Self {
        <f32>::acos(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn acos(self) -> Self {
        libm::acosf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn acos(self) -> Self {
        panic!("Floating point function 'acos' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn atan(self) -> Self {
        <f32>::atan(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn atan(self) -> Self {
        libm::atanf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn atan(self) -> Self {
        panic!("Floating point function 'atan' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn atan2(self, other: Self) -> Self {
        <f32>::atan2(self, other)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn atan2(self, other: Self) -> Self {
        libm::atan2f(self, other)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn atan2(self, _other: Self) -> Self {
        panic!("Floating point function 'atan2' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        <f32>::sin_cos(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        (libm::sinf(self), libm::cosf(self))
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
        <f32>::exp_m1(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn exp_m1(self) -> Self {
        libm::expm1f(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn exp_m1(self) -> Self {
        panic!("Floating point function 'exp_m1' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn ln_1p(self) -> Self {
        <f32>::ln_1p(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn ln_1p(self) -> Self {
        libm::log1pf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn ln_1p(self) -> Self {
        panic!("Floating point function 'ln_1p' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn sinh(self) -> Self {
        <f32>::sinh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn sinh(self) -> Self {
        libm::sinhf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn sinh(self) -> Self {
        panic!("Floating point function 'sinh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn cosh(self) -> Self {
        <f32>::cosh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn cosh(self) -> Self {
        libm::coshf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn cosh(self) -> Self {
        panic!("Floating point function 'cosh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn tanh(self) -> Self {
        <f32>::tanh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn tanh(self) -> Self {
        libm::tanhf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn tanh(self) -> Self {
        panic!("Floating point function 'tanh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn asinh(self) -> Self {
        <f32>::asinh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn asinh(self) -> Self {
        libm::asinhf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn asinh(self) -> Self {
        panic!("Floating point function 'asinh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn acosh(self) -> Self {
        <f32>::acosh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn acosh(self) -> Self {
        libm::acoshf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn acosh(self) -> Self {
        panic!("Floating point function 'acosh' not available in no-std without libm_math feature")
    }

    #[cfg(feature = "std")]
    #[inline]
    fn atanh(self) -> Self {
        <f32>::atanh(self)
    }

    #[cfg(all(not(feature = "std"), feature = "libm_math"))]
    #[inline]
    fn atanh(self) -> Self {
        libm::atanhf(self)
    }

    #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
    #[inline]
    fn atanh(self) -> Self {
        panic!("Floating point function 'atanh' not available in no-std without libm_math feature")
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
