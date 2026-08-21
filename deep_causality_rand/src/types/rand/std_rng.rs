/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::rng::Rng;
use crate::traits::rng_core::RngCore;

// Xoshiro256 prng
pub struct Xoshiro256 {
    s: [u64; 4],
}

impl Default for Xoshiro256 {
    fn default() -> Self {
        Self::new()
    }
}

/// Base seed shared by both seeding paths.
const BASE_SEED: u64 = 0x736f_6d65_7073_6575;

/// Ambient entropy from the host: a fresh `RandomState` mixed with the thread
/// id, so each thread of a process draws a distinct stream.
#[cfg(feature = "std")]
fn entropy_seed() -> u64 {
    use core::hash::BuildHasher;
    use std::collections::hash_map::RandomState;
    use std::thread;

    let hash_builder = RandomState::new();
    BASE_SEED.wrapping_add(hash_builder.hash_one(thread::current().id()))
}

/// Bare metal has neither ambient entropy nor a thread identity, so there is
/// nothing honest to draw from. A counter is mixed into the base seed instead:
/// successive `new()` calls within one run yield distinct streams, and the
/// sequence repeats identically after every reset.
///
/// When the stream has to differ per boot, seed explicitly with
/// [`Xoshiro256::from_seed`]. On an embedded target the entropy belongs to the
/// board — a hardware RNG peripheral, ADC noise, a timer capture — not to this
/// crate, which cannot know what the board offers.
///
/// Uses a 32-bit atomic, so targets without atomic compare-and-swap must use
/// [`Xoshiro256::from_seed`] directly.
#[cfg(not(feature = "std"))]
fn entropy_seed() -> u64 {
    use core::sync::atomic::{AtomicU32, Ordering};

    /// Golden-ratio odd stride: walks the whole 32-bit range before repeating.
    const GOLDEN: u32 = 0x9E37_79B9;
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let n = COUNTER.fetch_add(GOLDEN, Ordering::Relaxed);
    BASE_SEED.wrapping_add(u64::from(n))
}

impl Xoshiro256 {
    /// Creates a generator seeded from whatever ambient entropy the target
    /// offers. See [`entropy_seed`] for what that means on each: on `std` a
    /// per-thread draw, on `no_std` a per-call counter that repeats across
    /// resets. Use [`from_seed`](Self::from_seed) when the seed must be known
    /// or must vary per boot.
    pub fn new() -> Self {
        Self::from_seed(entropy_seed())
    }

    /// Creates a generator from an explicit 64-bit seed.
    ///
    /// The seed is expanded into the four-word state with SplitMix64 (the same
    /// expansion [`Xoshiro256::new`] applies to its thread-derived seed). Two
    /// generators built from the same seed produce identical streams, so this is
    /// the constructor to use when a run must be reproducible.
    pub fn from_seed(seed: u64) -> Self {
        let mut sm_state = seed;

        let mut s = [0; 4];
        for item in &mut s {
            // Seeding with SplitMix64
            sm_state = sm_state.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = sm_state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            *item = z ^ (z >> 31);
        }
        Xoshiro256 { s }
    }
}

impl RngCore for Xoshiro256 {
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;

        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];

        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        for chunk in dst.chunks_mut(8) {
            let val = self.next_u64();
            chunk.copy_from_slice(&val.to_ne_bytes()[..chunk.len()]);
        }
    }
}

impl Rng for Xoshiro256 {}
