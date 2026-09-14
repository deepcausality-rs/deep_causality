/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The point of the move: a distribution and the density that describes it, from one crate.
//!
//! Before the split, `stats` knew what a Gaussian *is* — `gaussian_log_density` — and could not
//! draw from one, while `rand` could draw from one and could not say what it was. Neither crate
//! could express "sample, then score the sample", which is the shape every importance-weighting,
//! sequential-Monte-Carlo and MCMC-diagnostic algorithm takes.

use deep_causality_stats::{Distribution, Normal, StandardNormal, Xoshiro256, gaussian_log_density};

#[test]
fn a_draw_can_be_scored_by_its_own_density() {
    let mut rng = Xoshiro256::from_seed(7);

    // Draw from the distribution ...
    let z: f64 = StandardNormal.sample(&mut rng);
    // ... and score it under the matching density. One crate, one import.
    let log_p = gaussian_log_density(z, 0.0, 1.0).expect("unit variance is positive");

    // log N(z | 0, 1) = -0.5 * ln(2 pi) - z^2 / 2
    let expected = -0.5 * (2.0 * std::f64::consts::PI).ln() - z * z / 2.0;
    assert!(
        (log_p - expected).abs() < 1e-12,
        "density disagreed with its closed form: {log_p} vs {expected}"
    );
}

#[test]
fn a_shifted_distribution_scores_against_its_own_parameters() {
    let mut rng = Xoshiro256::from_seed(11);
    let (mean, sigma) = (3.0f64, 2.0f64);
    let d = Normal::new(mean, sigma).expect("a positive standard deviation");

    let x: f64 = d.sample(&mut rng);
    let log_p = gaussian_log_density(x, mean, sigma * sigma).expect("positive variance");

    // The density peaks at the mean, so a draw scores no higher than the mean does.
    let at_mean = gaussian_log_density(mean, mean, sigma * sigma).expect("positive variance");
    assert!(log_p <= at_mean, "no point scores above the mode");
}
