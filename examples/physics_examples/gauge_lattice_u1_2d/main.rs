/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # 2D U(1) Lattice Gauge Theory against its exact solution
//!
//! Two-dimensional U(1) lattice gauge theory is one of the few gauge theories that is exactly
//! solvable, which makes it the right place to test a lattice implementation. The average
//! plaquette in the infinite-volume limit is
//!
//! ```text
//! <P> = I_1(beta) / I_0(beta)
//! ```
//!
//! with `I_n` the modified Bessel functions of the first kind (Creutz, *Quarks, Gluons and
//! Lattices*, CUP 1983, ch. 8).
//!
//! The run therefore does the thing that statement is about: it puts a hot random field on the
//! lattice, thermalizes it with Metropolis sweeps at each `beta`, **measures** the average
//! plaquette over a run of configurations, and compares that measurement with `I_1/I_0`.
//!
//! Two supporting checks keep the comparison honest, and neither is mistaken for the main one:
//!
//! ```text
//! structural   an identity field has every plaquette equal to 1, at any beta
//! reference    two independent Bessel algorithms agree, so I_1/I_0 is itself trustworthy
//! ```
//!
//! The identity check exercises the plaquette *machinery* only: it holds for every `beta`, so it
//! says nothing about the thermodynamics. The Bessel cross-check validates the reference curve,
//! not the lattice. Only the sampled measurement tests both together.
//!
//! ## APIs Demonstrated
//! - `LatticeGaugeField::random`, `try_metropolis_sweep`, `try_average_plaquette`
//! - `CubicalComplex` with periodic boundaries

use deep_causality_algebra::Real;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift, lift_usize, lower};
use deep_causality_num_complex::Complex;
use deep_causality_rand::Xoshiro256;
use deep_causality_topology::{CubicalComplex, LatticeGaugeField, TopologyError, U1};
use std::sync::Arc;

/// Seed of the one generator every coupling draws from, so a run reproduces bit for bit.
const SEED: u64 = 1008;

/// Lattice side; the lattice is `SIDE x SIDE` with periodic boundaries in both directions.
const SIDE: usize = 8;
/// Metropolis sweeps discarded before any measurement, so the field forgets its hot start.
const THERMAL_SWEEPS: usize = 400;
/// Sweeps measured after thermalization. The statistical error falls as `1 / sqrt(N)`.
const MEASURE_SWEEPS: usize = 400;
/// Metropolis proposal width. Tuned so the acceptance rate lands near one half.
const EPSILON: FloatType = const_scalar_from_float!(FloatType, 0.55);
/// Couplings to test, spanning strong (`beta < 1`) to weak (`beta > 5`) coupling.
const BETA_VALUES: [f64; 6] = [0.5, 1.0, 2.0, 4.0, 6.0, 10.0];

/// How far a measured plaquette may sit from `I_1/I_0` and still count as agreement, per entry
/// of `BETA_VALUES`.
///
/// Measured, not guessed: 60 seeds (101..=160) at the sweep counts above gave, per coupling, the
/// mean deviation `m`, its standard deviation `s` and the largest `|deviation|` seen:
///
/// ```text
/// beta    m          s         max|dev|   band
///  0.5   -2.9e-3    1.41e-2    3.42e-2    0.050
///  1.0   -1.0e-5    1.32e-2    4.06e-2    0.050
///  2.0   -1.1e-3    8.4e-3     2.65e-2    0.035
///  4.0   +4.3e-4    4.1e-3     1.42e-2    0.020
///  6.0   -5.0e-3    7.8e-3     3.65e-2    0.045
/// 10.0   -5.5e-3    1.02e-2    4.33e-2    0.050
/// ```
///
/// Each band is `max(|m| + 3s, max|dev| + 0.005)` rounded up to the next `0.005`. The spread is
/// several times the naive `1 / sqrt(MEASURE_SWEEPS)` error because successive sweeps are
/// correlated. At `beta = 6` and `10` the tail is heavier than Gaussian and the mean sits below
/// zero, which is consistent with a hot start settling in a winding sector that the local update
/// rarely leaves. The exact
/// finite-volume correction on this lattice is below `4e-4` at every coupling, so it does not
/// account for the spread. At `beta = 10` the band stays below `1 - I_1/I_0 = 0.051`, so a field
/// stuck at the identity still fails.
const AGREEMENT_TOLERANCES: [f64; 6] = [0.050, 0.050, 0.035, 0.020, 0.045, 0.050];

