/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    BernoulliParams, NormalDistributionParams, Sample, UncertainError, UniformDistributionParams,
};
// The shaped distributions are mathematics and come from `stats`, and so do the two traits a
// caller needs to name in order to draw from one — `stats` re-exports them so a crate reaching for
// distributions needs no second dependency to spell a bound.
//
// `Uniform` is the exception and comes from `rand` directly, because it is range sampling rather
// than a shaped distribution: it is built on `SampleUniform`, which can only be implemented in the
// crate that owns it, so it cannot move. Naming the entropy crate for it is truthful.
use crate::UncertainScalar;
use deep_causality_rand::Uniform;
use deep_causality_stats::{Bernoulli, Distribution, Normal, Rng};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistributionEnum<T> {
    Point(T),
    Normal(NormalDistributionParams<T>),
    Uniform(UniformDistributionParams<T>),
    Bernoulli(BernoulliParams),
}

/// One sampling body for every scalar.
///
/// There were three — `f64`, `Float106` and `bool` — and the first two had identical bodies, which
/// is what said the split was carrying no information. The third was not a precision at all: a
/// Bernoulli is a distribution parameterised by a probability, and what it produces is a truth
/// value at any scalar. So the Boolean case is a *branch* here rather than an instantiation, and
/// the return type says which by carrying a [`Sample`].
///
/// A scalar joins by satisfying [`UncertainScalar`] and by nothing else. There is no per-type entry in
/// this file to keep in step with a type added elsewhere.
impl<T: UncertainScalar> DistributionEnum<T> {
    /// Draws one value, at `T` for the shaped distributions and as a Boolean for the Bernoulli.
    ///
    /// # Errors
    ///
    /// The construction refusals of the underlying distributions — a non-finite or non-positive
    /// standard deviation, an empty or non-finite range, a probability outside `[0, 1]` — carried
    /// through with the message that names which. The "this distribution does not produce that
    /// type" refusals are gone: with one arm per distribution rather than three per type, there is
    /// no mismatched pair left to report.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<Sample<T>, UncertainError> {
        match self {
            DistributionEnum::Point(v) => Ok(Sample::Real(*v)),
            DistributionEnum::Normal(params) => {
                let normal = Normal::new(params.mean, params.std_dev)
                    .map_err(|e| UncertainError::NormalDistributionError(e.to_string()))?;
                Ok(Sample::Real(normal.sample(rng)))
            }
            DistributionEnum::Uniform(params) => {
                let uniform = Uniform::new(params.low, params.high)
                    .map_err(|e| UncertainError::UniformDistributionError(e.to_string()))?;
                Ok(Sample::Real(uniform.sample(rng)))
            }
            DistributionEnum::Bernoulli(params) => {
                let bernoulli = Bernoulli::new(params.p)
                    .map_err(|e| UncertainError::BernoulliDistributionError(e.to_string()))?;
                Ok(Sample::Bool(bernoulli.sample(rng)))
            }
        }
    }

    /// Whether drawing from this distribution consumes entropy.
    ///
    /// A point distribution returns its parameter and draws nothing, so it takes neither a leaf
    /// ordinal nor a Sobol dimension. Both pre-passes ask this one question rather than each
    /// matching on the variants, so they cannot drift apart.
    pub(crate) fn draws(&self) -> bool {
        !matches!(self, DistributionEnum::Point(_))
    }
}

impl<T> Display for DistributionEnum<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionEnum::Point(d) => write!(f, "Distribution: Point {{ D: {} }}", d),
            DistributionEnum::Normal(d) => write!(f, "Distribution: Normal {{ D: {} }}", d),
            DistributionEnum::Uniform(d) => write!(f, "Distribution: Uniform {{ D: {} }}", d),
            DistributionEnum::Bernoulli(d) => write!(f, "Distribution: Bernoulli {{ D: {} }}", d),
        }
    }
}
