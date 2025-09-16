/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float;
use std::num::FpCategory;

macro_rules! float_impl_std {
    ($T:ident $decode:ident) => {
        impl Float for $T {
            constant! {
                nan() -> $T::NAN;
                infinity() -> $T::INFINITY;
                neg_infinity() -> $T::NEG_INFINITY;
                neg_zero() -> -0.0;
                min_value() -> $T::MIN;
                min_positive_value() -> $T::MIN_POSITIVE;
                epsilon() -> $T::EPSILON;
                max_value() -> $T::MAX;
            }

            #[inline]
            #[allow(deprecated)]
            fn abs_sub(self, other: Self) -> Self {
                <$T>::abs_sub(self, other)
            }

            #[inline]
            fn integer_decode(self) -> (u64, i16, i8) {
                $decode(self)
            }

            forward! {
                Self::is_nan(self) -> bool;
                Self::is_infinite(self) -> bool;
                Self::is_finite(self) -> bool;
                Self::is_normal(self) -> bool;
                Self::is_subnormal(self) -> bool;
                Self::classify(self) -> FpCategory;
                Self::clamp(self, min: Self, max: Self) -> Self;
                Self::floor(self) -> Self;
                Self::ceil(self) -> Self;
                Self::round(self) -> Self;
                Self::trunc(self) -> Self;
                Self::fract(self) -> Self;
                Self::abs(self) -> Self;
                Self::signum(self) -> Self;
                Self::is_sign_positive(self) -> bool;
                Self::is_sign_negative(self) -> bool;
                Self::mul_add(self, a: Self, b: Self) -> Self;
                Self::recip(self) -> Self;
                Self::powi(self, n: i32) -> Self;
                Self::powf(self, n: Self) -> Self;
                Self::sqrt(self) -> Self;
                Self::exp(self) -> Self;
                Self::exp2(self) -> Self;
                Self::ln(self) -> Self;
                Self::log(self, base: Self) -> Self;
                Self::log2(self) -> Self;
                Self::log10(self) -> Self;
                Self::to_degrees(self) -> Self;
                Self::to_radians(self) -> Self;
                Self::max(self, other: Self) -> Self;
                Self::min(self, other: Self) -> Self;
                Self::cbrt(self) -> Self;
                Self::hypot(self, other: Self) -> Self;
                Self::sin(self) -> Self;
                Self::cos(self) -> Self;
                Self::tan(self) -> Self;
                Self::asin(self) -> Self;
                Self::acos(self) -> Self;
                Self::atan(self) -> Self;
                Self::atan2(self, other: Self) -> Self;
                Self::sin_cos(self) -> (Self, Self);
                Self::exp_m1(self) -> Self;
                Self::ln_1p(self) -> Self;
                Self::sinh(self) -> Self;
                Self::cosh(self) -> Self;
                Self::tanh(self) -> Self;
                Self::asinh(self) -> Self;
                Self::acosh(self) -> Self;
                Self::atanh(self) -> Self;
                Self::copysign(self, sign: Self) -> Self;
            }
        }
    };
}

/// Forward a method to an inherent method or a base trait method.
macro_rules! forward {
    ($( Self :: $method:ident ( self $( , $arg:ident : $ty:ty )* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method(self $( , $arg : $ty )* ) -> $ret {
                Self::$method(self $( , $arg )* )
            }
        )*};
    ($( $base:ident :: $method:ident ( self $( , $arg:ident : $ty:ty )* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method(self $( , $arg : $ty )* ) -> $ret {
                <Self as $base>::$method(self $( , $arg )* )
            }
        )*};
    ($( $base:ident :: $method:ident ( $( $arg:ident : $ty:ty ),* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method( $( $arg : $ty ),* ) -> $ret {
                <Self as $base>::$method( $( $arg ),* )
            }
        )*};
    ($( $imp:path as $method:ident ( self $( , $arg:ident : $ty:ty )* ) -> $ret:ty ; )*)
        => {$(
            #[inline]
            fn $method(self $( , $arg : $ty )* ) -> $ret {
                $imp(self $( , $arg )* )
            }
        )*};
}

macro_rules! constant {
    ($( $method:ident () -> $ret:expr ; )*)
        => {$(
            #[inline]
            fn $method() -> Self {
                $ret
            }
        )*};
}

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

float_impl_std!(f32 integer_decode_f32);
float_impl_std!(f64 integer_decode_f64);
