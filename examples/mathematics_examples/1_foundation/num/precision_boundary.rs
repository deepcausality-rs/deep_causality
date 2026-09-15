/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The precision boundary
//!
//! A program written against a scalar parameter still has to meet the concrete world at two
//! places: the literals it is configured with, and the display it prints to. Those two places are
//! the precision boundary, and `deep_causality_num` gives them names.
//!
//! ```text
//! lift(x: f64) -> T          a configuration literal, written once at the widest form a
//!                            source file can hold, lifted where it is used
//! lift_count(n: u64) -> T    a count onto the real axis: a step index, a dimension, a sample size
//! lower(x: T) -> f64         the display boundary, and the only place f64 belongs
//! to_count(x: T) -> u64      a real rounded back to a count, guarded
//! ```
//!
//! Every one has a `try_` twin returning `Option`. For a float target the crossing saturates
//! rather than refusing: a literal past the range arrives as infinity, and a count past the
//! significand arrives rounded. Section 3 measures both, because a crossing that stays quiet is
//! one the width has to be chosen for deliberately. `to_count` is the crossing that does refuse,
//! and it refuses on a negative, on a non-finite, and on anything past `u64`.
//!
//! There is also a family named by the source type — `lift_u32`, `lift_usize`, `lift_i64` and the
//! rest — so a count crosses from the type it already has rather than through `f64` on the way.
//!
//! ## What the crossing costs
//!
//! `lift` rounds to the nearest value the target holds, so `lower(lift::<T>(x))` returns `x` only
//! when `T` can represent it. Section 2 measures that for a decimal literal at all four scalars,
//! which is the answer to "what does my config file actually mean at this precision".

use deep_causality_num::{
    BFloat16, Float, Float106, Lift, Lower, lift, lift_count, lift_u32, lower, to_count, try_lift,
    try_lift_count,
};

/// A decimal literal with no exact binary form, which is what most configuration is.
const RATE: f64 = 0.1;
/// A value with digits well past what a narrow scalar holds.
const MEASURED: f64 = 1234.56789;
/// A count that crosses onto the real axis, the way a sample size does.
const SAMPLES: u64 = 4096;
/// Past the range every scalar here holds.
const ENORMOUS: f64 = 1.0e308;
/// A count past what a 24-bit significand represents exactly.
const LARGE_COUNT: u64 = 16_777_217;

/// The working scalars, named so the calls below read as one crossing each.
type Bf16 = BFloat16;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. The four crossings, in one round trip.
    // ---------------------------------------------------------------------
    // A literal goes in, a count goes in, arithmetic happens in the working type, and only the
    // printing comes back out.
    let rate = lift::<f64>(RATE);
    let samples = lift_count::<f64>(SAMPLES);
    let expected = rate * samples;
    print_round_trip(
        lower(rate),
        lower(samples),
        lower(expected),
        to_count(expected),
    );

    // `to_count` is the inverse crossing: a real that names a number of things becomes a count.
    assert_eq!(to_count(expected), Some(410));

    // ---------------------------------------------------------------------
    // 2. What the crossing costs, per scalar.
    // ---------------------------------------------------------------------
    // `lift` rounds to the nearest representable value. For a decimal literal that is a gap, and
    // how big a gap is the whole difference between the four scalars.
    print_cost_header();
    print_cost::<Bf16>("BFloat16");
    print_cost::<f32>("f32");
    print_cost::<f64>("f64");
    print_cost::<Float106>("Float106");

    // The gap never exceeds half an ulp, which is what "rounds to nearest" promises.
    for gap in [
        relative_gap::<Bf16>(RATE),
        relative_gap::<f32>(RATE),
        relative_gap::<f64>(RATE),
        relative_gap::<Float106>(RATE),
    ] {
        assert!(gap < 1.0e-2);
    }
    // And it shrinks monotonically as the scalar widens.
    assert!(relative_gap::<f64>(RATE) < relative_gap::<f32>(RATE));
    assert!(relative_gap::<f32>(RATE) < relative_gap::<Bf16>(RATE));

    // ---------------------------------------------------------------------
    // 3. Where the crossing refuses, and where it rounds.
    // ---------------------------------------------------------------------
    // Going in, a float target saturates: past the range is infinity, past the significand is the
    // nearest value it holds. Neither says so. Coming back, `to_count` refuses outright.
    print_fallible_header();
    print_fallible::<Bf16>("BFloat16");
    print_fallible::<f32>("f32");
    print_fallible::<f64>("f64");
    print_fallible::<Float106>("Float106");

    // Going in never refuses for a float: a literal past the range arrives as an infinity, which
    // is a value, so the `Option` is `Some`. The `Option` is there for the integer targets
    // `FromPrimitive` also covers.
    let past_range =
        try_lift::<Bf16>(ENORMOUS).expect("a float target saturates rather than refusing");
    assert!(past_range.is_infinite());

    // A count past the significand arrives rounded, and silently.
    let rounded_count = try_lift_count::<f32>(LARGE_COUNT).expect("a float target rounds");
    assert_eq!(lower(rounded_count), 16_777_216.0);

    // `to_count` guards the way back: a negative, a non-finite, or a value past `u64` is `None`.
    let negative = lift::<f64>(-1.0);
    let infinite = f64::infinity();
    print_to_count_guards(
        to_count(negative),
        to_count(infinite),
        to_count(lift::<f64>(2.5)),
    );

    assert_eq!(to_count(negative), None);
    assert_eq!(to_count(infinite), None);
    // 2.5 rounds to 3: `to_count` rounds, it does not truncate.
    assert_eq!(to_count(lift::<f64>(2.5)), Some(3));

    // ---------------------------------------------------------------------
    // 4. Crossing from the type a value already has.
    // ---------------------------------------------------------------------
    // `lift_u32` and its siblings take the source type directly, so a count never detours through
    // `f64`. The `Lift` and `Lower` traits offer the same crossings as methods.
    let width: u32 = 640;
    let from_u32 = lift_u32::<f32>(width);
    let by_method: f32 = width.lift();
    let back: f64 = by_method.lower();
    print_source_typed(lower(from_u32), lower(by_method), back);

    assert_eq!(from_u32, by_method);

    print_footer();
    Ok(())
}

