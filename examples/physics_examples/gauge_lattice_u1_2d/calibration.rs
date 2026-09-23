/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `--calibrate`: the seed statistics behind the agreement bands and the choice of start.
//!
//! For every seed in `CALIBRATION_SEEDS` and for each start, one generator is seeded and drives
//! the couplings in `BETA_VALUES` order, exactly as the check does with `SEED`. Per coupling it
//! prints the mean, standard deviation and largest magnitude of `<P> - I_1/I_0`, the band the
//! rule in `AGREEMENT_TOLERANCES` derives from them, the median number of topological-charge
//! changes per measured run, and, for runs whose charge never changed, the mean deviation by
//! `|Q|`. It closes with the exact finite-volume plaquette on this lattice.

use crate::{
    AGREEMENT_TOLERANCES, BETA_VALUES, FloatType, Measurement, ONE, SIDE, Start, TWO, ZERO,
    bessel_ratio_series, log_factorial, measure,
};
use deep_causality_algebra::Real;
use deep_causality_num::{lift, lift_usize, lower};
use deep_causality_rand::Xoshiro256;
use deep_causality_topology::{CubicalComplex, TopologyError};
use std::collections::BTreeMap;
use std::sync::Arc;

/// The seeds the published statistics come from.
const CALIBRATION_SEEDS: std::ops::RangeInclusive<u64> = 101..=160;
/// Bands are rounded up to a multiple of this.
const BAND_STEP: f64 = 0.005;
/// Margin added to the largest observed deviation before rounding.
const BAND_MARGIN: f64 = 0.005;
/// Largest `|n|` in the finite-volume character sum; the terms fall as `(I_n/I_0)^V`.
const CHARACTER_ORDERS: i64 = 20;

pub(crate) fn run(lattice: &Arc<CubicalComplex<2, FloatType>>) -> Result<(), TopologyError> {
    println!("=== gauge_lattice_u1_2d calibration ===");
    println!(
        "Seeds {}..={}, one generator per seed across the couplings.\n",
        CALIBRATION_SEEDS.start(),
        CALIBRATION_SEEDS.end()
    );

    for (start, label) in [(Start::Identity, "identity"), (Start::Random, "random")] {
        let runs = collect(lattice, start)?;
        print_statistics(label, start, &runs);
    }

    print_finite_volume();
    Ok(())
}

/// `runs[b]` holds one measurement per seed at `BETA_VALUES[b]`.
fn collect(
    lattice: &Arc<CubicalComplex<2, FloatType>>,
    start: Start,
) -> Result<Vec<Vec<Measurement>>, TopologyError> {
    let mut runs: Vec<Vec<Measurement>> = BETA_VALUES.iter().map(|_| Vec::new()).collect();
    for seed in CALIBRATION_SEEDS {
        let mut generator = Xoshiro256::from_seed(seed);
        for (b, (&beta, &tolerance)) in BETA_VALUES
            .iter()
            .zip(AGREEMENT_TOLERANCES.iter())
            .enumerate()
        {
            runs[b].push(measure(
                lattice,
                beta,
                lift(tolerance),
                start,
                &mut generator,
            )?);
        }
    }
    Ok(runs)
}

fn print_statistics(label: &str, start: Start, runs: &[Vec<Measurement>]) {
    println!("--- {label} start ---");
    println!(
        "  {:>5} {:>10} {:>9} {:>9} {:>7} {:>7} {:>6} {:>9}",
        "beta", "mean", "sd", "max|dev|", "derived", "config", "gap", "Q changes"
    );
    for (b, per_seed) in runs.iter().enumerate() {
        let devs: Vec<FloatType> = per_seed.iter().map(|m| m.signed_deviation).collect();
        let (mean, sd, max_abs) = summary(&devs);
        let derived = derive_band(mean, sd, max_abs);
        let mut changes: Vec<usize> = per_seed.iter().map(|m| m.charge_changes).collect();
        changes.sort_unstable();
        let median = (changes[(changes.len() - 1) / 2] + changes[changes.len() / 2]) as f64 / 2.0;
        let gap = ONE - bessel_ratio_series(lift(BETA_VALUES[b]));
        let configured = if start == Start::Identity {
            format!("{:.3}", AGREEMENT_TOLERANCES[b])
        } else {
            "-".to_string()
        };
        println!(
            "  {:>5.1} {:>+10.2e} {:>9.2e} {:>9.2e} {:>7.3} {:>7} {:>6.3} {:>9.1}",
            BETA_VALUES[b],
            lower(mean),
            lower(sd),
            lower(max_abs),
            lower(derived),
            configured,
            lower(gap),
            median
        );
    }

    println!("  runs whose charge never changed, mean deviation by |Q| (count):");
    for (b, per_seed) in runs.iter().enumerate() {
        let mut by_sector: BTreeMap<i64, Vec<FloatType>> = BTreeMap::new();
        for m in per_seed.iter().filter(|m| m.charge_changes == 0) {
            by_sector
                .entry(m.charge.abs())
                .or_default()
                .push(m.signed_deviation);
        }
        let cells: Vec<String> = by_sector
            .iter()
            .map(|(q, devs)| {
                let (mean, _, _) = summary(devs);
                format!("|Q|={q}: {:+.1e} ({})", lower(mean), devs.len())
            })
            .collect();
        let frozen = if cells.is_empty() {
            "-".to_string()
        } else {
            cells.join("  ")
        };
        println!("  {:>5.1}  {}", BETA_VALUES[b], frozen);
    }
    println!();
}

