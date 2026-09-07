/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The crate's error type.

use alloc::string::String;
use core::fmt;

/// What can go wrong computing a statistic.
///
/// The variants separate three kinds of cause: an input the mathematics does not admit
/// (`EmptyInput`, `NegativeProbability`, `NonPositiveScale`), a shape the caller got wrong
/// (`DimensionMismatch`, `InvalidBinCount`), and a computation that could not reach an answer
/// (`RankDeficient`, `NotConverged`, `ConversionFailed`). A caller can act on the second and
/// third; the first is a statement about the data.
#[derive(Debug, Clone, PartialEq)]
pub enum StatsErrorEnum {
    /// A statistic was asked for over no observations.
    EmptyInput(String),
    /// Fewer observations than the statistic's degrees of freedom require.
    ///
    /// The corrected variance needs two: with one observation the divisor `n − 1` is zero and
    /// there is no dispersion to estimate.
    InsufficientSamples(String),
    /// A probability was negative, which no zero policy can interpret.
    NegativeProbability(String),
    /// Two inputs that must agree on length or shape do not.
    DimensionMismatch(String),
    /// A scale parameter was zero or negative where the density requires it positive.
    NonPositiveScale(String),
    /// A design matrix has no unique solution at the penalty given.
    RankDeficient(String),
    /// An iterative fit reached its cap without meeting its stopping test.
    ///
    /// Carries the iteration count so the caller can distinguish a cap that is too low from a
    /// problem that does not converge.
    NotConverged { iterations: usize, detail: String },
    /// A bin count below two, or above what the data can support.
    InvalidBinCount(String),
    /// An input carried a non-finite value where the statistic has no meaning for one.
    NonFiniteInput(String),
    /// A count could not be represented in the working scalar.
    ConversionFailed(String),
}

/// The crate's error, wrapping [`StatsErrorEnum`].
#[derive(Debug, Clone, PartialEq)]
pub struct StatsError(pub StatsErrorEnum);

impl StatsError {
    /// A statistic was asked for over no observations.
    #[allow(non_snake_case)]
    pub fn EmptyInput<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::EmptyInput(msg.into()))
    }

    /// Fewer observations than the statistic's degrees of freedom require.
    #[allow(non_snake_case)]
    pub fn InsufficientSamples<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::InsufficientSamples(msg.into()))
    }

    /// A probability was negative, which no zero policy can interpret.
    #[allow(non_snake_case)]
    pub fn NegativeProbability<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::NegativeProbability(msg.into()))
    }

    /// Two inputs that must agree on length or shape do not.
    #[allow(non_snake_case)]
    pub fn DimensionMismatch<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::DimensionMismatch(msg.into()))
    }

    /// A scale parameter was zero or negative where the density requires it positive.
    #[allow(non_snake_case)]
    pub fn NonPositiveScale<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::NonPositiveScale(msg.into()))
    }

    /// A design matrix has no unique solution at the penalty given.
    #[allow(non_snake_case)]
    pub fn RankDeficient<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::RankDeficient(msg.into()))
    }

    /// An iterative fit reached its cap without meeting its stopping test.
    #[allow(non_snake_case)]
    pub fn NotConverged<S: Into<String>>(iterations: usize, detail: S) -> Self {
        Self(StatsErrorEnum::NotConverged {
            iterations,
            detail: detail.into(),
        })
    }

    /// A bin count below two, or above what the data can support.
    #[allow(non_snake_case)]
    pub fn InvalidBinCount<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::InvalidBinCount(msg.into()))
    }

    /// An input carried a non-finite value where the statistic has no meaning for one.
    #[allow(non_snake_case)]
    pub fn NonFiniteInput<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::NonFiniteInput(msg.into()))
    }

    /// A count could not be represented in the working scalar.
    #[allow(non_snake_case)]
    pub fn ConversionFailed<S: Into<String>>(msg: S) -> Self {
        Self(StatsErrorEnum::ConversionFailed(msg.into()))
    }

    /// The variant, for matching without reaching through the tuple field.
    pub fn kind(&self) -> &StatsErrorEnum {
        &self.0
    }
}

impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            StatsErrorEnum::EmptyInput(m) => write!(f, "Empty input: {m}"),
            StatsErrorEnum::InsufficientSamples(m) => write!(f, "Insufficient samples: {m}"),
            StatsErrorEnum::NegativeProbability(m) => write!(f, "Negative probability: {m}"),
            StatsErrorEnum::DimensionMismatch(m) => write!(f, "Dimension mismatch: {m}"),
            StatsErrorEnum::NonPositiveScale(m) => write!(f, "Non-positive scale: {m}"),
            StatsErrorEnum::RankDeficient(m) => write!(f, "Rank deficient: {m}"),
            StatsErrorEnum::NotConverged { iterations, detail } => {
                write!(
                    f,
                    "Did not converge after {iterations} iterations: {detail}"
                )
            }
            StatsErrorEnum::InvalidBinCount(m) => write!(f, "Invalid bin count: {m}"),
            StatsErrorEnum::NonFiniteInput(m) => write!(f, "Non-finite input: {m}"),
            StatsErrorEnum::ConversionFailed(m) => write!(f, "Conversion failed: {m}"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for StatsError {}
