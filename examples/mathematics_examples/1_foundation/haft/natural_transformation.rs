/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Natural Transformation Example
//!
//! A `Functor` maps the *payload* and leaves the container alone. A `NaturalTransformation`
//! does the opposite: it maps the *container* and never looks at the payload.
//!
//! `OptionToVec` is the canonical one: `None` becomes `[]`, `Some(a)` becomes `[a]`. It works
//! for any `A` precisely because it does not inspect `A`.
//!
//! The law that makes it "natural" is that the two orders agree:
//!
//! ```text
//! transform(fmap(fa, f)) == fmap(transform(fa), f)
//! ```
//!
//! Map then reshape, or reshape then map -- same answer. That is what lets a conversion be
//! pushed to either end of a pipeline without changing the result.

use deep_causality_haft::{Functor, NaturalTransformation, OptionToVec, OptionWitness, VecWitness};
use std::fmt::Debug;

fn main() {
    // 1. The component at one type: Option<i32> reshaped into Vec<i32>.
    let present: Option<i32> = Some(7);
    let absent: Option<i32> = None;

    let from_present = OptionToVec::transform(present);
    let from_absent = OptionToVec::transform(absent);
    print_component(&from_present, &from_absent);
    assert_eq!(from_present, vec![7]);
    assert_eq!(from_absent, Vec::<i32>::new());

    // 2. The same component at a different payload type, unchanged.
    let name: Option<String> = Some("sensor-a".to_string());
    let as_vec = OptionToVec::transform(name);
    print_other_payload(&as_vec);
    assert_eq!(as_vec, vec!["sensor-a".to_string()]);

    // 3. Naturality: map then reshape equals reshape then map.
    let double = |x: i32| x * 2;

    let map_then_reshape = OptionToVec::transform(OptionWitness::fmap(Some(21), double));
    let reshape_then_map = VecWitness::fmap(OptionToVec::transform(Some(21)), double);
    print_naturality(&map_then_reshape, &reshape_then_map);
    assert_eq!(map_then_reshape, reshape_then_map);

    // The empty case has to agree too, and it does for the same reason: there is no
    // payload to disagree about.
    let empty_first = OptionToVec::transform(OptionWitness::fmap(None, double));
    let empty_second = VecWitness::fmap(OptionToVec::transform(None), double);
    print_naturality_empty(&empty_first, &empty_second);
    assert_eq!(empty_first, empty_second);
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_component<T: Debug>(present: &T, absent: &T) {
    println!("=== DeepCausality HKT: Natural Transformation (Option => Vec) ===\n");
    println!("--- 1. The component reshapes the container ---");
    println!("  Some(7) -> {present:?}");
    println!("  None    -> {absent:?}");
}

fn print_other_payload<T: Debug>(as_vec: &T) {
    println!("\n--- 2. The same component at another payload type ---");
    println!("  Some(\"sensor-a\") -> {as_vec:?}");
}

fn print_naturality<T: Debug>(map_first: &T, reshape_first: &T) {
    println!("\n--- 3. Naturality: the two orders agree ---");
    println!("  transform(fmap(Some(21), double)) = {map_first:?}");
    println!("  fmap(transform(Some(21)), double) = {reshape_first:?}");
}

fn print_naturality_empty<T: Debug>(map_first: &T, reshape_first: &T) {
    println!("  transform(fmap(None, double))     = {map_first:?}");
    println!("  fmap(transform(None), double)     = {reshape_first:?}");
}
