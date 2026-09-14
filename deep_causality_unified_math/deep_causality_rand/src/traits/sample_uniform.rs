/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Rng, SampleBorrow, UniformDistributionError};

/// Which tower a scalar's range sampler comes from.
///
/// A type parameter and nothing else — neither marker is ever constructed. They exist so the two
/// blanket implementations of [`SampleUniform`] do not overlap: one trait cannot carry two blanket
/// impls over two towers, because coherence cannot prove a real field will never also be a natural
/// number. Distinguishing them by this parameter makes the two impls different items, and both can
/// then be written once for every type in their tower.
#[derive(Debug, Clone, Copy)]
pub struct FloatKind;

/// The unsigned-integer tower. See [`FloatKind`].
#[derive(Debug, Clone, Copy)]
pub struct UnsignedKind;

/// A scalar that a range can be sampled over, and the sampler that does it.
///
/// Implemented once per tower over the algebra, so no concrete type is named anywhere in this
/// crate and a scalar added to `deep_causality_num` works here on the day it arrives.
pub trait SampleUniform<Kind>: Sized {
    type Sampler: UniformSampler<X = Self>;
}

pub trait UniformSampler: Sized {
    type X;

    fn new<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized;

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized;

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X;

    fn sample_single<R: Rng + ?Sized, B1, B2>(
        low: B1,
        high: B2,
        rng: &mut R,
    ) -> Result<Self::X, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let uniform: Self = UniformSampler::new(low, high)?;
        Ok(uniform.sample(rng))
    }

    fn sample_single_inclusive<R: Rng + ?Sized, B1, B2>(
        low: B1,
        high: B2,
        rng: &mut R,
    ) -> Result<Self::X, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let uniform: Self = UniformSampler::new_inclusive(low, high)?;
        Ok(uniform.sample(rng))
    }
}
