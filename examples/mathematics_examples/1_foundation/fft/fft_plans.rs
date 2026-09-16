/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `deep_causality_fft`: plan-based transforms
//!
//! The crate is plan-based. A `FftPlan` is built once for a length, holds every precomputed
//! twiddle and stage schedule, and is then immutable; execution borrows a caller-supplied
//! scratch buffer and allocates nothing. That is what lets a transform sit inside a hot loop.
//!
//! Three properties are worth reading off this example:
//!
//! 1. **Every length is O(N log N).** The planner picks by length: hardcoded kernels for
//!    small powers of two, an iterative mixed radix-4/radix-2 Stockham pipeline for larger
//!    ones, and Bluestein's chirp-z for everything else. A prime length is not a slow path.
//! 2. **The inverse is the forward kernel.** `ifft(x) = conj(fft(conj(x))) / N`, so the pair
//!    cannot drift apart: there is no second kernel to keep in step.
//! 3. **Precision is a parameter.** Every plan is generic over `FftScalar`, so the alias
//!    below chooses the precision of the whole transform.

use deep_causality_algebra::Real;
use deep_causality_fft::{FftPlan, RfftPlan};
use deep_causality_num::{const_scalar_from_int, lift, lift_usize, lower};
use deep_causality_num_complex::Complex;

/// Signal length for the round trip and the single-bin check: a power of two, so the
/// Stockham pipeline handles it.
const N: usize = 16;
/// A prime length, which routes through Bluestein's chirp-z instead.
const N_PRIME: usize = 13;
/// Cycles of the test sinusoid across the window, so its energy lands in exactly one bin.
const FREQ: usize = 3;

/// The working scalar. The plans, the signal and the spectrum all carry it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const SIXTY_FOUR: FloatType = const_scalar_from_int!(FloatType, 64);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. A round trip: ifft(fft(x)) returns x to rounding.
    // ---------------------------------------------------------------------
    let plan = FftPlan::<FloatType>::new(N)?;
    let original = ramp(N);
    let mut data = original.clone();
    let mut scratch = vec![Complex::new(ZERO, lift(0.0)); plan.scratch_len()];

    plan.execute(&mut data, &mut scratch)?;
    plan.execute_inverse(&mut data, &mut scratch)?;

    let drift = max_deviation(&original, &data);
    print_round_trip(plan.len(), plan.scratch_len(), drift);
    // The tolerance moves with the alias, so this reads the same at f32 or Float106.
    assert!(drift < SIXTY_FOUR * <FloatType as Real>::epsilon());

    // ---------------------------------------------------------------------
    // 2. A pure sinusoid puts all its energy in one bin.
    // ---------------------------------------------------------------------
    // cos(2 pi f n / N) has its energy split between bins f and N - f, at N/2 each.
    let mut spectrum = cosine(N, FREQ);
    plan.execute(&mut spectrum, &mut scratch)?;
    print_spectrum(&spectrum);
    // A real cosine splits its energy evenly between bins f and N - f, so each holds N/2.
    let peak = magnitude(&spectrum[FREQ]);
    let expected = lift_usize::<FloatType>(N) / TWO;
    let tol = SIXTY_FOUR * <FloatType as Real>::epsilon();
    assert!(Real::abs(peak - expected) < tol);

    // ---------------------------------------------------------------------
    // 3. A real signal needs only half the spectrum.
    // ---------------------------------------------------------------------
    // A real input has a Hermitian spectrum: bin N-k is the conjugate of bin k, so storing
    // N/2 + 1 bins loses nothing. `RfftPlan` works in that layout directly.
    let rplan = RfftPlan::<FloatType>::new(N)?;
    let real_signal: Vec<FloatType> = cosine(N, FREQ).iter().map(|c| c.re).collect();
    let mut half = vec![Complex::new(ZERO, lift(0.0)); rplan.spectrum_len()];
    let mut rscratch = vec![Complex::new(ZERO, lift(0.0)); rplan.scratch_len()];
    rplan.execute(&real_signal, &mut half, &mut rscratch)?;

    let mut recovered = vec![ZERO; N];
    rplan.execute_inverse(&half, &mut recovered, &mut rscratch)?;
    let rdrift = max_real_deviation(&real_signal, &recovered);
    print_rfft(N, rplan.spectrum_len(), magnitude(&half[FREQ]), rdrift);
    assert!(rdrift < SIXTY_FOUR * <FloatType as Real>::epsilon());

    // ---------------------------------------------------------------------
    // 4. A prime length is not a special case.
    // ---------------------------------------------------------------------
    let prime_plan = FftPlan::<FloatType>::new(N_PRIME)?;
    let prime_original = ramp(N_PRIME);
    let mut prime_data = prime_original.clone();
    let mut prime_scratch = vec![Complex::new(ZERO, lift(0.0)); prime_plan.scratch_len()];
    prime_plan.execute(&mut prime_data, &mut prime_scratch)?;
    prime_plan.execute_inverse(&mut prime_data, &mut prime_scratch)?;

    let prime_drift = max_deviation(&prime_original, &prime_data);
    print_prime(N_PRIME, prime_plan.scratch_len(), prime_drift);
    assert!(prime_drift < SIXTY_FOUR * <FloatType as Real>::epsilon());

    print_footer();
    Ok(())
}

