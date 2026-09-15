/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Four scalars, one bound
//!
//! `Float` is implemented by exactly four types, and they span nine orders of magnitude in
//! resolution:
//!
//! ```text
//! BFloat16    8 significand bits    the top half of an f32, for throughput
//! f32        24                     hardware single precision
//! f64        53                     hardware double precision
//! Float106  106                     two f64 limbs held as an unevaluated sum
//! ```
//!
//! An algorithm written against `Float` runs at all four with no line added. That is what
//! "precision is a parameter" means, and this example is the demonstration: every function below
//! is written once, generic in `T: Float`, and called four times.
//!
//! Two results are worth the trip:
//!
//! **Compensated summation buys precision back.** Adding many small terms to a large running sum
//! loses the small ones to rounding. Kahan's algorithm keeps a correction term and recovers most
//! of what the naive loop drops — at `BFloat16` it turns a completely wrong answer into a usable
//! one, which is the same trick that makes low-precision training work.
//!
//! **Rewriting the expression recovers what rounding took.** Evaluating `(1 − cos x)/x²` as
//! written subtracts two nearly equal numbers, which throws away the digits the answer is made of.
//! The identity `1 − cos x = 2 sin²(x/2)` computes the same quantity with no subtraction at all.
//! Section 3 runs both forms at all four scalars: at `BFloat16` the difference is a wrong answer
//! against a right one, and at `f32` it is five orders of magnitude.

use deep_causality_num::{BFloat16, Float, Float106, FromPrimitive, ToPrimitive, lift, lower};

/// The summation: a large running total, then many small terms added to it.
const RUNNING_TOTAL: f64 = 1.0;
const SMALL_TERM: f64 = 1.0e-3;
const TERM_COUNT: usize = 10_000;

/// The angle section 3 evaluates the versine quotient at, small enough to cancel.
const SMALL_ANGLE: f64 = 1.0e-3;

/// The working scalars. Each name below is the parameter, and the code is written once.
type Bf16 = BFloat16;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. What each scalar resolves.
    // ---------------------------------------------------------------------
    // `epsilon` is the gap between 1 and the next representable value above it, so it says
    // directly how many digits a scalar carries.
    print_zoo_header();
    print_zoo::<Bf16>("BFloat16");
    print_zoo::<f32>("f32");
    print_zoo::<f64>("f64");
    print_zoo::<Float106>("Float106");

    // ---------------------------------------------------------------------
    // 2. Naive against compensated summation.
    // ---------------------------------------------------------------------
    // The exact total is 1 + 10000 * 0.001 = 11. A term is lost whenever it falls below half an
    // ulp of the running sum, which is exactly what `epsilon` measures.
    let exact = RUNNING_TOTAL + SMALL_TERM * TERM_COUNT as f64;
    print_summation_header(exact);
    print_summation::<Bf16>("BFloat16", exact);
    print_summation::<f32>("f32", exact);
    print_summation::<f64>("f64", exact);
    print_summation::<Float106>("Float106", exact);

    // A compensated sum is at least as good as a naive one at every precision.
    for (naive, kahan) in [
        summation_errors::<Bf16>(exact),
        summation_errors::<f32>(exact),
        summation_errors::<f64>(exact),
        summation_errors::<Float106>(exact),
    ] {
        assert!(kahan <= naive);
    }

    // ---------------------------------------------------------------------
    // 3. Cancellation, and the rearrangement that removes it.
    // ---------------------------------------------------------------------
    // `(1 - cos x) / x²` tends to 1/2. At small `x`, `cos x` is close to 1, so the subtraction
    // cancels the leading digits and leaves only what rounding left behind. `1 - cos x =
    // 2 sin²(x/2)` is the same quantity with nothing to cancel.
    let reference = reference_value();
    print_cancellation_header(reference);
    print_cancellation::<Bf16>("BFloat16", reference);
    print_cancellation::<f32>("f32", reference);
    print_cancellation::<f64>("f64", reference);
    print_cancellation::<Float106>("Float106", reference);

    // The stable form is at least as accurate at every precision.
    for (naive, stable) in [
        cancellation_errors::<Bf16>(reference),
        cancellation_errors::<f32>(reference),
        cancellation_errors::<f64>(reference),
        cancellation_errors::<Float106>(reference),
    ] {
        assert!(stable <= naive);
    }

    // At f32 the rearrangement is the difference between one good digit and full precision.
    let (f32_naive, f32_stable) = cancellation_errors::<f32>(reference);
    assert!(f32_stable * 1.0e5 < f32_naive);

    print_footer();
    Ok(())
}

/// Naive summation: add each term straight into the running total.
///
/// A term smaller than half an ulp of the total disappears into the rounding.
fn naive_sum<T: Float + FromPrimitive>(start: T, term: T, count: usize) -> T {
    (0..count).fold(start, |acc, _| acc + term)
}

/// Compensated summation, after Kahan.
///
/// `compensation` carries the part of the previous addition that rounding threw away, and the next
/// term is corrected by it before being added. The correction is itself computed by arithmetic the
/// scalar can represent, which is why this works at any precision.
fn kahan_sum<T: Float + FromPrimitive>(start: T, term: T, count: usize) -> T {
    let mut total = start;
    let mut compensation = T::zero();

    for _ in 0..count {
        let corrected = term - compensation;
        let next = total + corrected;
        // `next - total` is what the addition actually accepted; the rest was dropped.
        compensation = (next - total) - corrected;
        total = next;
    }

    total
}

