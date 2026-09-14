/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The three-way split: a sampler answers for one kind of mathematical object.
//!
//! `StandardUniform` samples real scalars, `StandardWord` machine words, `StandardBool` Booleans.
//! Keeping all three under one type is what made a blanket implementation over the algebra tower
//! `error[E0119]`, so these tests hold the separation in place.

use deep_causality_rand::{Distribution, Rng, StandardBool, StandardWord, Xoshiro256};

const SEED: u64 = 0x5EED_1234;

#[test]
fn standard_word_yields_a_u64() {
    let mut g = Xoshiro256::from_seed(SEED);
    let a: u64 = StandardWord.sample(&mut g);
    let b: u64 = StandardWord.sample(&mut g);
    // Two consecutive draws from a non-degenerate generator differ; a constant body fails here.
    assert_ne!(a, b, "consecutive u64 draws must differ");
}

#[test]
fn standard_word_yields_a_u32() {
    let mut g = Xoshiro256::from_seed(SEED);
    let a: u32 = StandardWord.sample(&mut g);
    let b: u32 = StandardWord.sample(&mut g);
    assert_ne!(a, b, "consecutive u32 draws must differ");
}

#[test]
fn standard_word_spans_the_full_width() {
    // A word draw that only ever fills the low bits is a defect a range check would miss.
    let mut g = Xoshiro256::from_seed(SEED);
    let mut seen_high = false;
    for _ in 0..64 {
        let v: u64 = StandardWord.sample(&mut g);
        if v >> 32 != 0 {
            seen_high = true;
            break;
        }
    }
    assert!(seen_high, "no draw in 64 set a bit above position 31");
}

#[test]
fn standard_bool_yields_both_values() {
    let mut g = Xoshiro256::from_seed(SEED);
    let (mut saw_true, mut saw_false) = (false, false);
    for _ in 0..64 {
        if StandardBool.sample(&mut g) {
            saw_true = true;
        } else {
            saw_false = true;
        }
    }
    assert!(saw_true && saw_false, "a constant Boolean body passes nothing");
}

#[test]
fn standard_bool_is_unbiased() {
    // The spec's band: 45% to 55% over 10 000 draws.
    let mut g = Xoshiro256::from_seed(SEED);
    let trials = 10_000;
    let hits = (0..trials).filter(|_| StandardBool.sample(&mut g)).count();
    let fraction = hits as f64 / trials as f64;
    assert!(
        (0.45..=0.55).contains(&fraction),
        "true fraction {fraction} outside [0.45, 0.55]"
    );
}

#[test]
fn the_rng_methods_reach_their_own_samplers() {
    let mut g = Xoshiro256::from_seed(SEED);
    let word: u64 = g.random_word();
    let mut g2 = Xoshiro256::from_seed(SEED);
    let direct: u64 = StandardWord.sample(&mut g2);
    assert_eq!(word, direct, "Rng::random_word must delegate to StandardWord");

    let mut g3 = Xoshiro256::from_seed(SEED);
    let flag = g3.random_boolean();
    let mut g4 = Xoshiro256::from_seed(SEED);
    let direct_flag = StandardBool.sample(&mut g4);
    assert_eq!(flag, direct_flag, "Rng::random_boolean must delegate to StandardBool");
}

#[test]
fn a_seeded_generator_is_reproducible() {
    // Determinism is what makes every other test in this file actionable on failure.
    let mut a = Xoshiro256::from_seed(SEED);
    let mut b = Xoshiro256::from_seed(SEED);
    for _ in 0..32 {
        let x: u64 = StandardWord.sample(&mut a);
        let y: u64 = StandardWord.sample(&mut b);
        assert_eq!(x, y);
    }
}
