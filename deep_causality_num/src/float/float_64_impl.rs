/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float;
use core::num::FpCategory;

#[cfg(all(not(feature = "std"), feature = "libm_math"))]
use libm;

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
        #[cfg(feature = "std")]
        return self.floor();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::floor(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for floor");
            f64::NAN
        }
    }

    #[inline]
    fn ceil(self) -> Self {
        #[cfg(feature = "std")]
        return self.ceil();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::ceil(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for ceil");
            f64::NAN
        }
    }

    #[inline]
    fn round(self) -> Self {
        #[cfg(feature = "std")]
        return self.round();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::round(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for round");
            f64::NAN
        }
    }

    #[inline]
    fn trunc(self) -> Self {
        #[cfg(feature = "std")]
        return self.trunc();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::trunc(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for trunc");
            f64::NAN
        }
    }

    #[inline]
    fn fract(self) -> Self {
        #[cfg(feature = "std")]
        return self.fract();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return self - libm::trunc(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for fract");
            f64::NAN
        }
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
        #[cfg(feature = "std")]
        return self.mul_add(a, b);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::fma(self, a, b);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for mul_add");
            f64::NAN
        }
    }

    #[inline]
    fn recip(self) -> Self {
        f64::recip(self)
    }

    #[inline]
    fn powi(self, n: i32) -> Self {
        #[cfg(feature = "std")]
        return self.powi(n);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::pow(self, n as f64);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for powi");
            f64::NAN
        }
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        #[cfg(feature = "std")]
        return self.powf(n);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::pow(self, n);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for powf");
            f64::NAN
        }
    }

    #[inline]
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")]
        return self.sqrt();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sqrt(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for sqrt");
            f64::NAN
        }
    }

    #[inline]
    fn exp(self) -> Self {
        #[cfg(feature = "std")]
        return self.exp();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::exp(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for exp");
            f64::NAN
        }
    }

    #[inline]
    fn exp2(self) -> Self {
        #[cfg(feature = "std")]
        return self.exp2();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::exp2(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for exp2");
            f64::NAN
        }
    }

    #[inline]
    fn ln(self) -> Self {
        #[cfg(feature = "std")]
        return self.ln();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for ln");
            f64::NAN
        }
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        #[cfg(feature = "std")]
        return self.log(base);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log(self) / libm::log(base);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for log");
            f64::NAN
        }
    }

    #[inline]
    fn log2(self) -> Self {
        #[cfg(feature = "std")]
        return self.log2();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log2(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for log2");
            f64::NAN
        }
    }

    #[inline]
    fn log10(self) -> Self {
        #[cfg(feature = "std")]
        return self.log10();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log10(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for log10");
            f64::NAN
        }
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
        #[cfg(feature = "std")]
        return self.cbrt();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::cbrt(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for cbrt");
            f64::NAN
        }
    }

    #[inline]
    fn hypot(self, other: Self) -> Self {
        #[cfg(feature = "std")]
        return self.hypot(other);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::hypot(self, other);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for hypot");
            f64::NAN
        }
    }

    #[inline]
    fn sin(self) -> Self {
        #[cfg(feature = "std")]
        return self.sin();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sin(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for sin");
            f64::NAN
        }
    }

    #[inline]
    fn cos(self) -> Self {
        #[cfg(feature = "std")]
        return self.cos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::cos(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for cos");
            f64::NAN
        }
    }

    #[inline]
    fn tan(self) -> Self {
        #[cfg(feature = "std")]
        return self.tan();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::tan(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for tan");
            f64::NAN
        }
    }

    #[inline]
    fn asin(self) -> Self {
        #[cfg(feature = "std")]
        return self.asin();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::asin(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for asin");
            f64::NAN
        }
    }

    #[inline]
    fn acos(self) -> Self {
        #[cfg(feature = "std")]
        return self.acos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::acos(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for acos");
            f64::NAN
        }
    }

    #[inline]
    fn atan(self) -> Self {
        #[cfg(feature = "std")]
        return self.atan();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::atan(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for atan");
            f64::NAN
        }
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        #[cfg(feature = "std")]
        return self.atan2(other);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::atan2(self, other);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for atan2");
            f64::NAN
        }
    }

    #[inline]
    fn sin_cos(self) -> (Self, Self) {
        #[cfg(feature = "std")]
        return self.sin_cos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return (libm::sin(self), libm::cos(self));

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for sin_cos");
            (f64::NAN, f64::NAN)
        }
    }

    #[inline]
    fn exp_m1(self) -> Self {
        #[cfg(feature = "std")]
        return self.exp_m1();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::expm1(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for exp_m1");
            f64::NAN
        }
    }

    #[inline]
    fn ln_1p(self) -> Self {
        #[cfg(feature = "std")]
        return self.ln_1p();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log1p(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for ln_1p");
            f64::NAN
        }
    }

    #[inline]
    fn sinh(self) -> Self {
        #[cfg(feature = "std")]
        return self.sinh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sinh(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for sinh");
            f64::NAN
        }
    }

    #[inline]
    fn cosh(self) -> Self {
        #[cfg(feature = "std")]
        return self.cosh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::cosh(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for cosh");
            f64::NAN
        }
    }

    #[inline]
    fn tanh(self) -> Self {
        #[cfg(feature = "std")]
        return self.tanh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::tanh(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for tanh");
            f64::NAN
        }
    }

    #[inline]
    fn asinh(self) -> Self {
        #[cfg(feature = "std")]
        return self.asinh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::asinh(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for asinh");
            f64::NAN
        }
    }

    #[inline]
    fn acosh(self) -> Self {
        #[cfg(feature = "std")]
        return self.acosh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::acosh(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for acosh");
            f64::NAN
        }
    }

    #[inline]
    fn atanh(self) -> Self {
        #[cfg(feature = "std")]
        return self.atanh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::atanh(self);

        #[cfg(all(not(feature = "std"), not(feature = "libm_math")))]
        {
            compile_error!("'std' or 'libm_math' feature required for atanh");
            f64::NAN
        }
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
