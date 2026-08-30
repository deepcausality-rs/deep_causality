/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod errors;
mod extensions;
mod traits;
pub mod types;
mod utils;

// Errors
pub use crate::errors::bernoulli_error::BernoulliDistributionError;
pub use crate::errors::normal_error::NormalDistributionError;
pub use crate::errors::rng_error::RngError;
pub use crate::errors::uniform_error::UniformDistributionError;
// Extensions

// Traits
pub use crate::traits::distribution::Distribution;
pub use crate::traits::fill::Fill;
pub use crate::traits::real_rng::RealRng;
pub use crate::traits::rng::Rng;
pub use crate::traits::rng_core::RngCore;
pub use crate::traits::sample_borrow::SampleBorrow;
pub use crate::traits::sample_range::SampleRange;
pub use crate::traits::sample_uniform::{SampleUniform, UniformSampler};
// Inverse-CDF transforms
pub use crate::utils::inverse_cdf::{
    bernoulli_inverse_cdf, standard_normal_inverse_cdf, standard_normal_inverse_cdf_f106,
    uniform_inverse_cdf,
};
// Types
pub use crate::types::Xoshiro256;
pub use crate::types::distr::bernoulli::Bernoulli;
pub use crate::types::distr::normal::Normal;
pub use crate::types::distr::normal::standard_normal::StandardNormal;
pub use crate::types::distr::uniform::standard_uniform::StandardUniform;
pub use crate::types::distr::uniform::{Uniform, UniformFloat};
pub use crate::types::iter::Iter;
pub use crate::types::map::Map;
pub use crate::types::qmc::sobol::{MAX_SOBOL_DIM, SobolSequence};
pub use crate::types::range::{Open01, OpenClosed01};

#[cfg(all(feature = "std", not(feature = "os-random")))]
use core::cell::RefCell;

#[cfg(all(feature = "std", not(feature = "os-random")))]
thread_local! {
    static THREAD_RNG: RefCell<Xoshiro256> = RefCell::new(Xoshiro256::new());
}

#[cfg(all(feature = "std", not(feature = "os-random")))]
pub struct ThreadRng;

#[cfg(all(feature = "std", not(feature = "os-random")))]
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

#[cfg(all(feature = "std", not(feature = "os-random")))]
impl Rng for ThreadRng {}

/// Returns a new random number generator. Which one depends on the enabled
/// features:
///
/// * With `os-random`: an `OsRandomRng` that sources every draw from the
///   operating system. This is the option to use in production.
/// * With `std` and without `os-random` (the default): a `ThreadRng` handle to
///   a thread-local [`Xoshiro256`]. Each thread seeds its generator once, from
///   a fresh `RandomState` mixed with the thread id, so threads of one process
///   run distinct streams.
/// * Without `std` and without `os-random` (bare metal): an owned
///   [`Xoshiro256`] that the caller keeps, since there are no threads to hang a
///   thread-local on. Its seed comes from a per-call counter rather than
///   ambient entropy, so the sequence of streams repeats identically after
///   every reset. When the stream has to differ per boot, build the generator
///   with [`Xoshiro256::from_seed`] and a seed drawn from the board.
pub fn rng() -> impl Rng {
    #[cfg(feature = "os-random")]
    {
        #[cfg(feature = "os-random")]
        use crate::types::OsRandomRng;

        OsRandomRng::new().expect("Failed to create OsRandomRng")
    }
    #[cfg(all(feature = "std", not(feature = "os-random")))]
    {
        ThreadRng
    }
    // Bare metal: no threads, so no thread-local generator. Hand back an
    // owned `Xoshiro256`; the caller keeps it rather than reaching for a
    // process-wide one.
    #[cfg(all(not(feature = "std"), not(feature = "os-random")))]
    {
        Xoshiro256::new()
    }
}
