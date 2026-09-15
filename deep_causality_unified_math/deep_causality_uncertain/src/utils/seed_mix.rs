/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The address of a draw.
//!
//! A leaf's draw is a function of three numbers and nothing else: the session's seed, the sample
//! index, and the leaf's ordinal within its tree. [`draw_seed`] combines them into the seed of the
//! generator that produces that one draw, which is what makes a draw reproducible without storing
//! it.

/// `⌊2^64 / φ⌋`, where `φ` is the golden ratio.
///
/// Verified rather than quoted: `11400714819323198485`, which is odd, so multiplication by it is a
/// bijection modulo `2^64` and no input collapses onto another. It is the increment `SplitMix64`
/// uses, and [`deep_causality_rand::Xoshiro256::from_seed`] uses it for the same purpose one layer
/// below.
const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;

/// The 64-bit finalizer from Austin Appleby's MurmurHash3 (2011), known there as `fmix64`.
///
/// Deliberately a different mixer from the `SplitMix64` finalizer that
/// `Xoshiro256::from_seed` applies to the result of [`draw_seed`], so the two stages are not the
/// same function run twice.
#[inline]
const fn fmix64(mut z: u64) -> u64 {
    z ^= z >> 33;
    z = z.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    z ^= z >> 33;
    z = z.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    z ^= z >> 33;
    z
}

/// The generator seed for one draw: leaf `ordinal`, at sample `index`, in a session seeded `seed`.
///
/// # The three arguments are not interchangeable
///
/// Each enters at a distinct position — `seed` additively, `index` through a multiply, `ordinal`
/// through a multiply and a 32-bit rotation, in a later round. Swapping any two therefore changes
/// the result, which matters because the three are all `u64` and a call site that transposes them
/// would otherwise sample a valid but wrong stream in silence. The test pins all three
/// transpositions against literal values.
///
/// # Why the ordinal and not the node's address
///
/// `ConstTree` identifies a node by `Arc::as_ptr`, which is a heap address: it differs between
/// runs and between two structurally identical trees. A draw derived from one could not be
/// reproduced from a recorded seed, which is the single property this function exists to provide.
/// The ordinal comes from a deterministic traversal instead, and the address is used only to
/// deduplicate shared nodes within that traversal.
///
/// # Construction
///
/// Two rounds, each adding `GOLDEN` before mixing, as `SplitMix64`'s step does. Adding it also
/// removes the fixed point the bare finalizer has at zero: `fmix64(0) == 0`, so without the
/// addition an all-zero address would return zero.
#[inline]
pub const fn draw_seed(seed: u64, index: u64, ordinal: u64) -> u64 {
    let z = fmix64(seed.wrapping_add(GOLDEN) ^ index.wrapping_mul(GOLDEN));
    fmix64(z.wrapping_add(GOLDEN) ^ ordinal.wrapping_mul(GOLDEN).rotate_left(32))
}
