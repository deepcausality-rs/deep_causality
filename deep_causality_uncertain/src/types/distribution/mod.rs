//! Defines the probability distributions used as sources of uncertainty.

use crate::errors::uncertain_error::UncertainError;
use crate::{NormalDistributionParams, UniformDistributionParams};
use rand::Rng;
use rand_distr::{Distribution as RandDistribution, Normal, Uniform};

/// Enum for all supported probability distributions.
/// This enum dispatch approach avoids the performance overhead of `dyn Trait`.
#[derive(Debug, Clone, Copy)]
pub enum Distribution {
    /// A single, certain value.
    Point(f64),
    /// A Normal (Gaussian) distribution.
    Normal(NormalDistributionParams),
    /// A Uniform distribution.
    Uniform(UniformDistributionParams),
}

impl Distribution {
    /// Samples a single value from the distribution.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Result<f64, UncertainError> {
        match self {
            Distribution::Point(v) => Ok(*v),
            Distribution::Normal(params) => {
                let normal = Normal::new(params.mean, params.std_dev)?;
                Ok(normal.sample(rng))
            }
            Distribution::Uniform(params) => {
                let uniform = Uniform::new(params.low, params.high)?;
                Ok(uniform.sample(rng))
            }
        }
    }
}
