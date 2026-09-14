/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{MaybeUncertain, Uncertain, UncertainError};

impl MaybeUncertain<bool> {
    pub fn from_uncertain(value: Uncertain<bool>) -> Self {
        Self {
            is_present: Uncertain::<bool>::point(true),
            value,
        }
    }

    /// Creates a new `MaybeUncertain` from a value that is certainly present and has a certain value.
    pub fn from_value(value: bool) -> Self {
        Self {
            is_present: Uncertain::<bool>::point(true),
            value: Uncertain::<bool>::point(value),
        }
    }

    /// Creates a new `MaybeUncertain` that is certainly absent.
    pub fn always_none() -> Self {
        Self {
            is_present: Uncertain::<bool>::point(false),
            value: Uncertain::<bool>::point(false), // This value will never be used
        }
    }

    /// Creates a `MaybeUncertain` where presence is determined by a Bernoulli trial.
    pub fn from_bernoulli_and_uncertain(
        prob_some: f64,
        present_value_dist: Uncertain<bool>,
    ) -> Self {
        Self {
            is_present: Uncertain::bernoulli(prob_some),
            value: present_value_dist,
        }
    }

    /// Samples the `MaybeUncertain` value, returning `Some(bool)` if present or `None` if absent.
    pub fn sample(
        &self,
        session: &mut crate::SampleSession,
    ) -> Result<Option<bool>, UncertainError> {
        // Both channels are drawn at one index. They were drawn at two separately chosen global
        // indices before, which made keeping them in step the cache's hardest job; at one index
        // there are no two channels to reconcile.
        let index = session.next_index();
        if self.is_present.sample_at(session, index)? {
            Ok(Some(self.value.sample_at(session, index)?))
        } else {
            Ok(None)
        }
    }

    /// As [`Self::sample`], with no session of the caller's own.
    pub fn sample_from_entropy(&self) -> Result<Option<bool>, UncertainError> {
        self.sample(&mut crate::SampleSession::from_entropy())
    }

    /// Returns an `Uncertain<bool>` representing the probability of the value being present.
    pub fn is_some(&self) -> Uncertain<bool> {
        self.is_present.clone()
    }

    /// Returns an `Uncertain<bool>` representing the probability of the value being absent.
    pub fn is_none(&self) -> Uncertain<bool> {
        !self.is_present.clone()
    }

    // `lift_to_uncertain` is the precision-generic gate on `MaybeUncertain<T>` (see `mod.rs`).
}
