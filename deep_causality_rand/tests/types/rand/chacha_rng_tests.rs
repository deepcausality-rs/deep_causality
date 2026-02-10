/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
#[cfg(feature = "aead-random")]
use deep_causality_rand::RngCore;
#[cfg(feature = "aead-random")]
use deep_causality_rand::types::ChaCha20Rng;

#[test]
#[cfg(feature = "aead-random")]
fn test_chacha_rng_determinism() {
    let seed = [1u8; 32];
    let mut rng1 = ChaCha20Rng::from_seed(seed);
    let mut rng2 = ChaCha20Rng::from_seed(seed);

    let val1 = rng1.next_u64();
    let val2 = rng2.next_u64();
    assert_eq!(
        val1, val2,
        "RNGs with same seed should produce same first output"
    );

    let mut buf1 = [0u8; 32];
    let mut buf2 = [0u8; 32];
    rng1.fill_bytes(&mut buf1);
    rng2.fill_bytes(&mut buf2);
    assert_eq!(
        buf1, buf2,
        "RNGs with same seed should produce same sequence"
    );
}

#[test]
#[cfg(feature = "aead-random")]
fn test_chacha_rng_diff_seeds() {
    let seed1 = [1u8; 32];
    let seed2 = [2u8; 32];
    let mut rng1 = ChaCha20Rng::from_seed(seed1);
    let mut rng2 = ChaCha20Rng::from_seed(seed2);

    assert_ne!(
        rng1.next_u64(),
        rng2.next_u64(),
        "RNGs with different seeds should produce different outputs"
    );
}

#[test]
#[cfg(feature = "aead-random")]
fn test_reseed() {
    let seed = [1u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);
    let val1 = rng.next_u64();

    // Reseed with same seed -> should restart sequence
    rng.reseed(seed);
    let val2 = rng.next_u64();
    assert_eq!(
        val1, val2,
        "Reseeding with same seed should restart sequence"
    );

    // Reseed with distinct seed
    rng.reseed([2u8; 32]);
    let val3 = rng.next_u64();
    assert_ne!(val1, val3, "Reseeding with new seed should change output");
}

#[test]
#[cfg(feature = "aead-random")]
fn test_large_buffer_refill() {
    let seed = [42u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    // Consume more than 1024 bytes (internal buffer size) to trigger refill
    let mut buf = vec![0u8; 2000];
    rng.fill_bytes(&mut buf);

    // Check that we got non-zero data (extremely unlikely to be all zeros)
    assert!(
        buf.iter().any(|&x| x != 0),
        "Large buffer should be filled with random data"
    );
}
