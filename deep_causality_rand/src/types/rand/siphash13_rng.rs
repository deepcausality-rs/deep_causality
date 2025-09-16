/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::rng::Rng;
use crate::traits::rng_core::RngCore;

pub struct SipHash13Rng {
    v0: u64,
    v1: u64,
    v2: u64,
    v3: u64,
    length: u64,
}

impl Default for SipHash13Rng {
    fn default() -> Self {
        Self::new()
    }
}

impl SipHash13Rng {
    pub fn new() -> Self {
        let k0 = 0x736f6d6570736575u64;
        let k1 = 0x646f72616e646f6du64;

        SipHash13Rng {
            v0: k0 ^ 0x736f6d6570736575u64,
            v1: k1 ^ 0x646f72616e646f6du64,
            v2: k0 ^ 0x6c7967656e657261u64,
            v3: k1 ^ 0x7465646279746573u64,
            length: 0,
        }
    }

    // Helper function for SipHash rounds
    #[inline]
    fn sip_round(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
        *v0 = v0.wrapping_add(*v1);
        *v1 = v1.rotate_left(13);
        *v1 ^= *v0;
        *v0 = v0.rotate_left(32);
        *v2 = v2.wrapping_add(*v3);
        *v3 = v3.rotate_left(16);
        *v3 ^= *v2;
        *v0 = v0.wrapping_add(*v3);
        *v3 = v3.rotate_left(21);
        *v3 ^= *v0;
        *v2 = v2.wrapping_add(*v1);
        *v1 = v1.rotate_left(17);
        *v1 ^= *v2;
        *v2 = v2.rotate_left(32);
    }

    // Simplified c_rounds for SipHash13
    #[inline]
    fn c_rounds(v0: &mut u64, v1: &mut u64, v2: &mut u64, v3: &mut u64) {
        Self::sip_round(v0, v1, v2, v3);
    }
}

impl RngCore for SipHash13Rng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    fn next_u64(&mut self) -> u64 {
        // Increment length to simulate processing more data
        self.length = self.length.wrapping_add(8);

        // Mix in the length to the state
        self.v3 ^= self.length;
        Self::c_rounds(&mut self.v0, &mut self.v1, &mut self.v2, &mut self.v3);
        self.v0 ^= self.length;

        // Finalize and return the hash
        let result = self.v0 ^ self.v1 ^ self.v2 ^ self.v3;

        // Re-initialize state for the next call, but with a new seed based on the result
        // This is a simplified approach for a PRNG, not a true hasher.
        // For now, let's just update the state based on the result to make it change.
        self.v0 = self.v0.wrapping_add(result);
        self.v1 = self.v1.wrapping_add(result.rotate_left(1));
        self.v2 = self.v2.wrapping_add(result.rotate_left(2));
        self.v3 = self.v3.wrapping_add(result.rotate_left(3));

        result
    }

    fn fill_bytes(&mut self, dst: &mut [u8]) {
        for chunk in dst.chunks_mut(8) {
            let val = self.next_u64();
            chunk.copy_from_slice(&val.to_ne_bytes()[..chunk.len()]);
        }
    }
}

impl Rng for SipHash13Rng {}
