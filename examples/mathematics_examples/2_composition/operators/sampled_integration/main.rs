/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Operators: integrating by quadrature, by sampling, and differentiating both
//!
//! One integral, `I(θ) = ∫₀¹ e^(θx) dx`, computed three ways that share nothing but the
//! integrand:
//!
//! ```text
//! quadrature        Simpson's rule on a fixed grid          error ~ n^-4, deterministic
//! Monte Carlo       the mean of f at uniform draws          error ~ n^-1/2, random
//! quasi-Monte Carlo the mean of f at a Sobol sequence       error ~ n^-1, deterministic
//! ```
//!
//! calculus` supplies the quadrature fold, `rand` the generator and the Sobol sequence, `stats` the
//! uniform distribution, and `haft` collects and folds the samples.
//!
//! Every estimator above is a fold over a `Scalar`, and `Dual` is a `Scalar`.
//! Seed `θ` as a `Dual::variable` and run the *same* estimators: each sample carries its own derivative,
//! and averaging the `ε` channel estimates `dI/dθ` from exactly the draws that estimated `I(θ)`.
//! No second pass, no difference quotient, no re-derivation of the integrand.
//!
//! That is the pathwise derivative, and here it arrives because differentiation and sampling
//! are separate values that compose rather than one library's feature.

use deep_causality_algebra::Real;
use deep_causality_calculus::quadrature;
use deep_causality_haft::{Collectable, Foldable, VecWitness};
use deep_causality_num::{lift, lift_usize, lower};
use deep_causality_num_dual::Dual;
use deep_causality_rand::{SobolSequence, Xoshiro256};
use deep_causality_stats::{Distribution, StandardUniform};

/// The parameter the integral is differentiated with respect to.
const THETA: f64 = 1.3;
/// Draws per sampled estimator, and panels for the quadrature.
const N: usize = 4096;
/// Fixed so the run reproduces.
const SEED: u64 = 0x5EED_1234;

/// The working scalar. Every estimate below carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let theta = lift::<FloatType>(THETA);
    print_header(theta);

    // Closed forms, to measure every estimator against.
    let exact_i = (Real::exp(theta) - lift::<FloatType>(1.0)) / theta;
    let exact_di =
        (theta * Real::exp(theta) - Real::exp(theta) + lift::<FloatType>(1.0)) / (theta * theta);

    // ---------------------------------------------------------------------
    // 1. Simpson's rule: a deterministic fold over a fixed grid.
    // ---------------------------------------------------------------------
    let by_quadrature = quadrature(
        |x: FloatType| Real::exp(theta * x),
        lift::<FloatType>(0.0),
        lift::<FloatType>(1.0),
        N,
    );

    // ---------------------------------------------------------------------
    // 2. Monte Carlo: the mean of the integrand at uniform draws.
    // ---------------------------------------------------------------------
    // `Collectable::collect` lands the draws in a container and `Foldable::fold` reduces them,
    // so the estimator is the witness vocabulary rather than a hand-rolled loop.
    let mut rng = Xoshiro256::from_seed(SEED);
    let draws: Vec<FloatType> =
        VecWitness::collect((0..N).map(|_| StandardUniform.sample(&mut rng)));
    let by_monte_carlo = mean(&draws, |x| Real::exp(theta * x));

    // ---------------------------------------------------------------------
    // 3. Quasi-Monte Carlo: the same mean over a low-discrepancy sequence.
    // ---------------------------------------------------------------------
    // Sobol points are deterministic and spread more evenly than random ones, which is what
    // buys the better error rate for a smooth integrand.
    let sobol = SobolSequence::new(1)?;
    let sobol_points: Vec<FloatType> =
        VecWitness::collect((0..N as u64).map(|i| sobol.coordinate::<FloatType>(i, 0)));
    let by_qmc = mean(&sobol_points, |x| Real::exp(theta * x));

    print_estimates(exact_i, by_quadrature, by_monte_carlo, by_qmc);

    // ---------------------------------------------------------------------
    // 4. The same estimators over `Dual`: dI/dθ from the very same samples.
    // ---------------------------------------------------------------------
    // The draw `x` is a constant -- the randomness does not depend on θ -- so only θ is seeded.
    // Each evaluation then carries `d/dθ e^(θx) = x e^(θx)` in its ε channel, and the mean of
    // those estimates `∫₀¹ x e^(θx) dx`, which is exactly `dI/dθ`.
    let dual_theta = Dual::variable(theta);

    let quad_dual = quadrature(
        |x: Dual<FloatType>| (dual_theta * x).exp(),
        Dual::constant(lift::<FloatType>(0.0)),
        Dual::constant(lift::<FloatType>(1.0)),
        N,
    );
    let mc_dual = mean_dual(&draws, dual_theta);
    let qmc_dual = mean_dual(&sobol_points, dual_theta);

    print_derivatives(
        exact_di,
        quad_dual.derivative(),
        mc_dual.derivative(),
        qmc_dual.derivative(),
    );

    // One sweep carried both answers in every case: the value channel still holds the integral.
    print_both_channels(quad_dual.value(), quad_dual.derivative());
    assert!(Real::abs(quad_dual.value() - exact_i) < lift::<FloatType>(1e-9));
    assert!(Real::abs(quad_dual.derivative() - exact_di) < lift::<FloatType>(1e-9));

    // The sampled estimators are unbiased, not exact. Monte Carlo error falls as 1/sqrt(n),
    // so at N = 4096 one standard error is about 1/64 = 1.6% of the integrand's spread. The
    // bound below is roughly three of those: loose enough that a different seed still passes,
    // tight enough that a broken estimator does not.
    let mc_tolerance = lift::<FloatType>(3.0) / Real::sqrt(lift_usize::<FloatType>(N));
    assert!(Real::abs(by_monte_carlo - exact_i) / exact_i < mc_tolerance);
    assert!(Real::abs(mc_dual.derivative() - exact_di) / exact_di < mc_tolerance);

    print_footer();
    Ok(())
}

