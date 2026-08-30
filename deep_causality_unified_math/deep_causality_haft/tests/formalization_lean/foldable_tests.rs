/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Foldable.lean`.

use deep_causality_algebra::{Count, Monoid};
use deep_causality_haft::{Foldable, OptionWitness, Pure, VecWitness};

/// THEOREM_MAP: haft.foldable.pure_compat
#[test]
fn test_foldable_pure_compat() {
    // fold (pure x) init f = f init x
    let f = |acc: i32, x: i32| acc + x * 10;
    assert_eq!(OptionWitness::fold(OptionWitness::pure(4), 2, f), f(2, 4));
    // and the unit case: fold none init f = init
    assert_eq!(OptionWitness::fold(None::<i32>, 2, f), 2);
}

/// THEOREM_MAP: haft.foldable.fold_map_pure
#[test]
fn test_fold_map_pure() {
    // fold_map(pure a, f) = f a — the singleton law (via the monoid identity).
    let f = |x: u64| Count(x);
    assert_eq!(OptionWitness::fold_map(OptionWitness::pure(4u64), f), f(4));
    // The empty structure folds to the monoid identity.
    assert_eq!(OptionWitness::fold_map(None::<u64>, f), Count::empty());
}

/// THEOREM_MAP: haft.foldable.fold_map_monoid_coherence
#[test]
fn test_fold_map_monoid_coherence() {
    // Monoid-homomorphism coherence: `fold_map` over a concatenation is the `combine` of the
    // per-part `fold_map`s (and the empty part maps to `empty`).
    let f = |x: u64| Count(x);
    let xs = vec![1u64, 2, 3];
    let ys = vec![4u64, 5];
    let mut xy = xs.clone();
    xy.extend(ys.clone());
    let lhs = VecWitness::fold_map(xy, f);
    let rhs = VecWitness::fold_map(xs.clone(), f).combine(VecWitness::fold_map(ys, f));
    assert_eq!(lhs, rhs);

    // `Count` is a `CommutativeMonoid`, so `fold_map` is order-independent.
    let rev: Vec<u64> = xs.iter().rev().copied().collect();
    assert_eq!(VecWitness::fold_map(xs, f), VecWitness::fold_map(rev, f));
}
