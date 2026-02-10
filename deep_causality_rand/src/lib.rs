/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

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
pub use crate::types::range::{Open01, OpenClosed01};

#[cfg(not(feature = "os-random"))]
use crate::types::Xoshiro256;
#[cfg(not(feature = "os-random"))]
use std::cell::RefCell;

#[cfg(feature = "aead-random")]
use crate::types::ChaCha20Rng;


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
/// By default, this returns a `ThreadRng` backed by `Xoshiro256` PRNG, with each
/// thread getting a unique seed derived from the thread ID.
/// If the `os-random` feature is enabled, it returns an `OsRandomRng` that
/// sources entropy from the operating system.
/// If the `aead-random` feature is enabled, it returns a `ChaCha20Rng` seeded from
/// the OS CSPRNG. This is the preferred secure option.
pub fn rng() -> impl Rng {
    #[cfg(feature = "aead-random")]
    {
        // Use getrandom::u64 to seed ChaCha20
        use getrandom::u64 as getrandom_u64;
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        use std::thread;
        use std::time::{Instant, SystemTime};

        // 1. Get Hardware Entropy (OS RNG)
        let mut hardware_seed = [0u8; 32];
        for chunk in hardware_seed.chunks_mut(8) {
            let val = getrandom_u64().expect("Failed to get secure seed for ChaCha20Rng");
            chunk.copy_from_slice(&val.to_ne_bytes()[..chunk.len()]);
        }

        // 2. Gather Software Entropy
        // We use a Hasher to mix various environmental sources into a 64-bit value,
        // then expand/mix it into the 32-byte seed.
        let mut hasher = RandomState::new().build_hasher();

        // A. Time Sources
        hasher.write_u64(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );
        hasher.write_u64(Instant::now().elapsed().as_nanos() as u64);

        // B. Thread/Process Identity
        use std::hash::Hash;
        thread::current().id().hash(&mut hasher);

        // C. Memory Layout (ASLR) - Address of a stack variable
        let stack_var = 0;
        hasher.write_usize(&stack_var as *const i32 as usize);

        // D. CPU Cycle Counter (High-resolution hardware timer)
        // Extremely hard for external observers to predict the exact cycle count.
        #[cfg(target_arch = "x86_64")]
        {
            // SAFETY: RDTSC is available on all x86_64 CPUs.
            unsafe {
                hasher.write_u64(core::arch::x86_64::_rdtsc());
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            // Use CNTVCT_EL0 (Virtual Count Register) on ARM64
            // SAFETY: Available on standard aarch64 platforms.
            unsafe {
                let mut cntvct: u64;
                std::arch::asm!("mrs {}, cntvct_el0", out(reg) cntvct);
                hasher.write_u64(cntvct);
            }
        }

        // E. Heap Layout (ASLR) - Address of a heap allocation
        // Heap allocators are non-deterministic and vary based on system state.
        let heap_var = Box::new(0u8);
        hasher.write_usize(heap_var.as_ref() as *const u8 as usize);

        // Final Mixed Entropy
        let software_entropy = hasher.finish();

        // 3. Mix (XOR) Software Entropy into Hardware Seed
        // We spread the 64-bit software entropy across the 256-bit seed.
        // This ensures that even if the OS RNG is backdoored (predictable),
        // the final seed depends on the exact nanosecond execution time and memory layout,
        // which the adversary cannot know efficiently.
        for (i, chunk) in hardware_seed.chunks_mut(8).enumerate() {
            let mut val = u64::from_ne_bytes(chunk.try_into().unwrap());
            // Rotate the entropy for each chunk to avoid repeating the same pattern
            val ^= software_entropy.rotate_left(i as u32 * 13);
            chunk.copy_from_slice(&val.to_ne_bytes());
        }

        ChaCha20Rng::new(hardware_seed)
    }
    #[cfg(all(feature = "os-random", not(feature = "aead-random")))]
    {
        OsRandomRng::new().expect("Failed to create OsRandomRng")
    }
    #[cfg(all(not(feature = "os-random"), not(feature = "aead-random")))]
    {
        ThreadRng
    }
}
