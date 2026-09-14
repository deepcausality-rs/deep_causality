/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
//! A generic function that draws from a distribution names the scalar's capabilities only.
//!
//! This is the requirement the re-export exists for: a crate that wants distributions should not
//! have to spell a bound about how those distributions are implemented, nor declare a second
//! dependency to name the generator trait. Before the retrofit the signature below needed
//! `where StandardUniform: Distribution<S>`; naming `RandScalar` says the same thing about the
//! scalar instead, which is what the caller actually knows.

use deep_causality_num::{lift, lift_count};
use deep_causality_stats::{RandScalar, RandomExt, Xoshiro256};

fn monte_carlo<S: RandScalar>(samples: u64, seed: u64) -> (S, usize) {
    let mut rng = Xoshiro256::from_seed(seed);
    let draws: Vec<S> = (0..samples).map(|_| rng.random::<S>()).collect();
    let bytes = core::mem::size_of_val(draws.as_slice());
    let sum = draws.iter().fold(lift::<S>(0.0), |acc, &x| acc + x * x);
    (sum / lift_count::<S>(samples), bytes)
}

#[test]
fn a_generic_draw_needs_no_distribution_clause() {
    let (v, _) = monte_carlo::<f64>(10_000, 7);
    assert!((v - 1.0 / 3.0).abs() < 0.02, "got {v}");
    let (v32, _) = monte_carlo::<f32>(10_000, 7);
    assert!((v32 - 1.0f32 / 3.0).abs() < 0.02, "got {v32}");
}
