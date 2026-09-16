/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Collectable Example
//!
//! `Foldable` takes a structure apart into one value. `Collectable` is the other direction:
//! it builds the structure from a sequence.
//!
//! ```text
//! Foldable::fold      F<T>      ->  B
//! Collectable::collect  I: IntoIterator<Item = T>  ->  F<T>
//! ```
//!
//! The argument is anything iterable, so a caller can hand over a `Vec`, a slice's iterator,
//! a map, or a sequence that is generated lazily and never materialised. That is what makes
//! it the landing point for a computed stream: `Uncertain::materialize::<W>` uses exactly this
//! trait to put a run of draws into whichever container the caller names.

use deep_causality_haft::{Collectable, Foldable, VecWitness};
use deep_causality_num::{const_scalar_from_int, lift, lift_usize, lower};

/// The working scalar for the numeric section below.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);

fn main() {
    // ---------------------------------------------------------------------
    // 1. Building a structure from a sequence.
    // ---------------------------------------------------------------------
    let from_array: Vec<i32> = VecWitness::collect([1, 2, 3, 4]);
    // Nothing intermediate is built here: the squares are generated as they are placed.
    let squares: Vec<i32> = VecWitness::collect((0..4).map(|i| i * i));
    let empty: Vec<i32> = VecWitness::collect(Vec::new());
    print_collect(&from_array, &squares, &empty);
    assert_eq!(from_array, vec![1, 2, 3, 4]);
    assert_eq!(squares, vec![0, 1, 4, 9]);
    assert!(empty.is_empty());

    // ---------------------------------------------------------------------
    // 2. Order is preserved, which is what makes it the inverse of a fold.
    // ---------------------------------------------------------------------
    // The check uses a fold that is *not* commutative, so a reordering would show up.
    let folded = VecWitness::fold(VecWitness::collect([1, 2, 3]), 0, |acc, x| acc * 2 + x);
    let by_hand = [1, 2, 3].into_iter().fold(0, |acc, x| acc * 2 + x);
    print_round_trip(folded, by_hand);
    assert_eq!(folded, by_hand);

    // ---------------------------------------------------------------------
    // 3. Collect into the working precision, then fold back out.
    // ---------------------------------------------------------------------
    // A lazily generated run of samples lands in the container and is reduced in one pass.
    let samples: Vec<FloatType> =
        VecWitness::collect((1..=5).map(|i| lift_usize::<FloatType>(i) / TWO));
    let total = VecWitness::fold(samples.clone(), ZERO, |acc, x| acc + x);
    print_numeric(&samples, total);
    assert_eq!(total, lift::<FloatType>(7.5)); // 0.5 + 1.0 + 1.5 + 2.0 + 2.5
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_collect(from_array: &[i32], squares: &[i32], empty: &[i32]) {
    println!("=== DeepCausality HKT: Collectable ===\n");
    println!("--- 1. Building a structure from a sequence ---");
    println!("  collect([1, 2, 3, 4])            {from_array:?}");
    println!("  collect((0..4).map(|i| i * i))   {squares:?}");
    println!("  collect(nothing)                 {empty:?}   (the empty structure, not a failure)");
}

fn print_round_trip(folded: i32, by_hand: i32) {
    println!("\n--- 2. Order survives the round trip ---");
    println!("  fold(collect([1, 2, 3]))  {folded}");
    println!("  the same fold by hand     {by_hand}");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_numeric(samples: &[FloatType], total: FloatType) {
    let shown: Vec<f64> = samples.iter().map(|&x| lower(x)).collect();
    println!("\n--- 3. A generated run, collected then folded ---");
    println!("  samples  {shown:?}");
    println!("  sum      {}", lower(total));
}
