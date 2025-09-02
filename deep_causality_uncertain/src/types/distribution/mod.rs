/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{BernoulliParams, NormalDistributionParams, UncertainError, UniformDistributionParams};
use rand::Rng;
use rand_distr::{Bernoulli, Distribution, Normal, Uniform};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
pub enum DistributionEnum<T> {
    Point(T),
    Normal(NormalDistributionParams),
    Uniform(UniformDistributionParams),
    Bernoulli(BernoulliParams),
}

impl DistributionEnum<f64> {
    /// Samples a value from the distribution for `f64` types.
    ///
    /// # Arguments
    ///
    /// * `rng` - A mutable reference to a random number generator.
    ///
    /// # Returns
    ///
    /// A `Result` containing the sampled `f64` value or an `UncertainError` if the distribution type is unsupported.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<f64, UncertainError> {
        match self {
            DistributionEnum::Point(v) => Ok(*v),
            DistributionEnum::Normal(params) => {
                let normal = Normal::new(params.mean, params.std_dev)?;
                Ok(normal.sample(rng))
            }
            DistributionEnum::Uniform(params) => {
                let uniform = Uniform::new(params.low, params.high)?;
                Ok(uniform.sample(rng))
            }
            _ => Err(UncertainError::UnsupportedTypeError(
                "Distribution does not produce f64".to_string(),
            )),
        }
    }
}

impl DistributionEnum<bool> {
    /// Samples a value from the distribution for `bool` types.
    ///
    /// # Arguments
    ///
    /// * `rng` - A mutable reference to a random number generator.
    ///
    /// # Returns
    ///
    /// A `Result` containing the sampled `bool` value or an `UncertainError` if the distribution type is unsupported.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<bool, UncertainError> {
        match self {
            DistributionEnum::Point(v) => Ok(*v),
            DistributionEnum::Bernoulli(params) => {
                let bernoulli = Bernoulli::new(params.p)?;
                Ok(bernoulli.sample(rng))
            }
            _ => Err(UncertainError::UnsupportedTypeError(
                "Distribution does not produce bool".to_string(),
            )),
        }
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