/// Terms in the Bessel series; well past convergence for the `beta` range above.
const BESSEL_TERMS: usize = 200;
/// Starting order for Miller's backward recurrence.
const MILLER_ORDER: usize = 100;
/// How closely the two Bessel algorithms must agree for the reference to be trusted.
const REFERENCE_TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// Small whole numbers, declared once at the working type.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
/// Sweeps in the measurement run, at the working type, for the average.
const MEASURED: FloatType = const_scalar_from_int!(FloatType, MEASURE_SWEEPS as i128);

/// `f64` is the right precision here, and the reason is worth stating: the measurement is a
/// Monte Carlo average whose seed-to-seed spread is `4e-3` to `1.4e-2` at these run lengths
/// (see `AGREEMENT_TOLERANCES`). That is more than ten orders of magnitude above `f64` rounding, so extra
/// precision buys nothing the error bars would notice. `Float106` would only sharpen the
/// *reference* curve, which already agrees with itself to `1e-12`.
type FloatType = f64;

fn main() -> Result<(), TopologyError> {
    print_header();

    let lattice = Arc::new(CubicalComplex::new([SIDE, SIDE], [true, true]));
    print_setup();

    // The plaquette machinery, on its own. An identity field has every plaquette equal to the
    // identity, so <P> = 1 whatever beta is. That is a structural check and nothing more.
    print_structural(structural_check(&lattice)?);

    let mut generator = Xoshiro256::from_seed(SEED);
    let mut results = Vec::with_capacity(BETA_VALUES.len());
    for (&beta, &tolerance) in BETA_VALUES.iter().zip(AGREEMENT_TOLERANCES.iter()) {
        results.push(measure(&lattice, beta, lift(tolerance), &mut generator)?);
    }
    print_results(&results);

    let all_agree = results.iter().all(|r| r.agrees);
    let reference_sound = results.iter().all(|r| r.reference_agrees);
    print_summary(all_agree, reference_sound);

    Ok(())
}

/// One coupling: what the lattice measured, and what theory says it should be.
struct Measurement {
    beta: f64,
    /// The Monte Carlo average of the plaquette over `MEASURE_SWEEPS` configurations.
    measured: FloatType,
    /// `I_1(beta) / I_0(beta)` from the series expansion.
    exact: FloatType,
    /// The same ratio from Miller's backward recurrence, as a check on the reference itself.
    exact_miller: FloatType,
    /// Mean Metropolis acceptance over the measured sweeps.
    acceptance: f64,
    deviation: FloatType,
    /// The agreement band for this coupling, from `AGREEMENT_TOLERANCES`.
    tolerance: FloatType,
    agrees: bool,
    reference_agrees: bool,
}

/// Thermalizes a hot field at `beta`, then averages the plaquette over a run of configurations.
fn measure(
    lattice: &Arc<CubicalComplex<2, FloatType>>,
    beta: f64,
    tolerance: FloatType,
    generator: &mut Xoshiro256,
) -> Result<Measurement, TopologyError> {
    // Hot start: random links, the high-temperature configuration.
    let mut field: LatticeGaugeField<U1, 2, Complex<FloatType>, FloatType> =
        LatticeGaugeField::random(lattice.clone(), lift(beta), generator);

    for _ in 0..THERMAL_SWEEPS {
        field.try_metropolis_sweep(EPSILON, generator)?;
    }

    // Measurement run. Each sweep produces a new configuration; the plaquette is averaged over
    // all of them, which is what `<P>` means.
    let mut total = ZERO;
    let mut accepted = 0.0;
    for _ in 0..MEASURE_SWEEPS {
        accepted += field.try_metropolis_sweep(EPSILON, generator)?;
        total += field.try_average_plaquette()?;
    }
    let measured = total / MEASURED;

    let exact = bessel_ratio_series(lift(beta));
    let exact_miller = bessel_ratio_miller(lift(beta));
    let deviation = Real::abs(measured - exact);

    Ok(Measurement {
        beta,
        measured,
        exact,
        exact_miller,
        acceptance: accepted / MEASURE_SWEEPS as f64,
        deviation,
        tolerance,
        agrees: deviation < tolerance,
        reference_agrees: Real::abs(exact - exact_miller) < REFERENCE_TOLERANCE,
    })
}

