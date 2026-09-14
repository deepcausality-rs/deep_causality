/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::BernoulliDistributionError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_rand::{Distribution, Rng};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bernoulli {
    /// Probability of success, relative to the maximal integer.
    p_int: u64,
}

// To sample from the Bernoulli distribution we use a method that compares a
// random `u64` value `v < (p * 2^64)`.
//
// If `p == 1.0`, the integer `v` to compare against can not represented as a
// `u64`. We manually set it to `u64::MAX` instead (2^64 - 1 instead of 2^64).
// Note that  value of `p < 1.0` can never result in `u64::MAX`, because an
// `f64` only has 53 bits of precision, and the next largest value of `p` will
// result in `2^64 - 2048`.
//
// Also there is a 100% theoretical concern: if someone consistently wants to
// generate `true` using the Bernoulli distribution (i.e. by using a probability
// of `1.0`), just using `u64::MAX` is not enough. On average it would return
// false once every 2^64 iterations. Some people apparently care about this
// case.
//
// That is why we special-case `u64::MAX` to always return `true`, without using
// the RNG, and pay the performance price for all uses that *are* reasonable.
// Luckily, if `new()` and `sample` are close, the compiler can optimize out the
// extra check.
const ALWAYS_TRUE: u64 = u64::MAX;

// This is just `2.0.powi(64)`, but written this way because it is not available
// in `no_std` mode.
const SCALE: f64 = 2.0 * (1u64 << 63) as f64;

impl Bernoulli {
    /// Construct a new `Bernoulli` with the given probability of success `p`, stated in the
    /// caller's scalar.
    ///
    /// For `p = 1.0` the distribution always generates true, and for `p = 0.0` always false; both
    /// are exact rather than overwhelmingly likely.
    ///
    /// # The probability is quantised to `2^-64`, whatever scalar states it
    ///
    /// This is the exception to precision as a parameter in this crate, and it is deliberate. The
    /// draw is an integer comparison against `p · 2^64` — see the note above `ALWAYS_TRUE` — which
    /// makes it exact at both endpoints and free of the rounding a unit-draw comparison would
    /// carry. What it costs is resolution: `p` is held to a multiple of `2^-64` and no finer, so a
    /// `Float106` caller stating 106 significand bits of probability keeps 64 of them, and
    /// [`Bernoulli::p`] returns the quantised value rather than the one they gave.
    ///
    /// The bound is the representation's, not the scalar's, so widening the scalar does not move
    /// it. A caller who needs a probability finer than `2^-64` needs a different construction, not
    /// a wider float.
    #[inline]
    pub fn new<T: RealField>(p: T) -> Result<Bernoulli, BernoulliDistributionError> {
        let p = p
            .to_f64()
            .ok_or(BernoulliDistributionError::InvalidProbability)?;
        if !(0.0..1.0).contains(&p) {
            if p == 1.0 {
                return Ok(Bernoulli { p_int: ALWAYS_TRUE });
            }
            return Err(BernoulliDistributionError::InvalidProbability);
        }
        Ok(Bernoulli {
            p_int: (p * SCALE) as u64,
        })
    }

    /// Construct a new `Bernoulli` with the probability of success of
    /// `numerator`-in-`denominator`. I.e. `new_ratio(2, 3)` will return
    /// a `Bernoulli` with a 2-in-3 chance, or about 67%, of returning `true`.
    ///
    /// return `true`. If `numerator == 0` it will always return `false`.
    /// For `numerator > denominator` and `denominator == 0`, this returns an
    /// error. Otherwise, for `numerator == denominator`, samples are always
    /// true; for `numerator == 0` samples are always false.
    #[inline]
    pub fn from_ratio(
        numerator: u32,
        denominator: u32,
    ) -> Result<Bernoulli, BernoulliDistributionError> {
        if numerator > denominator || denominator == 0 {
            return Err(BernoulliDistributionError::InvalidProbability);
        }
        if numerator == denominator {
            return Ok(Bernoulli { p_int: ALWAYS_TRUE });
        }
        let p_int = ((f64::from(numerator) / f64::from(denominator)) * SCALE) as u64;
        Ok(Bernoulli { p_int })
    }

    #[inline]
    /// Returns the probability (`p`) of the distribution, at the caller's scalar.
    ///
    /// This is the quantised probability, a multiple of `2^-64`, which may differ from the value
    /// passed to [`Bernoulli::new`] — see the note there. The scalar chosen affects how the value
    /// is carried away from here, never how finely it was held.
    pub fn p<T: RealField + FromPrimitive>(&self) -> T {
        let p = if self.p_int == ALWAYS_TRUE {
            1.0
        } else {
            (self.p_int as f64) / SCALE
        };
        T::from_f64(p).expect("a probability in [0, 1] converts to every supported scalar")
    }
}

impl Distribution<bool> for Bernoulli {
    #[inline]
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        // Make sure to always return true for p = 1.0.
        if self.p_int == ALWAYS_TRUE {
            return true;
        }
        let v: u64 = deep_causality_rand::StandardWord.sample(rng);
        v < self.p_int
    }
}
