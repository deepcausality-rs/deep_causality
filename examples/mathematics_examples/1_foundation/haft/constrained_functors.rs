/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constrained Functor Example
//!
//! `Functor` maps a payload. These three map a *capability* instead: given that the payload
//! is `Clone`, `Debug` or `PartialEq`, the witness supplies that same capability for its
//! container.
//!
//!   `CloneFunctor::clone_type`  clone the container when `T: Clone`
//!   `DebugFunctor::fmt_type`    format the container when `T: Debug`
//!   `EqFunctor::eq_type`        compare two containers when `T: PartialEq`
//!
//! `OptionWitness`, `VecWitness`, `BoxWitness`, `LinkedListWitness` and `VecDequeWitness`
//! implement all three, each body delegating to the container's own instance.
//!
//! They exist because a `#[derive]` cannot reach through the GAT projection. A recursive
//! carrier such as `Free<F, A>` holds a `Self::Type<Box<Free<F, A>>>`, and asking the derive
//! to prove that field `Debug` sends the trait solver into an overflow. Routing the capability
//! through the witness settles it in one step, so the recursive carriers get their `Clone`,
//! `Debug` and `PartialEq` instances from the witness rather than from a derive.

use deep_causality_haft::{
    CloneFunctor, DebugFunctor, EqFunctor, HKT, LinkedListWitness, OptionWitness, VecDequeWitness,
    VecWitness,
};
use deep_causality_num::{const_scalar_from_float, lift};
use std::collections::{LinkedList, VecDeque};
use std::fmt;

/// The working scalar. One of the payloads below carries it, to show the capability
/// traits are indifferent to which type the container holds.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const THREE_HALVES: FloatType = const_scalar_from_float!(FloatType, 1.5);

fn main() {
    // ---------------------------------------------------------------------
    // 1. One generic function per capability, serving every witness.
    // ---------------------------------------------------------------------
    let xs = vec![1, 2, 3];
    let copy = duplicate::<VecWitness, i32>(&xs);
    print_clone(&xs, &copy);
    assert_eq!(copy, xs);

    let same = equal::<VecWitness, i32>(&xs, &copy);
    let different = equal::<VecWitness, i32>(&xs, &vec![1, 2, 4]);
    print_eq(same, different);
    assert!(same && !different);

    // ---------------------------------------------------------------------
    // 2. The same two functions, other witnesses, no new code.
    // ---------------------------------------------------------------------
    let opt = Some("sensor".to_string());
    let list = LinkedList::from([10, 20]);
    let deque = VecDeque::from([THREE_HALVES, lift(2.5)]);

    print_other_witnesses(
        &duplicate::<OptionWitness, String>(&opt),
        &duplicate::<LinkedListWitness, i32>(&list),
        &duplicate::<VecDequeWitness, FloatType>(&deque),
    );
    assert!(equal::<OptionWitness, String>(&opt, &opt.clone()));
    assert!(equal::<LinkedListWitness, i32>(&list, &list.clone()));

    // ---------------------------------------------------------------------
    // 3. `DebugFunctor` needs a formatter, so it is reached through a wrapper.
    // ---------------------------------------------------------------------
    // `Shown` is `Debug` for any witness that is a `DebugFunctor`, without knowing which
    // container it holds.
    print_debug(
        &Shown::<OptionWitness, String>(&opt),
        &Shown::<VecWitness, i32>(&xs),
        &Shown::<VecDequeWitness, FloatType>(&deque),
    );
}

/// Clones any container whose witness is a `CloneFunctor`.
fn duplicate<W: CloneFunctor, T: Clone>(fa: &W::Type<T>) -> W::Type<T> {
    W::clone_type(fa)
}

/// Compares any two containers whose witness is an `EqFunctor`.
fn equal<W: EqFunctor, T: PartialEq>(a: &W::Type<T>, b: &W::Type<T>) -> bool {
    W::eq_type(a, b)
}

/// A `Debug` view of `W::Type<T>`, borrowed, for any `W: DebugFunctor`.
struct Shown<'a, W: HKT, T>(&'a W::Type<T>);

impl<W: DebugFunctor, T: fmt::Debug> fmt::Debug for Shown<'_, W, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        W::fmt_type(self.0, f)
    }
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_clone(original: &[i32], copy: &[i32]) {
    println!("=== DeepCausality HKT: Constrained Functors ===\n");
    println!("--- 1. CloneFunctor: the witness clones the container ---");
    println!("  original          {original:?}");
    println!("  clone_type        {copy:?}");
}

fn print_eq(same: bool, different: bool) {
    println!("\n--- 2. EqFunctor: the witness compares two containers ---");
    println!("  eq_type(xs, copy)      {same}");
    println!("  eq_type(xs, [1, 2, 4]) {different}");
}

fn print_other_witnesses(
    opt: &Option<String>,
    list: &LinkedList<i32>,
    deque: &VecDeque<FloatType>,
) {
    println!("\n--- 3. The same functions, other witnesses ---");
    println!("  OptionWitness     {opt:?}");
    println!("  LinkedListWitness {list:?}");
    println!("  VecDequeWitness   {deque:?}");
}

fn print_debug(
    opt: &Shown<'_, OptionWitness, String>,
    xs: &Shown<'_, VecWitness, i32>,
    deque: &Shown<'_, VecDequeWitness, FloatType>,
) {
    println!("\n--- 4. DebugFunctor: formatting supplied by the witness ---");
    println!("  OptionWitness     {opt:?}");
    println!("  VecWitness        {xs:?}");
    println!("  VecDequeWitness   {deque:?}");
}
