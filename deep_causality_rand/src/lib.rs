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
pub use crate::types::distr::normal::Normal;
pub use crate::types::distr::normal::standard_normal::StandardNormal;
pub use crate::types::distr::uniform::standard_uniform::StandardUniform;
pub use crate::types::distr::uniform::{Uniform, UniformFloat};
pub use crate::types::misc::iter::Iter;
pub use crate::types::misc::map::Map;

#[cfg(not(feature = "os-random"))]
use crate::types::Xoshiro256;
#[cfg(not(feature = "os-random"))]
use std::cell::RefCell;

#[cfg(feature = "os-random")]
use crate::types::rand::os_random_rng::OsRandomRng;

#[cfg(not(feature = "os-random"))]
thread_local! {
    static THREAD_RNG: RefCell<Xoshiro256> = RefCell::new(Xoshiro256::new());
}

#[cfg(not(feature = "os-random"))]
pub struct ThreadRng;

#[cfg(not(feature = "os-random"))]
impl RngCore for ThreadRng {
    fn next_u32(&mut self) -> u32 {
        THREAD_RNG.with(|rng| rng.borrow_mut().next_u32())
    }
    fn next_u64(&mut self) -> u64 {
        THREAD_RNG.with(|rng| rng.borrow_mut().next_u64())
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        THREAD_RNG.with(|rng| rng.borrow_mut().fill_bytes(dest))
    }
}

#[cfg(not(feature = "os-random"))]
impl Rng for ThreadRng {}

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
        ThreadRng
    }
}