/// An identity field has `<P> = 1` at any coupling. Exercises the plaquette sum, not the physics.
fn structural_check(
    lattice: &Arc<CubicalComplex<2, FloatType>>,
) -> Result<FloatType, TopologyError> {
    let field: LatticeGaugeField<U1, 2, Complex<FloatType>, FloatType> =
        LatticeGaugeField::identity(lattice.clone(), lift(BETA_VALUES[0]));
    field.try_average_plaquette()
}

/// `I_1(x) / I_0(x)` from the defining series `I_n(x) = sum_k (x/2)^(2k+n) / (k! (k+n)!)`.
fn bessel_ratio_series(x: FloatType) -> FloatType {
    let half_x = x / TWO;
    let mut i0 = ZERO;
    let mut i1 = ZERO;

    for k in 0..BESSEL_TERMS {
        let k_f = lift_usize::<FloatType>(k);
        // (x/2)^(2k) / (k!)^2 and (x/2)^(2k+1) / (k! (k+1)!), built by ratio to stay in range.
        let power_even = Real::powf(half_x, TWO * k_f);
        let fact_k = log_factorial(k);
        i0 += power_even / Real::exp(TWO * fact_k);
        i1 += power_even * half_x / Real::exp(fact_k + log_factorial(k + 1));
    }

    i1 / i0
}

/// `ln(n!)`, accumulated so the series terms stay representable at large `k`.
fn log_factorial(n: usize) -> FloatType {
    let mut acc = ZERO;
    for i in 2..=n {
        acc += Real::ln(lift_usize::<FloatType>(i));
    }
    acc
}

/// `I_1(x) / I_0(x)` from Miller's backward recurrence, `r_n = x / (2(n+1) + x r_{n+1})`.
///
/// Numerically stable and structurally unlike the series, so agreement between the two is
/// evidence that the reference curve itself is right.
fn bessel_ratio_miller(x: FloatType) -> FloatType {
    let mut r = ZERO;
    for n in (0..=MILLER_ORDER).rev() {
        r = x / (TWO * lift_usize::<FloatType>(n + 1) + x * r);
    }
    r
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== 2D U(1) Lattice Gauge Theory against its exact solution ===");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Exact solution: <P> = I_1(beta) / I_0(beta)   (Creutz 1983, ch. 8)\n");
}

fn print_setup() {
    println!(
        "Lattice:  {SIDE}x{SIDE}, periodic in both directions ({} links)",
        2 * SIDE * SIDE
    );
    println!(
        "Sampling: {THERMAL_SWEEPS} thermalization sweeps, {MEASURE_SWEEPS} measured, epsilon = {:.2}\n",
        lower(EPSILON)
    );
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_structural(identity_plaquette: FloatType) {
    println!("--- Structural check (the plaquette machinery, not the physics) ---");
    println!(
        "  identity field <P>  = {:.15}   (must be 1 at every beta)\n",
        lower(identity_plaquette)
    );
}

fn print_results(results: &[Measurement]) {
    println!("--- Measured against exact ---");
    println!(
        "  {:>6} {:>12} {:>12} {:>11} {:>6} {:>8}",
        "beta", "measured <P>", "I_1/I_0", "deviation", "band", "accept"
    );
    for r in results {
        println!(
            "  {:>6.1} {:>12.6} {:>12.6} {:>11.2e} {:>6.3} {:>7.0}%  {}",
            r.beta,
            lower(r.measured),
            lower(r.exact),
            lower(r.deviation),
            lower(r.tolerance),
            r.acceptance * 100.0,
            if r.agrees { "ok" } else { "DISAGREES" }
        );
    }

    println!("\n--- Reference cross-check (series vs Miller recurrence) ---");
    for r in results {
        println!(
            "  beta = {:>4.1}   |series - Miller| = {:.2e}",
            r.beta,
            lower(Real::abs(r.exact - r.exact_miller))
        );
    }
}

fn print_summary(all_agree: bool, reference_sound: bool) {
    println!("\n--- Summary ---");
    println!(
        "  sampled plaquette matches I_1/I_0 within each beta's band:  {}",
        if all_agree { "yes" } else { "NO" }
    );
    println!(
        "  Bessel reference self-consistent within {:e}: {}",
        lower(REFERENCE_TOLERANCE),
        if reference_sound { "yes" } else { "NO" }
    );
    if all_agree && reference_sound {
        println!("\n  The lattice reproduces the exact solution across strong and weak coupling.");
    }
}
