/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The 64-bit FNV-1a hash behind the snapshot checksum and the world fingerprint.
//!
//! This is **corruption detection, not a security boundary**: the threat model is truncated
//! writes, bit rot, and partial copies, not adversaries. FNV-1a is a dozen lines, needs no
//! external dependency, and detects the accidental-damage cases the snapshot workflow cares
//! about. Nobody should mistake it for a cryptographic integrity guarantee.

/// FNV-1a 64-bit offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The 64-bit FNV-1a digest of `bytes`.
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Digest caller-supplied world-description bytes into the fingerprint a snapshot stores.
/// The input is a seam: today an example hashes its own constants; when the canonical config
/// serialization lands, that serialization becomes the input without changing the container.
pub fn fingerprint64(world_description: &[u8]) -> u64 {
    fnv1a64(world_description)
}
