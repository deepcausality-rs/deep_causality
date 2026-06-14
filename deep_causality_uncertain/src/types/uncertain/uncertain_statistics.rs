/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ProbabilisticType, Uncertain, UncertainError};
use deep_causality_num::{FromPrimitive, RealField};

// Precision-generic Monte-Carlo statistics. Unlike the sampling surface (which needs only
// `ProbabilisticType`), these reduce many samples into one scalar, so they require the value
// type to be a real field with arithmetic and a square root — which also keeps them off
// `Uncertain<bool>`, where a mean is meaningless. `FromPrimitive` supplies the sample-count
// divisor at the value type's precision (no narrowing through `f64`).
impl<T: ProbabilisticType + RealField + FromPrimitive> Uncertain<T> {
    /// Estimates the expected value (mean) by averaging `num_samples` draws.
    pub fn expected_value(&self, num_samples: usize) -> Result<T, UncertainError> {
        if num_samples == 0 {
            return Ok(T::zero());
        }
        let mut sum = T::zero();
        for i in 0..num_samples {
            sum += self.sample_with_index(i as u64)?;
        }
        let n = T::from_usize(num_samples).ok_or_else(|| {
            UncertainError::SamplingError("sample count does not fit the value type".to_string())
        })?;
        Ok(sum / n)
    }

    /// Estimates the (sample) standard deviation from `num_samples` draws, using the
    /// `(n − 1)` Bessel-corrected denominator.
    pub fn standard_deviation(&self, num_samples: usize) -> Result<T, UncertainError> {
        if num_samples <= 1 {
            return Ok(T::zero());
        }

        let samples: Vec<T> = (0..num_samples)
            .map(|i| self.sample_with_index(i as u64))
            .collect::<Result<Vec<T>, UncertainError>>()?;

        let count_err = || {
            UncertainError::SamplingError("sample count does not fit the value type".to_string())
        };
        let n = T::from_usize(num_samples).ok_or_else(count_err)?;
        let n_minus_one = T::from_usize(num_samples - 1).ok_or_else(count_err)?;

        let mut sum = T::zero();
        for &x in &samples {
            sum += x;
        }
        let mean = sum / n;

        let mut variance = T::zero();
        for &x in &samples {
            let d = x - mean;
            variance += d * d;
        }
        variance /= n_minus_one;

        Ok(variance.sqrt())
    }
}
