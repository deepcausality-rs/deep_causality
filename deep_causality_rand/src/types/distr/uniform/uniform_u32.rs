/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Rng, SampleBorrow, SampleRange, SampleUniform, UniformSampler};
use crate::{RngError, UniformDistributionError};
use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UniformU32 {
    low: u32,
    high: u32,
}

impl UniformSampler for UniformU32 {
    type X = u32;

    fn new<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low_val = *low.borrow();
        let high_val = *high.borrow();
        if low_val >= high_val {
            return Err(UniformDistributionError::InvalidRange);
        }
        Ok(UniformU32 {
            low: low_val,
            high: high_val,
        })
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low_val = *low.borrow();
        let high_val = *high.borrow();
        if low_val >= high_val {
            return Err(UniformDistributionError::InvalidRange);
        }
        Ok(UniformU32 {
            low: low_val,
            high: high_val + 1,
        }) // Inclusive range
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        self.low + (rng.next_u32() % (self.high - self.low))
    }
}

impl SampleUniform for u32 {
    type Sampler = UniformU32;
}
impl SampleRange<u32> for Range<u32> {
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<u32, RngError> {
        // Changed RngCore to Rng
        let uniform = UniformU32::new(self.start, self.end)?;
        Ok(uniform.sample(rng))
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
