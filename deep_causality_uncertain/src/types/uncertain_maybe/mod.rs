/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod uncertain_maybe_lift;
mod uncertain_maybe_op_arithmetic;

use crate::{Uncertain, UncertainError};

/// A first-class type representing a value that is probabilistically present or absent.
/// If the value is present, its own value is uncertain.
#[derive(Clone, Debug)]
pub struct MaybeUncertain<T> {
    is_present: Uncertain<bool>,
    value: Uncertain<T>,
}

// Implementation for f64 since the underlying Uncertain<T> is specialized for f64 and bool.
impl MaybeUncertain<f64> {
    /// Creates a new `MaybeUncertain` from a value that is certainly present but uncertain in value.
    pub fn from_uncertain(value: Uncertain<f64>) -> Self {
        Self {
            is_present: Uncertain::<bool>::point(true),
            value,
        }
    }

    /// Creates a new `MaybeUncertain` from a value that is certainly present and has a certain value.
    pub fn from_value(value: f64) -> Self {
        Self {
            is_present: Uncertain::<bool>::point(true),
            value: Uncertain::<f64>::point(value),
        }
    }

    /// Creates a new `MaybeUncertain` that is certainly absent.
    pub fn always_none() -> Self {
        Self {
            is_present: Uncertain::<bool>::point(false),
            value: Uncertain::<f64>::point(0.0), // This value will never be used
        }
    }

    /// Creates a `MaybeUncertain` where presence is determined by a Bernoulli trial.
    pub fn from_bernoulli_and_uncertain(
        prob_some: f64,
        present_value_dist: Uncertain<f64>,
    ) -> Self {
        Self {
            is_present: Uncertain::bernoulli(prob_some),
            value: present_value_dist,
        }
    }

    /// Samples the `MaybeUncertain` value, returning `Some(f64)` if present or `None` if absent.
    pub fn sample(&self) -> Result<Option<f64>, UncertainError> {
        if self.is_present.sample()? {
            Ok(Some(self.value.sample()?))
        } else {
            Ok(None)
        }
    }

    /// Returns an `Uncertain<bool>` representing the probability of the value being present.
    pub fn is_some(&self) -> Uncertain<bool> {
        self.is_present.clone()
    }

    /// Returns an `Uncertain<bool>` representing the probability of the value being absent.
    pub fn is_none(&self) -> Uncertain<bool> {
        !self.is_present.clone()
    }
}
