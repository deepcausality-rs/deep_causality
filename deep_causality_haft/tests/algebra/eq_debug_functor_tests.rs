/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the opt-in `EqFunctor`/`DebugFunctor` capability and the `PartialEq`/`Eq`/`Debug`
//! instances they give `Free` and `Cofree`.
//!
//! These are derived instances, not categorical laws (the `Cofree` comonad laws live in
//! `formalization_lean/cofree_tests.rs`), so they carry no THEOREM_MAP id. They check that the
//! witness capability agrees with the underlying container, and that the induced `PartialEq` is a
//! structural equivalence while `Debug` mirrors the derive shape. Opt-in (a witness *without* the
//! capability yields no instance) is a `compile_fail` doctest on `EqFunctor`.

use std::collections::{LinkedList, VecDeque};

use deep_causality_haft::{
    BoxWitness, Cofree, DebugFunctor, EqFunctor, Free, LinkedListWitness, OptionWitness,
    VecDequeWitness, VecWitness,
};

// Render a `W::Type<T>` through its `DebugFunctor::fmt_type` (which takes a `Formatter`).
fn dbg_via<W, T>(x: &W::Type<T>) -> String
where
    W: DebugFunctor,
    T: core::fmt::Debug,
{
    struct Adapter<'a, W: DebugFunctor, T>(&'a W::Type<T>);
    impl<W: DebugFunctor, T: core::fmt::Debug> core::fmt::Display for Adapter<'_, W, T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            W::fmt_type(self.0, f)
        }
    }
    format!("{}", Adapter::<W, T>(x))
}

// ---- 1.3 : eq_type / fmt_type agree with the underlying container ----

#[test]
fn test_eq_functor_matches_container_eq() {
    let none: Option<i32> = None;
    assert!(OptionWitness::eq_type(&Some(1), &Some(1)));
    assert!(!OptionWitness::eq_type(&Some(1), &Some(2)));
    assert!(!OptionWitness::eq_type(&Some(1), &none));

    assert!(VecWitness::eq_type(&vec![1, 2, 3], &vec![1, 2, 3]));
    assert!(!VecWitness::eq_type(&vec![1, 2], &vec![1, 2, 3]));

    assert!(BoxWitness::eq_type(&Box::new(9), &Box::new(9)));
    assert!(!BoxWitness::eq_type(&Box::new(9), &Box::new(8)));

    let l1: LinkedList<i32> = [1, 2].into_iter().collect();
    let l2: LinkedList<i32> = [1, 2].into_iter().collect();
    let l3: LinkedList<i32> = [1, 3].into_iter().collect();
    assert!(LinkedListWitness::eq_type(&l1, &l2));
    assert!(!LinkedListWitness::eq_type(&l1, &l3));

    let d1: VecDeque<i32> = [1, 2].into_iter().collect();
    let d2: VecDeque<i32> = [1, 2].into_iter().collect();
    let d3: VecDeque<i32> = [2, 1].into_iter().collect();
    assert!(VecDequeWitness::eq_type(&d1, &d2));
    assert!(!VecDequeWitness::eq_type(&d1, &d3));
}

#[test]
fn test_debug_functor_matches_container_debug() {
    assert_eq!(
        dbg_via::<OptionWitness, i32>(&Some(7)),
        format!("{:?}", Some(7))
    );
    assert_eq!(
        dbg_via::<VecWitness, i32>(&vec![1, 2]),
        format!("{:?}", vec![1, 2])
    );
    assert_eq!(
        dbg_via::<BoxWitness, i32>(&Box::new(3)),
        format!("{:?}", Box::new(3))
    );
    let l: LinkedList<i32> = [4, 5].into_iter().collect();
    assert_eq!(dbg_via::<LinkedListWitness, i32>(&l), format!("{l:?}"));
    let d: VecDeque<i32> = [6, 7].into_iter().collect();
    assert_eq!(dbg_via::<VecDequeWitness, i32>(&d), format!("{d:?}"));
}

