/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod standard_uniform;
mod uniform_float;
pub mod uniform_u32;
pub mod uniform_u64;

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    Distribution, Rng, SampleBorrow, SampleUniform, UniformDistributionError, UniformSampler,
};

pub struct Uniform<X: SampleUniform>(X::Sampler);

impl<X: SampleUniform> Uniform<X> {
    /// Create a new `Uniform` instance, which samples uniformly from the half
    /// open range `[low, high)` (excluding `high`).
    ///
    /// For discrete types (e.g. integers), samples will always be strictly less
    /// than `high`. For (approximations of) continuous types (e.g. `f32`, `f64`),
    /// samples may equal `high` due to loss of precision but may not be
    /// greater than `high`.
    ///
    /// Fails if `low >= high`, or if `low`, `high` or the range `high - low` is
    /// non-finite. In release mode, only the range is checked.
    pub fn new<B1, B2>(low: B1, high: B2) -> Result<Uniform<X>, UniformDistributionError>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        X::Sampler::new(low, high).map(Uniform)
    }

    /// Create a new `Uniform` instance, which samples uniformly from the closed
    /// range `[low, high]` (inclusive).
    ///
    /// Fails if `low > high`, or if `low`, `high` or the range `high - low` is
    /// non-finite. In release mode, only the range is checked.
    pub fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Uniform<X>, UniformDistributionError>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        X::Sampler::new_inclusive(low, high).map(Uniform)
    }
}

impl<X: SampleUniform> Distribution<X> for Uniform<X> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> X {
        self.0.sample(rng)
    }
}
