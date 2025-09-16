/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::float_utils::*;
use crate::{Distribution, Rng, StandardUniform};
use core::mem;

#[derive(Clone, Copy, Debug, Default)]
pub struct OpenClosed01;

#[derive(Clone, Copy, Debug, Default)]
pub struct Open01;

// This trait is needed by both this lib and rand_distr hence is a hidden export
#[doc(hidden)]
pub trait IntoFloat {
    type F;

    /// Helper method to combine the fraction and a constant exponent into a
    /// float.
    ///
    /// Only the least significant bits of `self` may be set, 23 for `f32` and
    /// 52 for `f64`.
    /// The resulting value will fall in a range that depends on the exponent.
    /// As an example the range with exponent 0 will be
    /// [2<sup>0</sup>..2<sup>1</sup>), which is [1..2).
    fn into_float_with_exponent(self, exponent: i32) -> Self::F;
}

macro_rules! float_impls {
    ($($meta:meta)?, $ty:ident, $uty:ident, $f_scalar:ident, $u_scalar:ty,
     $fraction_bits:expr, $exponent_bias:expr) => {
        $(#[cfg($meta)])?
        impl IntoFloat for $uty {
            type F = $ty;
            #[inline(always)]
            fn into_float_with_exponent(self, exponent: i32) -> $ty {
                // The exponent is encoded using an offset-binary representation
                let exponent_bits: $u_scalar =
                    (($exponent_bias + exponent) as $u_scalar) << $fraction_bits;
                $ty::from_bits(self | $uty::splat(exponent_bits))
            }
        }

        $(#[cfg($meta)])?
        impl Distribution<$ty> for StandardUniform {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $ty {
                // Multiply-based method; 24/53 random bits; [0, 1) interval.
                // We use the most significant bits because for simple RNGs
                // those are usually more random.
                let float_size = mem::size_of::<$f_scalar>() as $u_scalar * 8;
                let precision = $fraction_bits + 1;
                let scale = 1.0 / ((1 as $u_scalar << precision) as $f_scalar);

                let value: $uty = rng.random();
                let value = value >> $uty::splat(float_size - precision);
                $ty::splat(scale) * $ty::cast_from_int(value)
            }
        }

        $(#[cfg($meta)])?
        impl Distribution<$ty> for OpenClosed01 {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $ty {
                // Multiply-based method; 24/53 random bits; (0, 1] interval.
                // We use the most significant bits because for simple RNGs
                // those are usually more random.
                let float_size = mem::size_of::<$f_scalar>() as $u_scalar * 8;
                let precision = $fraction_bits + 1;
                let scale = 1.0 / ((1 as $u_scalar << precision) as $f_scalar);

                let value: $uty = rng.random();
                let value = value >> $uty::splat(float_size - precision);
                // Add 1 to shift up; will not overflow because of right-shift:
                $ty::splat(scale) * $ty::cast_from_int(value + $uty::splat(1))
            }
        }

        $(#[cfg($meta)])?
        impl Distribution<$ty> for Open01 {
            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> $ty {
                // Transmute-based method; 23/52 random bits; (0, 1) interval.
                // We use the most significant bits because for simple RNGs
                // those are usually more random.
                let float_size = mem::size_of::<$f_scalar>() as $u_scalar * 8;

                let value: $uty = rng.random();
                let fraction = value >> $uty::splat(float_size - $fraction_bits);
                fraction.into_float_with_exponent(0) - $ty::splat(1.0 - $f_scalar::EPSILON / 2.0)
            }
        }
    }
}

float_impls! { , f32, u32, f32, u32, 23, 127 }
float_impls! { , f64, u64, f64, u64, 52, 1023 }
