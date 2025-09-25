/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{MaybeUncertain, Uncertain, UncertainError};

impl MaybeUncertain<f64> {
    /// Probabilistically converts the `MaybeUncertain<f64>` to a standard `Uncertain<f64>`.
    ///
    /// This acts as a gate, returning `Ok(Uncertain<f64>)` only if the statistical evidence
    /// for the value's presence meets the specified threshold.
    pub fn lift_to_uncertain(
        &self,
        threshold_prob_some: f64,
        confidence_level: f64,
        epsilon: f64,       // 0.05
        max_samples: usize, // 1000
    ) -> Result<Uncertain<f64>, UncertainError> {
        let is_present = self.is_present.to_bool(
            threshold_prob_some,
            confidence_level,
            epsilon, // Default epsilon
            max_samples,
        )?;

        if is_present {
            Ok(self.value.clone())
        } else {
            Err(UncertainError::PresenceError(
                "Insufficient evidence for presence".to_string(),
            ))
        }
    }
}