// ---- 2.3 : Free PartialEq is a structural equivalence; Debug mirrors the derive shape ----

// `Free` is not `Clone`, so build a fresh tree each call.
fn free_opt() -> Free<OptionWitness, i32> {
    Free::Suspend(Some(Box::new(Free::Suspend(Some(Box::new(Free::Pure(7)))))))
}
fn free_vec() -> Free<VecWitness, i32> {
    Free::Suspend(vec![
        Box::new(Free::Pure(1)),
        Box::new(Free::Suspend(vec![Box::new(Free::Pure(2))])),
    ])
}

#[test]
fn test_free_partial_eq_reflexive() {
    assert_eq!(free_opt(), free_opt());
    assert_eq!(free_vec(), free_vec());
}

#[test]
fn test_free_partial_eq_symmetric_and_inequality() {
    let (a, b) = (free_opt(), free_opt());
    assert!(a == b && b == a);

    let leaf: Free<OptionWitness, i32> = Free::Pure(7);
    let node = free_opt();
    assert!(leaf != node && node != leaf);
}

#[test]
fn test_free_partial_eq_transitive() {
    let (a, b, c) = (free_vec(), free_vec(), free_vec());
    assert!(a == b && b == c && a == c);
}

#[test]
fn test_free_debug_mirrors_derive_shape() {
    let opt: Free<OptionWitness, i32> = Free::Suspend(Some(Box::new(Free::Pure(7))));
    assert_eq!(format!("{opt:?}"), "Suspend(Some(Pure(7)))");

    let vec_tree: Free<VecWitness, i32> =
        Free::Suspend(vec![Box::new(Free::Pure(1)), Box::new(Free::Pure(2))]);
    assert_eq!(format!("{vec_tree:?}"), "Suspend([Pure(1), Pure(2)])");

    let leaf: Free<OptionWitness, i32> = Free::Pure(42);
    assert_eq!(format!("{leaf:?}"), "Pure(42)");
}

// ---- 4.2 : Cofree PartialEq / Eq / Debug ----

fn cofree_vec() -> Cofree<VecWitness, i32> {
    Cofree::new(
        1,
        vec![
            Box::new(Cofree::new(2, vec![])),
            Box::new(Cofree::new(3, vec![])),
        ],
    )
}

#[test]
fn test_cofree_partial_eq_equivalence() {
    // reflexive
    assert_eq!(cofree_vec(), cofree_vec());
    // symmetric
    let (a, b) = (cofree_vec(), cofree_vec());
    assert!(a == b && b == a);
    // transitive
    let (a, b, c) = (cofree_vec(), cofree_vec(), cofree_vec());
    assert!(a == b && b == c && a == c);
    // inequality
    let diff: Cofree<VecWitness, i32> = Cofree::new(1, vec![]);
    assert!(cofree_vec() != diff);
}

#[test]
fn test_free_and_cofree_resolve_eq_and_debug() {
    // Witness that the `Eq` marker and `Debug` impls resolve for representative witnesses
    // (the property tests above exercise `PartialEq`, not the `Eq` marker directly).
    fn assert_eq_debug<T: Eq + core::fmt::Debug>() {}
    assert_eq_debug::<Free<OptionWitness, i32>>();
    assert_eq_debug::<Free<VecWitness, i32>>();
    assert_eq_debug::<Cofree<VecWitness, i32>>();
    assert_eq_debug::<Cofree<OptionWitness, i32>>();
}

#[test]
fn test_cofree_debug() {
    let leaf: Cofree<VecWitness, i32> = Cofree::new(5, vec![]);
    assert_eq!(format!("{leaf:?}"), "Cofree { head: 5, tail: [] }");

    let tree: Cofree<VecWitness, i32> = Cofree::new(1, vec![Box::new(Cofree::new(2, vec![]))]);
    assert_eq!(
        format!("{tree:?}"),
        "Cofree { head: 1, tail: [Cofree { head: 2, tail: [] }] }"
    );
}
