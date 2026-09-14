/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The categorical distribution: weighted choice over a finite set.

use crate::{RandWidth, StandardUniform, StatsError};
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_rand::{Distribution, Rng};

/// Weighted choice over `0..n`, returning an index.
///
/// # Weights need not be normalised
///
/// The constructor stores what it is given and the sampler divides by the total, so `[1, 3, 6]` and
/// `[0.1, 0.3, 0.6]` are the same distribution. A caller should not have to normalise, and a
/// sampler that forgets to is wrong in a way that is invisible whenever the caller happened to.
#[derive(Clone, Debug, PartialEq)]
pub struct Categorical<T> {
    weights: Vec<T>,
    total: T,
}

impl<T> Categorical<T>
where
    T: RealField,
{
    /// Construct from weights.
    ///
    /// Refuses an empty vector, a negative weight, a non-finite weight, or weights summing to
    /// zero — none of which names a distribution over the indices.
    pub fn new(weights: Vec<T>) -> Result<Self, StatsError> {
        if weights.is_empty() {
            return Err(StatsError::EmptyInput(
                "a categorical distribution needs at least one category",
            ));
        }
        let mut total = T::zero();
        for w in &weights {
            if !w.is_finite() {
                return Err(StatsError::NonFiniteInput(
                    "a categorical weight must be finite",
                ));
            }
            if *w < T::zero() {
                return Err(StatsError::NegativeProbability(
                    "a categorical weight cannot be negative",
                ));
            }
            total += *w;
        }
        if total <= T::zero() {
            return Err(StatsError::NonPositiveScale(
                "the categorical weights must not all be zero",
            ));
        }
        Ok(Self { weights, total })
    }
}

/// Accessors. These need no algebra, so they are not behind the `RealField` bound — a caller
/// holding a constructed value can read its shape without proving anything about its scalar.
impl<T> Categorical<T> {
    /// The weights, as given.
    pub fn weights(&self) -> &[T] {
        &self.weights
    }

    /// How many categories there are.
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// Whether there are no categories. Never true for a constructed value.
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }
}

/// A cumulative scan over the weights, against a uniform draw scaled by their total.
impl<T> Distribution<usize> for Categorical<T>
where
    T: RealField + FromPrimitive + RandWidth,
    StandardUniform: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        // Scaled by the total, so the caller need not normalise.
        let u: T = StandardUniform.sample(rng);
        let mut remaining = u * self.total;
        for (i, w) in self.weights.iter().enumerate() {
            // Strictly less, so a zero weight is never selected: with `<=` a category of weight
            // zero would win whenever the running remainder reached exactly zero.
            if remaining < *w {
                return i;
            }
            remaining -= *w;
        }
        // Reachable only through accumulated rounding, where the remainder outlives the weights by
        // a few ulps. The last category is the right answer there — it is the one the scan was in
        // the middle of — and returning it is what keeps the draw inside `0..n`.
        self.weights.len() - 1
    }
}