/// The sample mean of `f` over `xs`, through `Foldable`.
fn mean(xs: &[FloatType], f: impl Fn(FloatType) -> FloatType) -> FloatType {
    let total = VecWitness::fold(xs.to_vec(), lift::<FloatType>(0.0), |acc, x| acc + f(x));
    total / lift_usize::<FloatType>(xs.len())
}

/// The same mean, evaluated over `Dual` so the ε channel accumulates alongside the value.
fn mean_dual(xs: &[FloatType], theta: Dual<FloatType>) -> Dual<FloatType> {
    let zero = Dual::constant(lift::<FloatType>(0.0));
    let total = VecWitness::fold(xs.to_vec(), zero, |acc, x| {
        acc + (theta * Dual::constant(x)).exp()
    });
    total / Dual::constant(lift_usize::<FloatType>(xs.len()))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn print_header(theta: FloatType) {
    println!("=== Operators: quadrature, sampling, and differentiating through both ===\n");
    println!("  I(θ) = ∫₀¹ e^(θx) dx   with θ = {}", lower(theta));
    println!("  {N} panels / draws, seed {SEED:#x}");
    println!("  Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_estimates(exact: FloatType, quad: FloatType, mc: FloatType, qmc: FloatType) {
    println!("--- 1. Three estimators for I(θ) ---");
    println!(
        "  {:<26} {:>14}  {:>12}",
        "estimator", "value", "rel. error"
    );
    row("closed form", exact, exact);
    row("Simpson quadrature", quad, exact);
    row("Monte Carlo", mc, exact);
    row("quasi-Monte Carlo (Sobol)", qmc, exact);
}

fn print_derivatives(exact: FloatType, quad: FloatType, mc: FloatType, qmc: FloatType) {
    println!("\n--- 2. dI/dθ, from the same folds run over Dual ---");
    println!(
        "  {:<26} {:>14}  {:>12}",
        "estimator", "value", "rel. error"
    );
    row("closed form", exact, exact);
    row("Simpson quadrature", quad, exact);
    row("Monte Carlo", mc, exact);
    row("quasi-Monte Carlo (Sobol)", qmc, exact);
}

fn row(label: &str, value: FloatType, exact: FloatType) {
    let rel = lower(Real::abs(value - exact) / exact);
    println!("  {:<26} {:>14.9}  {:>12.2e}", label, lower(value), rel);
}

fn print_both_channels(value: FloatType, derivative: FloatType) {
    println!("\n--- 3. One sweep, two channels ---");
    println!(
        "  the Dual quadrature's value channel      = {:.9}",
        lower(value)
    );
    println!(
        "  the same sweep's ε channel               = {:.9}",
        lower(derivative)
    );
    println!("  I(θ) and dI/dθ came out of a single pass over the integrand.");
}

fn print_footer() {
    println!("\n--- What composed ---");
    println!("  calculus supplied the quadrature fold, rand the generator and Sobol sequence,");
    println!("  stats the uniform draw, haft the collect and the fold, num_dual the tangent.");
    println!("  Differentiation reached through the sampled estimator because `Dual` is a");
    println!("  `Scalar` and every estimator was written generic over `Scalar`.");
}
