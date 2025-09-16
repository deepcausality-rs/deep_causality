/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::float::*;
use crate::utils::float_utils::*;
use crate::{Rng, SampleBorrow, SampleUniform, UniformDistributionError, UniformSampler};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformFloat<X> {
    low: X,
    scale: X,
}

macro_rules! uniform_float_impl {
    ($($meta:meta)?, $ty:ty, $uty:ident, $f_scalar:ident, $u_scalar:ident, $bits_to_discard:expr) => {
        $(#[cfg($meta)])?
        impl UniformFloat<$ty> {
            /// Construct, reducing `scale` as required to ensure that rounding
            /// can never yield values greater than `high`.
            ///
            /// Note: though it may be tempting to use a variant of this method
            /// to ensure that samples from `[low, high)` are always strictly
            /// less than `high`, this approach may be very slow where
            /// `scale.abs()` is much smaller than `high.abs()`
            /// (example: `low=0.99999999997819644, high=1.`).
            fn new_bounded(low: $ty, high: $ty, mut scale: $ty) -> Self {
                let max_rand = <$ty>::splat(1.0 as $f_scalar - $f_scalar::EPSILON);

                loop {
                    let mask = (scale * max_rand + low).gt_mask(high);
                    if !mask.any() {
                        break;
                    }
                    scale = scale.decrease_masked(mask);
                }

                debug_assert!(<$ty>::splat(0.0).all_le(scale));

                UniformFloat { low, scale }
            }
        }

        $(#[cfg($meta)])?
        impl SampleUniform for $ty {
            type Sampler = UniformFloat<$ty>;
        }

        $(#[cfg($meta)])?
        impl UniformSampler for UniformFloat<$ty> {
            type X = $ty;

            fn new<B1, B2>(low_b: B1, high_b: B2) -> Result<Self, UniformDistributionError>
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                #[cfg(debug_assertions)]
                if !(low.all_finite()) || !(high.all_finite()) {
                    return Err(UniformDistributionError::NonFinite);
                }
                if !(low.all_lt(high)) {
                    return Err(UniformDistributionError::EmptyRange);
                }

                let scale = high - low;
                if !(scale.all_finite()) {
                    return Err(UniformDistributionError::NonFinite);
                }

                Ok(Self::new_bounded(low, high, scale))
            }

            fn new_inclusive<B1, B2>(low_b: B1, high_b: B2) -> Result<Self, UniformDistributionError>
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                #[cfg(debug_assertions)]
                if !(low.all_finite()) || !(high.all_finite()) {
                    return Err(UniformDistributionError::NonFinite);
                }
                if !low.all_le(high) {
                    return Err(UniformDistributionError::EmptyRange);
                }

                let max_rand = <$ty>::splat(1.0 as $f_scalar - $f_scalar::EPSILON);
                let scale = (high - low) / max_rand;
                if !scale.all_finite() {
                    return Err(UniformDistributionError::NonFinite);
                }

                Ok(Self::new_bounded(low, high, scale))
            }

            fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
                // Generate a value in the range [1, 2)
                let value1_2 = (rng.random::<$uty>() >> $uty::splat($bits_to_discard)).into_float_with_exponent(0);

                // Get a value in the range [0, 1) to avoid overflow when multiplying by scale
                let value0_1 = value1_2 - <$ty>::splat(1.0);

                // We don't use `f64::mul_add`, because it is not available with
                // `no_std`. Furthermore, it is slower for some targets (but
                // faster for others). However, the order of multiplication and
                // addition is important, because on some platforms (e.g. ARM)
                // it will be optimized to a single (non-FMA) instruction.
                value0_1 * self.scale + self.low
            }

            #[inline]
            fn sample_single<R: Rng + ?Sized, B1, B2>(low_b: B1, high_b: B2, rng: &mut R) -> Result<Self::X, UniformDistributionError>
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                Self::sample_single_inclusive(low_b, high_b, rng)
            }

            #[inline]
            fn sample_single_inclusive<R: Rng + ?Sized, B1, B2>(low_b: B1, high_b: B2, rng: &mut R) -> Result<Self::X, UniformDistributionError>
            where
                B1: SampleBorrow<Self::X> + Sized,
                B2: SampleBorrow<Self::X> + Sized,
            {
                let low = *low_b.borrow();
                let high = *high_b.borrow();
                #[cfg(debug_assertions)]
                if !low.all_finite() || !high.all_finite() {
                    return Err(UniformDistributionError::NonFinite);
                }
                if !low.all_le(high) {
                    return Err(UniformDistributionError::EmptyRange);
                }
                let scale = high - low;
                if !scale.all_finite() {
                    return Err(UniformDistributionError::NonFinite);
                }

                // Generate a value in the range [1, 2)
                let value1_2 =
                    (rng.random::<$uty>() >> $uty::splat($bits_to_discard)).into_float_with_exponent(0);

                // Get a value in the range [0, 1) to avoid overflow when multiplying by scale
                let value0_1 = value1_2 - <$ty>::splat(1.0);

                // Doing multiply before addition allows some architectures
                // to use a single instruction.
                Ok(value0_1 * scale + low)
            }
        }
    };
}

uniform_float_impl! { , f32, u32, f32, u32, 32 - 23 }
uniform_float_impl! { , f64, u64, f64, u64, 64 - 52 }