/// The relative error of each summation against the exact total.
fn summation_errors<T: Float + FromPrimitive + ToPrimitive>(exact: f64) -> (f64, f64) {
    let start = lift::<T>(RUNNING_TOTAL);
    let term = lift::<T>(SMALL_TERM);
    let naive = lower(naive_sum(start, term, TERM_COUNT));
    let kahan = lower(kahan_sum(start, term, TERM_COUNT));

    (
        ((naive - exact) / exact).abs(),
        ((kahan - exact) / exact).abs(),
    )
}

/// `(1 - cos x) / x²`, written the way the formula reads.
///
/// `cos x` is within `x²/2` of 1 for small `x`, so the numerator is a subtraction of two values
/// that agree to as many digits as the scalar holds. What survives is rounding.
fn versine_naive<T: Float + FromPrimitive>(x: T) -> T {
    (T::one() - x.cos()) / (x * x)
}

/// The same quantity through `1 - cos x = 2 sin²(x/2)`.
///
/// `sin(x/2)` is computed directly and squared, so no two nearly equal numbers ever meet. The
/// identity is exact; only the arithmetic around it changes.
fn versine_stable<T: Float + FromPrimitive>(x: T) -> T {
    let half = x / lift::<T>(2.0);
    let s = half.sin();

    lift::<T>(2.0) * s * s / (x * x)
}

/// The value the other scalars are measured against: the stable form at `Float106`.
///
/// `0.5` is the limit as `x` tends to zero, and at `x = 0.001` the function sits `x²/24` below it.
/// Measuring against the limit would report that gap as error, so the reference is the value at
/// this `x`, computed in the widest scalar by the form that has nothing to cancel. That is what
/// `Float106` is for in this workspace: the oracle a narrower path is diffed against.
fn reference_value() -> f64 {
    lower(versine_stable(lift::<Float106>(SMALL_ANGLE)))
}

/// The relative error of each form against the reference.
fn cancellation_errors<T: Float + FromPrimitive + ToPrimitive>(reference: f64) -> (f64, f64) {
    let x = lift::<T>(SMALL_ANGLE);

    (
        ((lower(versine_naive(x)) - reference) / reference).abs(),
        ((lower(versine_stable(x)) - reference) / reference).abs(),
    )
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== Four scalars, one bound ===\n");
    println!("  Every function below is written once against `T: Float` and called four times.\n");
}

fn print_zoo_header() {
    println!("--- 1. What each scalar resolves ---");
    println!("  scalar       epsilon      decimal digits          largest finite");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_zoo<T: Float + ToPrimitive>(name: &str) {
    let epsilon = lower(T::epsilon());
    let digits = -epsilon.log10();
    println!(
        "  {name:<10} {epsilon:11.3e}   {digits:5.1}                 {:11.3e}",
        lower(T::max_value())
    );
}

fn print_summation_header(exact: f64) {
    println!("\n--- 2. {TERM_COUNT} terms of {SMALL_TERM} added to {RUNNING_TOTAL} ---");
    println!("  the exact total is {exact}\n");
    println!("  scalar            naive          error      compensated          error");
}

fn print_summation<T: Float + FromPrimitive + ToPrimitive>(name: &str, exact: f64) {
    let start = lift::<T>(RUNNING_TOTAL);
    let term = lift::<T>(SMALL_TERM);
    let naive = lower(naive_sum(start, term, TERM_COUNT));
    let kahan = lower(kahan_sum(start, term, TERM_COUNT));
    let (naive_error, kahan_error) = summation_errors::<T>(exact);

    println!("  {name:<10} {naive:14.6}   {naive_error:8.2e}   {kahan:14.6}   {kahan_error:8.2e}");
}

fn print_cancellation_header(reference: f64) {
    println!("\n--- 3. (1 - cos x) / x^2 at x = {SMALL_ANGLE} ---");
    println!("  the naive form subtracts two nearly equal numbers; the stable form does not.");
    println!("  reference, from the stable form at Float106: {reference:.15}\n");
    println!("  scalar             naive          error           stable          error");
}

fn print_cancellation<T: Float + FromPrimitive + ToPrimitive>(name: &str, reference: f64) {
    let x = lift::<T>(SMALL_ANGLE);
    let (naive_error, stable_error) = cancellation_errors::<T>(reference);

    println!(
        "  {name:<10} {:14.9}   {naive_error:8.2e}   {:14.9}   {stable_error:8.2e}",
        lower(versine_naive(x)),
        lower(versine_stable(x))
    );
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  `Float` is the bound and the four types are the parameter, so an algorithm is");
    println!("  written once and the caller decides what it costs. Section 2 is why a");
    println!("  compensated accumulator is worth the extra add, and section 3 is why the shape");
    println!("  of an expression is worth attention before the width of its type. Float106 is");
    println!("  the oracle in section 3, which is the role a 106-bit scalar earns: the value");
    println!("  the narrower paths are diffed against.");
}
