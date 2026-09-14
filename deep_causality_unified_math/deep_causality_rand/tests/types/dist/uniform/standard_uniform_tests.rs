/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::{Distribution, Rng, RngCore, StandardBool, StandardUniform, StandardWord};

struct MockRng {
    val_u32: u32,
    val_u64: u64,
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        self.val_u32
    }
    fn next_u64(&mut self) -> u64 {
        self.val_u64
    }
    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!()
    }
}

impl Rng for MockRng {}

#[test]
fn test_standard_word_u32() {
    let mut rng = MockRng {
        val_u32: 123,
        val_u64: 0,
    };
    let sample: u32 = StandardWord.sample(&mut rng);
    assert_eq!(sample, 123);
}

#[test]
fn test_standard_word_u64() {
    let mut rng = MockRng {
        val_u32: 0,
        val_u64: 456,
    };
    let sample: u64 = StandardWord.sample(&mut rng);
    assert_eq!(sample, 456);
}

#[test]
fn test_standard_bool() {
    // The old parity rule (`next_u64() % 2 == 0`) read one bit's worth of information from a
    // whole word. `StandardBool` takes a bit directly, so the mock's value is chosen by its top
    // bit rather than its low one.
    let mut rng_high = MockRng {
        val_u32: 0,
        val_u64: 1 << 63,
    };
    let sample_true: bool = StandardBool.sample(&mut rng_high);
    assert!(sample_true, "a word with the top bit set yields true");

    let mut rng_low = MockRng {
        val_u32: 0,
        val_u64: 0,
    };
    let sample_false: bool = StandardBool.sample(&mut rng_low);
    assert!(!sample_false, "a word with the top bit clear yields false");
}

#[test]
fn test_derived_traits() {
    let s1 = StandardUniform;
    let s2 = s1;
    let s3 = s1; // Copy
    assert_eq!(format!("{:?}", s1), "StandardUniform");
    assert_eq!(format!("{:?}", s2), "StandardUniform");
    assert_eq!(format!("{:?}", s3), "StandardUniform");
}
