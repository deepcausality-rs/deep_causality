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

use crate::{Open01, OpenClosed01, RandWidth, StandardUniform};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_rand::{Distribution, Rng, StandardWord};

/// `2^-53`: the weight of each further word below the one before it.
const WORD_SCALE: f64 = 1.0 / ((1_u64 << 53) as f64);

/// One 53-bit word as an `f64` in `[0, 1)`.
///
/// Computed in `f64` and converted once, rather than accumulated in the target scalar. A narrow
/// significand would otherwise quantize the intermediate before the value is formed: `BFloat16`
/// keeps 8 bits, so `(w >> 11)` would round to 8 bits before the division rather than after.
///
/// Measured, and worth recording because it is not what one would guess: converting once did
/// **not** move the observed distribution for `BFloat16` — the most frequent value took 0.00429 of
/// 200 000 draws either way. The two paths agree because the division is exact at both widths for
/// the values involved. The `f64` form is kept because it rounds once by construction rather than
/// by coincidence, and because it is the same shape for every scalar.
///
/// `f64` is wide enough for every scalar of 53 bits or fewer; the wider ones take several words.
#[inline]
fn word_unit_f64<R: Rng + ?Sized>(rng: &mut R) -> f64 {
    let w: u64 = StandardWord.sample(rng);
    (w >> 11) as f64 * WORD_SCALE
}

/// A value in `[0, 1)` carrying as much entropy as the scalar's significand absorbs.
///
/// Each further word is placed `2^-53` below the last, so a double-double receives two independent
/// 53-bit draws rather than one widened. See [`RandWidth`] for why the count cannot be assumed.
#[inline]
fn unit<T, R>(rng: &mut R) -> T
where
    T: RealField + FromPrimitive + RandWidth,
    R: Rng + ?Sized,
{
    let word_scale = T::from_f64(WORD_SCALE).expect("2^-53 converts to every scalar");
    let mut acc = T::from_f64(word_unit_f64(rng)).expect("a unit value converts to every scalar");
    let mut scale = T::one();
    for _ in 1..T::WORDS {
        // Deepened before use rather than after. The other order leaves the final update dead —
        // the loop runs `WORDS - 1` times and the last write is never read — which makes every
        // mutation of it equivalent and so untestable. Mutation testing found exactly that.
        scale *= word_scale;
        acc += T::from_f64(word_unit_f64(rng)).expect("a unit value converts to every scalar")
            * scale;
    }
    acc
}

/// Uniform on `[0, 1)`.
impl<T> Distribution<T> for StandardUniform
where
    T: RealField + FromPrimitive + RandWidth,
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
    T: RealField + FromPrimitive + RandWidth,
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
    T: RealField + FromPrimitive + RandWidth,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        // `1 - u` over a half-open `[0, 1)` reflects the excluded endpoint onto the included one.
        let v: T = StandardUniform.sample(rng);
        T::one() - v
    }
}
