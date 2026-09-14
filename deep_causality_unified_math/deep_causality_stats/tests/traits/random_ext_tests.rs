/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The numerical draw over a generator.
//!
//! These came from `deep_causality_rand`'s generator suites, where they tested the numerical path
//! rather than the generator. The generator tests that remained there draw words; a draw on
//! `[0, 1)` is a real number and belongs with the distributions.

use deep_causality_rand::{Distribution, Xoshiro256};
use deep_causality_stats::{RandomExt, StandardUniform};

#[test]
fn random_f64_lands_in_the_unit_interval() {
    let mut rng = Xoshiro256::new();
    let val: f64 = rng.random();
    assert!(
        (0.0..1.0).contains(&val),
        "random() for f64 should produce a value in [0.0, 1.0)"
    );
}

#[test]
fn random_iter_f64_varies_and_stays_in_range() {
    let mut rng = Xoshiro256::new();
    let mut iter = rng.random_iter::<f64>();
    let val1 = iter.next().unwrap();
    let val2 = iter.next().unwrap();
    assert_ne!(val1, val2, "random_iter should produce different values");
    assert!((0.0..1.0).contains(&val1));
}

#[test]
fn map_random_composes_over_the_numerical_draw() {
    let mut rng = Xoshiro256::new();
    let mapped = rng.map_random(|x: f64| x as f32);
    let val: f32 = mapped.sample(&mut rng);
    assert!(
        (0.0..1.0).contains(&val),
        "a mapped f64 -> f32 draw should stay in [0.0, 1.0)"
    );
}

#[test]
fn sample_iter_over_the_uniform_varies() {
    let mut rng = Xoshiro256::new();
    let mut iter = StandardUniform.sample_iter(&mut rng);
    let val1: f64 = iter.next().unwrap();
    let val2: f64 = iter.next().unwrap();
    assert_ne!(val1, val2);
    assert!((0.0..1.0).contains(&val1));
}

#[test]
fn the_extension_reaches_every_generator() {
    // Blanket-implemented, so a generator gains it by importing the trait and nothing else.
    fn draw<R: deep_causality_rand::Rng>(rng: &mut R) -> f64 {
        rng.random()
    }
    let mut rng = Xoshiro256::from_seed(7);
    assert!((0.0..1.0).contains(&draw(&mut rng)));
}
