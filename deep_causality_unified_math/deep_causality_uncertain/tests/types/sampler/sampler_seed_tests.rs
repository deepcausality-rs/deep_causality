/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::{Uncertain, clear_sampler_seed, seed_sampler};

// Two distinct `Uncertain` values over the same distribution, sampled under the same seed, must
// produce identical streams — the seed governs both the index selection and the distribution draw.
#[test]
fn same_seed_reproduces_stream() {
    seed_sampler(0xC0FFEE);
    let a = Uncertain::<f64>::normal(1.0, 0.03);
    let first = a.take_samples(64).expect("draw a");

    seed_sampler(0xC0FFEE);
    let b = Uncertain::<f64>::normal(1.0, 0.03);
    let second = b.take_samples(64).expect("draw b");

    clear_sampler_seed();
    assert_eq!(first, second, "same seed must reproduce the sample stream");
}

// A single-sample draw is reproducible under the same seed as well.
#[test]
fn same_seed_reproduces_single_sample() {
    seed_sampler(7);
    let a = Uncertain::<f64>::normal(0.0, 1.0);
    let first = a.sample().expect("draw a");

    seed_sampler(7);
    let b = Uncertain::<f64>::normal(0.0, 1.0);
    let second = b.sample().expect("draw b");

    clear_sampler_seed();
    assert_eq!(first, second);
}

// Different seeds yield different streams (a 64-sample f64 collision is astronomically unlikely).
#[test]
fn different_seeds_diverge() {
    seed_sampler(1);
    let a = Uncertain::<f64>::normal(1.0, 0.03);
    let first = a.take_samples(64).expect("draw a");

    seed_sampler(2);
    let b = Uncertain::<f64>::normal(1.0, 0.03);
    let second = b.take_samples(64).expect("draw b");

    clear_sampler_seed();
    assert_ne!(first, second, "distinct seeds must diverge");
}

// Clearing the seed restores OS-entropy sampling without panicking, and a draw still succeeds.
#[test]
fn clear_seed_restores_default_and_samples() {
    seed_sampler(42);
    clear_sampler_seed();
    let u = Uncertain::<f64>::normal(0.0, 1.0);
    assert!(u.sample().is_ok());
}