/// Mean, sample standard deviation and largest magnitude.
fn summary(values: &[FloatType]) -> (FloatType, FloatType, FloatType) {
    let n = lift_usize::<FloatType>(values.len());
    let mean = values.iter().fold(ZERO, |acc, &v| acc + v) / n;
    let sd = if values.len() > 1 {
        let ss = values
            .iter()
            .fold(ZERO, |acc, &v| acc + (v - mean) * (v - mean));
        Real::sqrt(ss / (n - ONE))
    } else {
        ZERO
    };
    let max_abs = values.iter().fold(ZERO, |acc, &v| {
        if Real::abs(v) > acc {
            Real::abs(v)
        } else {
            acc
        }
    });
    (mean, sd, max_abs)
}

/// `max(|m| + 3s, max|dev| + BAND_MARGIN)`, rounded up to a multiple of `BAND_STEP`.
fn derive_band(mean: FloatType, sd: FloatType, max_abs: FloatType) -> FloatType {
    let three: FloatType = lift(3.0);
    let step: FloatType = lift(BAND_STEP);
    let statistical = Real::abs(mean) + three * sd;
    let observed = max_abs + lift::<FloatType>(BAND_MARGIN);
    let wider = if statistical > observed {
        statistical
    } else {
        observed
    };
    Real::ceil(wider / step) * step
}

/// `<P>` on the finite periodic lattice from the character expansion,
/// `sum_n r_n^(V-1) (r_(n-1) + r_(n+1)) / 2 / sum_n r_n^V` with `r_n = I_n / I_0` and `V`
/// plaquettes, against the infinite-volume `I_1/I_0`.
fn print_finite_volume() {
    let plaquettes = SIDE * SIDE;
    println!("--- exact finite-volume plaquette ({SIDE}x{SIDE}, {plaquettes} plaquettes) ---");
    println!(
        "  {:>5} {:>12} {:>12} {:>11}",
        "beta", "finite V", "I_1/I_0", "difference"
    );
    for &beta in &BETA_VALUES {
        let x: FloatType = lift(beta);
        let i0 = bessel_i(0, x);
        let r = |n: i64| bessel_i(n.unsigned_abs() as usize, x) / i0;
        let v = lift_usize::<FloatType>(plaquettes);
        let mut numerator = ZERO;
        let mut denominator = ZERO;
        for n in -CHARACTER_ORDERS..=CHARACTER_ORDERS {
            let rn = r(n);
            numerator += Real::powf(rn, v - ONE) * (r(n - 1) + r(n + 1)) / TWO;
            denominator += Real::powf(rn, v);
        }
        let finite = numerator / denominator;
        let infinite = bessel_ratio_series(x);
        println!(
            "  {:>5.1} {:>12.9} {:>12.9} {:>+11.2e}",
            beta,
            lower(finite),
            lower(infinite),
            lower(finite - infinite)
        );
    }
}

/// `I_n(x) = sum_k (x/2)^(2k+n) / (k! (k+n)!)`, each term formed in logarithms.
fn bessel_i(n: usize, x: FloatType) -> FloatType {
    let log_half_x = Real::ln(x / TWO);
    (0..crate::BESSEL_TERMS).fold(ZERO, |acc, k| {
        let power = lift_usize::<FloatType>(2 * k + n) * log_half_x;
        acc + Real::exp(power - log_factorial(k) - log_factorial(k + n))
    })
}