/// The relative gap a literal picks up crossing into `T` and back.
fn relative_gap<T: Float + deep_causality_num::FromPrimitive + deep_causality_num::ToPrimitive>(
    literal: f64,
) -> f64 {
    ((lower(lift::<T>(literal)) - literal) / literal).abs()
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== The precision boundary ===\n");
    println!("  Two places a program written against a scalar parameter meets the concrete");
    println!("  world: the literals it is configured with, and the display it prints to.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_round_trip(rate: f64, samples: f64, expected: f64, count: Option<u64>) {
    println!("--- 1. One round trip through all four crossings ---");
    println!("  lift({RATE})            -> {rate}");
    println!("  lift_count({SAMPLES})       -> {samples}");
    println!("  rate * samples          -> {expected}");
    println!("  to_count(...)           -> {count:?}      the inverse crossing, rounded");
}

fn print_cost_header() {
    println!("\n--- 2. What a decimal literal costs, per scalar ---");
    println!("  {RATE} and {MEASURED} have no exact binary form.\n");
    println!("  scalar       lift({RATE}) lowered          relative gap      lift({MEASURED})");
}

fn print_cost<T: Float + deep_causality_num::FromPrimitive + deep_causality_num::ToPrimitive>(
    name: &str,
) {
    println!(
        "  {name:<10} {:22.18}   {:9.2e}   {:14.6}",
        lower(lift::<T>(RATE)),
        relative_gap::<T>(RATE),
        lower(lift::<T>(MEASURED))
    );
}

fn print_fallible_header() {
    println!("\n--- 3. Where the crossing refuses, and where it rounds ---");
    println!("  try_lift({ENORMOUS:.0e}) and try_lift_count({LARGE_COUNT})\n");
    println!("  scalar       past the range         a count past the significand");
}

fn print_fallible<
    T: Float + deep_causality_num::FromPrimitive + deep_causality_num::ToPrimitive,
>(
    name: &str,
) {
    let huge = try_lift::<T>(ENORMOUS).map(lower);
    let counted = try_lift_count::<T>(LARGE_COUNT).map(lower);
    println!("  {name:<10} {:<22} {:?}", format!("{huge:?}"), counted);
}

fn print_to_count_guards(negative: Option<u64>, infinite: Option<u64>, rounded: Option<u64>) {
    println!();
    println!("  Going in, every answer is `Some`: a float target saturates to an infinity and");
    println!("  rounds a large count, and both are values. Choosing the width is the guard.");
    println!("\n  Coming back, to_count refuses:");
    println!("    a negative      {negative:?}");
    println!("    a non-finite    {infinite:?}");
    println!("    2.5             {rounded:?}        it rounds rather than truncating");
}

fn print_source_typed(from_u32: f64, by_method: f64, back: f64) {
    println!("\n--- 4. Crossing from the type a value already has ---");
    println!("  lift_u32::<f32>(640)   -> {from_u32}");
    println!("  640u32.lift::<f32>()   -> {by_method}      the same crossing, as a method");
    println!("  .lower()               -> {back}      and back at the display boundary");
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  A literal is written once, at the widest form a source file holds, and lifted");
    println!("  where it is used. A value is lowered where it is printed and nowhere else.");
    println!("  Between those two lines the working type is the only type, which is what lets");
    println!("  the scalar be a parameter at all.");
}
