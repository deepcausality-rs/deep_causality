/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Rng, RngError};
use alloc::string::ToString;
use core::ops::Range;

pub trait SampleRange<T> {
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<T, RngError>;

    fn is_empty(&self) -> bool;
}

impl SampleRange<f32> for Range<f32> {
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<f32, RngError> {
        if self.is_empty() {
            return Err(RngError::InvalidRange(
                "Invalid range: low must be less than high".to_string(),
            ));
        }
        // Through the range sampler rather than a numerical draw: sampling uniformly from a
        // range is arithmetic over the bounds, and owes nothing to the distribution layer.
        let random_val: f32 = <f32 as crate::types::distr::uniform::RandFloat>::rand_float_gen(rng);
        Ok(self.start + (self.end - self.start) * random_val)
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

impl SampleRange<f64> for Range<f64> {
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<f64, RngError> {
        if self.is_empty() {
            return Err(RngError::InvalidRange(
                "Invalid range: low must be less than high".to_string(),
            ));
        }
        // Through the range sampler rather than a numerical draw: sampling uniformly from a
        // range is arithmetic over the bounds, and owes nothing to the distribution layer.
        let random_val: f64 = <f64 as crate::types::distr::uniform::RandFloat>::rand_float_gen(rng);
        Ok(self.start + (self.end - self.start) * random_val)
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
