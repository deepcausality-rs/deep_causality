/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The unit-interval draws, written once and generic in the scalar.
//!
//! These replace four per-type files — one each for `f32`, `f64` and `Float106`, plus a shared
//! bit-kernel module — with one body per distribution. `deep_causality_fft` is the pattern: a
//! blanket-implemented capability trait and no per-type source file anywhere in the crate.
//!
//! The draw is arithmetic in the caller's scalar rather than a bit pattern reinterpreted as a
//! float. That costs a multiply and buys a body that serves any real field, including the two
//! software scalars whose layout no bit trick reaches: `Float106` is two limbs and `BFloat16` has
//! no `from_bits` at all.

use crate::{Open01, OpenClosed01, StandardUniform};
use deep_causality_rand::{Distribution, RandScalar, Rng};

/// A value in `[0, 1)` carrying as much entropy as the scalar's significand absorbs.
///
/// The accumulation lives in `deep_causality_rand` as [`RandScalar::rand_float_gen`], because the
/// range sampler there needs the same value and two copies of one loop is two things to get wrong.
/// It reads how many words the scalar can hold from the scalar's own `epsilon`, so nothing here
/// declares a width.
#[inline]
fn unit<T, R>(rng: &mut R) -> T
where
    T: RandScalar,
    R: Rng + ?Sized,
{
    T::rand_float_gen(rng)
}

/// Uniform on `[0, 1)`.
impl<T> Distribution<T> for StandardUniform
where
    T: RandScalar,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        loop {
            let v = unit::<T, R>(rng);
            // A narrow significand can round a value drawn from `[0, 1)` onto exactly `1.0`,
            // leaving the interval every inverse-CDF transform assumes. Reject and redraw rather
            // than clamp, which would pile an atom of probability mass on one value, or pre-scale,
            // which would bias every draw to correct a rare one. `BFloat16` keeps 8 significand
            // bits, so the rejection probability there is about `2^-9`.
            if v < T::one() {
                return v;
            }
        }
    }
}

/// Uniform on `(0, 1)`: the open interval a logarithm can be taken over.
impl<T> Distribution<T> for Open01
where
    T: RandScalar,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        loop {
            // Rejection rather than clamping a zero to some small positive value, which would put
            // mass at that value. A zero draw has probability `2^-53` per word, so the loop is
            // expected to run once.
            let v: T = StandardUniform.sample(rng);
            if v > T::zero() {
                return v;
            }
        }
    }
}

/// Uniform on `(0, 1]`.
impl<T> Distribution<T> for OpenClosed01
where
    T: RandScalar,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        // `1 - u` over a half-open `[0, 1)` reflects the excluded endpoint onto the included one.
        let v: T = StandardUniform.sample(rng);
        T::one() - v
    }
}
