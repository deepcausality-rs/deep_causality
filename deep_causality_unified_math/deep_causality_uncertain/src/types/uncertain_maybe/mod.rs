/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A value that is probabilistically present, and uncertain when it is.

mod uncertain_maybe_ops;

use crate::UncertainScalar;
use crate::{SampleSession, Uncertain, UncertainBool, UncertainError};

/// A real quantity that is probabilistically present or absent. When present, its value is
/// uncertain.
///
/// # Two channels, one scalar, one index
///
/// `is_present` is an [`UncertainBool<R>`] and `value` an [`Uncertain<R>`] — the Boolean and real
/// carriers over the same scalar. Presence is a Boolean event and not a real, which is why the
/// channels differ in carrier; they agree in scalar because they are drawn at **one** session
/// index, and an index addresses one tree.
///
/// They were drawn at two separately chosen global indices before, which made keeping them in step
/// the sample cache's hardest job. At one index there are no two channels to reconcile.
///
/// # `MaybeUncertain<R>` is a maybe-real
///
/// There is deliberately no Boolean form. A probabilistically-present truth value is expressible —
/// Kleene's three values, `None` / `Some(false)` / `Some(true)` — but it had no use outside this
/// crate's own tests, and a third carrier struct with the same body would cut against the point of
/// making the scalar a parameter. Presence itself is already Boolean, and that is the Boolean
/// content this type has.
#[derive(Debug, Clone)]
pub struct MaybeUncertain<R> {
    is_present: UncertainBool<R>,
    value: Uncertain<R>,
}

impl<R: UncertainScalar> MaybeUncertain<R> {
    /// Certainly present, but uncertain in value.
    pub fn from_uncertain(value: Uncertain<R>) -> Self {
        Self {
            is_present: UncertainBool::point(true),
            value,
        }
    }

    /// Certainly present with a certain value.
    pub fn from_value(value: R) -> Self {
        Self {
            is_present: UncertainBool::point(true),
            value: Uncertain::point(value),
        }
    }

    /// Certainly absent.
    ///
    /// The value channel holds a point at zero. It is never read — the presence channel gates it —
    /// and zero is the one value every scalar has without naming one.
    pub fn always_none() -> Self {
        Self {
            is_present: UncertainBool::point(false),
            value: Uncertain::point(R::zero()),
        }
    }

    /// Presence drawn from a Bernoulli trial; value from the given distribution.
    pub fn from_bernoulli_and_uncertain(prob_some: R, present_value_dist: Uncertain<R>) -> Self {
        Self {
            is_present: UncertainBool::bernoulli(prob_some),
            value: present_value_dist,
        }
    }

    /// Samples the value, returning `Some(R)` if present and `None` if absent.
    ///
    /// Both channels are drawn at one index, so presence and value belong to the same draw.
    pub fn sample(&self, session: &mut SampleSession) -> Result<Option<R>, UncertainError> {
        let index = session.next_index();
        if self.is_present.sample_at(session, index)? {
            Ok(Some(self.value.sample_at(session, index)?))
        } else {
            Ok(None)
        }
    }

    /// As [`Self::sample`], with no session of the caller's own.
    pub fn sample_from_entropy(&self) -> Result<Option<R>, UncertainError> {
        self.sample(&mut SampleSession::from_entropy())
    }

    /// The probability of the value being present, as an uncertain truth value.
    pub fn is_some(&self) -> UncertainBool<R> {
        self.is_present.clone()
    }

    /// The probability of the value being absent, as an uncertain truth value.
    pub fn is_none(&self) -> UncertainBool<R> {
        !self.is_present.clone()
    }

    /// SPRT-gated collapse to a plain [`Uncertain<R>`].
    ///
    /// Acts as a presence gate: the value channel is returned only if the statistical evidence
    /// that the value is *present* meets the threshold; otherwise a
    /// [`UncertainError::PresenceError`] is returned. The presence test runs entirely on the
    /// `is_present` Bernoulli channel, so the gate is the same test at every scalar — and its
    /// parameters, being dimensionless probabilities, are stated in that scalar.
    ///
    /// # Arguments
    /// * `threshold_prob_some` — presence probability the evidence must clear.
    /// * `confidence_level` — SPRT confidence (e.g. `0.95`).
    /// * `epsilon` — indifference region around the threshold.
    /// * `max_samples` — SPRT sampling budget.
    pub fn lift_to_uncertain(
        &self,
        session: &SampleSession,
        threshold_prob_some: R,
        confidence_level: R,
        epsilon: R,
        max_samples: usize,
    ) -> Result<Uncertain<R>, UncertainError> {
        let is_present = self.is_present.to_bool(
            session,
            threshold_prob_some,
            confidence_level,
            epsilon,
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

    /// As [`Self::lift_to_uncertain`], with no session of the caller's own.
    pub fn lift_to_uncertain_from_entropy(
        &self,
        threshold_prob_some: R,
        confidence_level: R,
        epsilon: R,
        max_samples: usize,
    ) -> Result<Uncertain<R>, UncertainError> {
        self.lift_to_uncertain(
            &SampleSession::from_entropy(),
            threshold_prob_some,
            confidence_level,
            epsilon,
            max_samples,
        )
    }
}

/// Structural equality on both channels.
///
/// Written out rather than derived: the derive would bound `R: PartialEq`, but what is compared is
/// the two carriers, whose own equality is on their graphs and needs the carrier bound instead.
impl<R: UncertainScalar> PartialEq for MaybeUncertain<R> {
    fn eq(&self, other: &Self) -> bool {
        self.is_present == other.is_present && self.value == other.value
    }
}
