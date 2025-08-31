//! Defines the probability distributions used as sources of uncertainty.

use crate::errors::uncertain_error::UncertainError;
use crate::types::distribution_parameters::{
    BernoulliParams, NormalDistributionParams, UniformDistributionParams,
};
use rand::Rng;
use rand_distr::{Bernoulli, Distribution, Normal, Uniform};

/// Enum for all supported probability distributions, generic over the value it produces.
#[derive(Debug, Clone, Copy)]
pub enum DistributionEnum<T> {
    Point(T),
    Normal(NormalDistributionParams),
    Uniform(UniformDistributionParams),
    Bernoulli(BernoulliParams),
}

// Implementation for distributions that produce f64.
impl DistributionEnum<f64> {
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

// Implementation for distributions that produce bool.
impl DistributionEnum<bool> {
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
