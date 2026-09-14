/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Entropy: generators, and the draws that are properties of bits rather than of a distribution.
//!
//! # This crate makes no distributional claim
//!
//! What it supplies is a source of numbers — `Xoshiro256`, an OS-entropy thread generator, a Sobol
//! sequence — together with the draws that are facts about the bits themselves: a raw machine word
//! ([`StandardWord`]), a Boolean ([`StandardBool`]), and a value uniform over a range
//! ([`Uniform`]). None of those is a density. Nothing here states a mean, a variance or a moment,
//! and nothing here should.
//!
//! The shaped distributions — normal, exponential, Cauchy, Weibull, log-normal, Poisson,
//! categorical, the unit-interval draws and their inverse-CDF transforms — are mathematics and
//! live in `deep_causality_stats`, beside the densities and moments that define them. A crate that
//! wants to draw from one depends on `stats`, which re-exports the generator traits by name so
//! that no second dependency is needed to spell a bound.
//!
//! # The split is a coherence fact, not a preference
//!
//! A blanket `impl<T: RealField> Distribution<T> for StandardUniform` is `error[E0119]` against the
//! `u64`, `u32` and `bool` implementations that a generator must also provide: coherence cannot
//! prove `u64` will never be a real field. Separating *what a word is* from *what a number is
//! distributed as* removes the overlap rather than working around it, and the crate boundary is
//! where that separation is cheapest — `stats` has no reason to sample a machine word.
//!
//! `Uniform<X>` stays here for the mirror reason: it is built on [`SampleUniform`], which can only
//! be implemented in the crate that owns it. Range sampling is entropy wearing an interval.
//!
//! # Precision
//!
//! The float draws are generic in the scalar under [`RandScalar`], with one body and no per-type
//! implementation. The exceptions are documented at the item that has them: the Sobol coordinate
//! is fixed at `2^-32` by its direction-number table, and that is a property of the sequence rather
//! than of the scalar holding it.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod errors;
mod traits;
pub mod types;

// Errors
pub use crate::errors::rng_error::RngError;
pub use crate::errors::uniform_error::UniformDistributionError;

// Traits
pub use crate::traits::distribution::Distribution;
pub use crate::traits::fill::Fill;
pub use crate::traits::rng::Rng;
pub use crate::traits::rng_core::RngCore;
pub use crate::traits::sample_borrow::SampleBorrow;
pub use crate::traits::sample_range::SampleRange;
pub use crate::traits::sample_uniform::{FloatKind, SampleUniform, UniformSampler, UnsignedKind};

// Types
pub use crate::types::Xoshiro256;
pub use crate::types::distr::uniform::standard_word::{StandardBool, StandardWord};
pub use crate::types::distr::uniform::*;
pub use crate::types::iter::Iter;
pub use crate::types::map::Map;
pub use crate::types::qmc::sobol::{MAX_SOBOL_DIM, SobolSequence};

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
