/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Rng, RngError, SampleUniform, UniformSampler};
use alloc::string::ToString;
use core::ops::Range;

/// A range a value can be drawn from.
///
/// `Kind` names the tower the scalar belongs to and is inferred at the call site; it is what lets
/// one implementation below serve both. See [`FloatKind`](crate::FloatKind).
pub trait SampleRange<T, Kind> {
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<T, RngError>;

    fn is_empty(&self) -> bool;
}

/// The half-open range, for every scalar of either tower.
///
/// The draw is delegated to the scalar's own sampler, so this file names no type and knows nothing
/// about floats or integers beyond which sampler the tower supplies.
impl<T, K> SampleRange<T, K> for Range<T>
where
    T: SampleUniform<K> + PartialOrd + Copy,
{
    fn sample_single<R: Rng + ?Sized>(self, rng: &mut R) -> Result<T, RngError> {
        if self.is_empty() {
            return Err(RngError::InvalidRange(
                "Invalid range: low must be less than high".to_string(),
            ));
        }
        let sampler = <T::Sampler as UniformSampler>::new(self.start, self.end)
            .map_err(|e| RngError::InvalidRange(e.to_string()))?;
        Ok(sampler.sample(rng))
    }

    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}
