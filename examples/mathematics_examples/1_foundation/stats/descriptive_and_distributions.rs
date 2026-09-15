/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `deep_causality_stats`: the API surface
//!
//! The crate holds the statistics of a sample, and the
//! distributions those statistics describe.
//!
//! ```text
//! mean / variance / std_dev      the moments of a slice
//! pearson                        linear correlation between two slices
//! entropy                        the information content of a distribution
//! Normal / Exponential / ...     the shaped distributions, sampled through `Distribution`
//! ```
//!
//! Every function returns a `Result` rather than panicking on an empty slice or a length
//! mismatch, and every one is generic over the working scalar.
//!
//! The reason the distributions live here rather than in `rand` is worth stating: a
//! distribution *is* its density and its moments, and both are statistics. `rand` keeps what
//! is genuinely entropy -- a source of bits, the raw machine word, the Boolean draw, the
//! Sobol sequence, and a value uniform over a range.

use deep_causality_algebra::Real;
use deep_causality_num::{lift, lift_usize, lower};
use deep_causality_rand::Xoshiro256;
use deep_causality_stats::{
    Distribution, Normal, StandardUniform, mean, pearson, std_dev, variance,
};

/// Draws taken from each distribution.
const DRAWS: usize = 10_000;
/// Fixed so the run reproduces.
const SEED: u64 = 0xC0FFEE;

/// The working scalar. Every sample and every moment carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. Moments of a known sample.
    // ---------------------------------------------------------------------
    // 1..=10 has mean 5.5, and sample variance (n-1 denominator) 55/6 = 9.1667.
    let sample: Vec<FloatType> = (1..=10).map(lift_usize::<FloatType>).collect();
    let m = mean(&sample)?;
    let v = variance(&sample)?;
    let sd = std_dev(&sample)?;
    print_moments(m, v, sd);
    assert!(Real::abs(m - lift::<FloatType>(5.5)) < lift::<FloatType>(1e-12));

    // An empty slice is an error, not a panic or a silent NaN.
    print_empty_case(mean(&[] as &[FloatType]).err().map(|e| e.to_string()));

    // ---------------------------------------------------------------------
    // 2. Correlation between two slices.
    // ---------------------------------------------------------------------
    // A perfect line has r = 1; reversing one side gives r = -1.
    let doubled: Vec<FloatType> = sample.iter().map(|&x| x * lift::<FloatType>(2.0)).collect();
    let reversed: Vec<FloatType> = sample.iter().rev().copied().collect();
    let (r_pos, n_pos) = pearson(&sample, &doubled)?;
    let (r_neg, _) = pearson(&sample, &reversed)?;
    print_correlation(r_pos, n_pos, r_neg);
    assert!(Real::abs(r_pos - lift::<FloatType>(1.0)) < lift::<FloatType>(1e-12));
    assert!(Real::abs(r_neg + lift::<FloatType>(1.0)) < lift::<FloatType>(1e-12));

    // ---------------------------------------------------------------------
    // 3. Sampling a distribution, and recovering its parameters.
    // ---------------------------------------------------------------------
    // The statistics above and the distributions below are the same crate for a reason: a
    // distribution is defined by the moments the estimators here measure.
    let mut rng = Xoshiro256::from_seed(SEED);
    let normal = Normal::new(lift::<FloatType>(3.0), lift::<FloatType>(2.0))?;
    let draws: Vec<FloatType> = (0..DRAWS).map(|_| normal.sample(&mut rng)).collect();
    let est_mean = mean(&draws)?;
    let est_sd = std_dev(&draws)?;
    print_normal(
        lift::<FloatType>(3.0),
        lift::<FloatType>(2.0),
        est_mean,
        est_sd,
    );

    // The standard error of the mean is sigma/sqrt(n); three of those is a safe bound that
    // still fails if the sampler is wrong.
    let se = lift::<FloatType>(2.0) / Real::sqrt(lift_usize::<FloatType>(DRAWS));
    assert!(Real::abs(est_mean - lift::<FloatType>(3.0)) < lift::<FloatType>(3.0) * se);

    // ---------------------------------------------------------------------
    // 4. The uniform on [0, 1), whose mean is 1/2 and variance 1/12.
    // ---------------------------------------------------------------------
    let uniform: Vec<FloatType> = (0..DRAWS)
        .map(|_| StandardUniform.sample(&mut rng))
        .collect();
    let u_mean = mean(&uniform)?;
    let u_var = variance(&uniform)?;
    print_uniform(u_mean, u_var);
    let u_se = lift::<FloatType>(1.0)
        / (Real::sqrt(lift::<FloatType>(12.0)) * Real::sqrt(lift_usize::<FloatType>(DRAWS)));
    assert!(Real::abs(u_mean - lift::<FloatType>(0.5)) < lift::<FloatType>(4.0) * u_se);

    print_footer();
    Ok(())
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== deep_causality_stats: descriptive statistics and distributions ===\n");
    println!("  Precision: {}", core::any::type_name::<FloatType>());
    println!("  {DRAWS} draws per distribution, seed {SEED:#x}");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_moments(mean: FloatType, variance: FloatType, std_dev: FloatType) {
    println!("\n--- 1. Moments of 1..=10 ---");
    println!("  mean     = {:.6}   (exactly 5.5)", lower(mean));
    println!(
        "  variance = {:.6}   (55/6, the n-1 denominator)",
        lower(variance)
    );
    println!("  std_dev  = {:.6}", lower(std_dev));
}

fn print_empty_case(error: Option<String>) {
    match error {
        Some(e) => println!("  mean of an empty slice is an error: {e}"),
        None => println!("  mean of an empty slice returned a value"),
    }
}

fn print_correlation(r_pos: FloatType, n: usize, r_neg: FloatType) {
    println!("\n--- 2. Pearson correlation ---");
    println!("  r(x, 2x)        = {:.6}   over {n} pairs", lower(r_pos));
    println!("  r(x, reverse x) = {:.6}", lower(r_neg));
}

fn print_normal(mu: FloatType, sigma: FloatType, est_mean: FloatType, est_sd: FloatType) {
    println!("\n--- 3. Normal(mu, sigma), sampled and measured back ---");
    println!("  {:<12} {:>12} {:>12}", "parameter", "set", "recovered");
    println!(
        "  {:<12} {:>12.4} {:>12.4}",
        "mean",
        lower(mu),
        lower(est_mean)
    );
    println!(
        "  {:<12} {:>12.4} {:>12.4}",
        "std dev",
        lower(sigma),
        lower(est_sd)
    );
}

fn print_uniform(mean: FloatType, variance: FloatType) {
    println!("\n--- 4. StandardUniform on [0, 1) ---");
    println!("  mean     = {:.6}   (1/2)", lower(mean));
    println!("  variance = {:.6}   (1/12 = 0.083333)", lower(variance));
}

fn print_footer() {
    println!("\n--- Why the distributions live here ---");
    println!("  A distribution is defined by its density and its moments, and both are");
    println!("  statistics. `rand` keeps what is genuinely entropy: a source of bits, the raw");
    println!("  machine word, the Boolean draw, the Sobol sequence, a value uniform over a");
    println!("  range. The estimators above and the samplers they measure are one crate.");
}
