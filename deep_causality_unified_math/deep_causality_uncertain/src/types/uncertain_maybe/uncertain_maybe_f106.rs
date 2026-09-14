/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `MaybeUncertain<Float106>`: a probabilistically-present double-double value. Mirrors the
//! `f64` surface for the precision-carrying paths (construction, sampling, the SPRT-gated
//! `lift_to_uncertain`). Presence is a Bernoulli fact, so the `is_present` channel stays
//! `Uncertain<bool>`; only the value channel carries `Float106`.

use crate::{MaybeUncertain, Uncertain, UncertainError};
use deep_causality_num::Float106;

impl MaybeUncertain<Float106> {
    /// Certainly present, but uncertain in value.
    pub fn from_uncertain(value: Uncertain<Float106>) -> Self {
        Self {
            is_present: Uncertain::<bool>::point(true),
            value,
        }
    }

    /// Certainly present with a certain `Float106` value (carried losslessly).
    pub fn from_value(value: Float106) -> Self {
        Self {
            is_present: Uncertain::<bool>::point(true),
            value: Uncertain::<Float106>::point(value),
        }
    }

    /// Certainly absent.
    pub fn always_none() -> Self {
        Self {
            is_present: Uncertain::<bool>::point(false),
            value: Uncertain::<Float106>::point(Float106::from(0.0)),
        }
    }

    /// Presence drawn from a Bernoulli trial; value from the given distribution.
    pub fn from_bernoulli_and_uncertain(
        prob_some: f64,
        present_value_dist: Uncertain<Float106>,
    ) -> Self {
        Self {
            is_present: Uncertain::bernoulli(prob_some),
            value: present_value_dist,
        }
    }

    /// Sample: `Some(Float106)` if present, else `None`.
    pub fn sample(
        &self,
        session: &mut crate::SampleSession,
    ) -> Result<Option<Float106>, UncertainError> {
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
    pub fn sample_from_entropy(&self) -> Result<Option<Float106>, UncertainError> {
        self.sample(&mut crate::SampleSession::from_entropy())
    }

    /// The probability of being present.
    pub fn is_some(&self) -> Uncertain<bool> {
        self.is_present.clone()
    }

    /// The probability of being absent.
    pub fn is_none(&self) -> Uncertain<bool> {
        !self.is_present.clone()
    }

    // `lift_to_uncertain` is the precision-generic gate on `MaybeUncertain<T>` (see `mod.rs`).
}
