/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod errors;
mod traits;
pub mod types;
mod utils;

// Errors
pub use crate::errors::bernoulli_error::BernoulliDistributionError;
pub use crate::errors::normal_error::NormalDistributionError;
pub use crate::errors::rng_error::RngError;
pub use crate::errors::uniform_error::UniformDistributionError;
// Traits
pub use crate::traits::distribution::Distribution;
pub use crate::traits::fill::Fill;
pub use crate::traits::rng::Rng;
pub use crate::traits::rng_core::RngCore;
pub use crate::traits::sample_borrow::SampleBorrow;
pub use crate::traits::sample_range::SampleRange;
pub use crate::traits::sample_uniform::{SampleUniform, UniformSampler};
// Types
pub use crate::types::distr::bernoulli::Bernoulli;
pub use crate::types::distr::normal::normal::Normal;
pub use crate::types::distr::normal::standard_normal::StandardNormal;
pub use crate::types::distr::uniform::standard_uniform::StandardUniform;
pub use crate::types::distr::uniform::uniform::Uniform;
pub use crate::types::distr::uniform::uniform_u32::UniformU32;
pub use crate::types::distr::uniform::uniform_u64::UniformU64;
pub use crate::types::misc::iter::Iter;
pub use crate::types::misc::map::Map;

#[cfg(not(feature = "os-random"))]
use crate::types::SipHash13Rng;

#[cfg(feature = "os-random")]
use crate::types::rand::os_random_rng::OsRandomRng;

/// Returns a new random number generator.
///
/// By default, this returns a `SipHash13Rng`. If the `os-random` feature is enabled,
/// it returns an `OsRandomRng`.
pub fn rng() -> impl Rng {
    #[cfg(feature = "os-random")]
    {
        OsRandomRng::new().expect("Failed to create OsRandomRng")
    }
    #[cfg(not(feature = "os-random"))]
    {
        SipHash13Rng::new()
    }
}