/// `0, 1, 2, ...` as complex samples with zero imaginary part.
fn ramp(n: usize) -> Vec<Complex<FloatType>> {
    (0..n)
        .map(|i| Complex::new(lift_usize::<FloatType>(i), ZERO))
        .collect()
}

/// `cos(2 pi f i / n)` as complex samples.
fn cosine(n: usize, f: usize) -> Vec<Complex<FloatType>> {
    let two_pi_f_over_n =
        TWO * FloatType::pi() * lift_usize::<FloatType>(f) / lift_usize::<FloatType>(n);
    (0..n)
        .map(|i| {
            Complex::new(
                Real::cos(two_pi_f_over_n * lift_usize::<FloatType>(i)),
                ZERO,
            )
        })
        .collect()
}

fn magnitude(c: &Complex<FloatType>) -> FloatType {
    Real::sqrt(c.re * c.re + c.im * c.im)
}

/// The largest component-wise gap between two complex signals.
fn max_deviation(a: &[Complex<FloatType>], b: &[Complex<FloatType>]) -> FloatType {
    a.iter()
        .zip(b)
        .map(|(x, y)| Real::abs(x.re - y.re) + Real::abs(x.im - y.im))
        .fold(ZERO, |m, d| if d > m { d } else { m })
}

/// The same, for real signals.
fn max_real_deviation(a: &[FloatType], b: &[FloatType]) -> FloatType {
    a.iter()
        .zip(b)
        .map(|(x, y)| Real::abs(x - y))
        .fold(ZERO, |m, d| if d > m { d } else { m })
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== deep_causality_fft: plan-based transforms ===\n");
    println!("  Precision: {}", core::any::type_name::<FloatType>());
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_round_trip(len: usize, scratch: usize, drift: FloatType) {
    println!("\n--- 1. Round trip: ifft(fft(x)) == x ---");
    println!("  length          : {len}");
    println!("  scratch needed  : {scratch} complex slots, and no allocation during execute");
    println!("  max drift       : {:.3e}", lower(drift));
    println!("  The inverse reuses the forward kernel, so the pair cannot drift apart.");
}

fn print_spectrum(spectrum: &[Complex<FloatType>]) {
    println!("\n--- 2. cos(2 pi * {FREQ} n / {N}) puts its energy in two bins ---");
    println!("  bin   |X[k]|");
    for (k, c) in spectrum.iter().enumerate() {
        let m = lower(magnitude(c));
        if m > 1e-9 {
            println!("  {k:>3}   {m:>8.3}");
        }
    }
    println!(
        "  Bins {FREQ} and {} hold N/2 each: a real cosine has a",
        N - FREQ
    );
    println!("  Hermitian spectrum, so the two halves mirror one another.");
}

fn print_rfft(n: usize, spectrum_len: usize, peak: FloatType, drift: FloatType) {
    println!("\n--- 3. RfftPlan: half the spectrum, none of the information lost ---");
    println!("  real input length : {n}");
    println!("  stored bins       : {spectrum_len}  (N/2 + 1, the Hermitian half)");
    println!("  |X[{FREQ}]|            : {:.3}", lower(peak));
    println!("  round-trip drift  : {:.3e}", lower(drift));
}

fn print_prime(n: usize, scratch: usize, drift: FloatType) {
    println!("\n--- 4. A prime length is not a slow path ---");
    println!("  length          : {n}  (prime, so Bluestein's chirp-z handles it)");
    println!("  scratch needed  : {scratch} complex slots");
    println!("  max drift       : {:.3e}", lower(drift));
    println!("  Still O(N log N): the planner has no O(N^2) fallback to drop into.");
}

fn print_footer() {
    println!("\n--- The plan is the point ---");
    println!("  Twiddles and stage schedules are computed once, at plan construction.");
    println!("  Execution borrows scratch from the caller and allocates nothing, so a");
    println!("  transform can sit inside a hot loop without touching the allocator.");
}
