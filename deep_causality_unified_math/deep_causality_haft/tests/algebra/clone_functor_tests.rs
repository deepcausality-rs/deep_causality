/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the opt-in `CloneFunctor` capability and the `Clone` instances it gives `Free` and
//! `Cofree`, plus the `Cofree` comonad surface it unblocks: the inherent `duplicate` and the
//! by-reference `CoMonad`/`Functor` **trait** impls for `CofreeWitness`.
//!
//! These are derived / structurally-defined instances, not new categorical laws (the `Cofree`
//! comonad laws live in `formalization_lean/cofree_tests.rs`), so they carry no THEOREM_MAP id. They
//! check that the witness capability agrees with the underlying container's `Clone`, that the
//! induced `Clone` reproduces the tree, and that the `CoMonad` trait ops satisfy the three comonad
//! laws for `Cofree`. Opt-in (a witness *without* the capability yields no `Clone`) is a
//! `compile_fail` doctest on `CloneFunctor`.

use std::collections::{LinkedList, VecDeque};

use deep_causality_haft::{
    BoxWitness, CloneFunctor, CoMonad, Cofree, CofreeWitness, Free, Functor, LinkedListWitness,
    OptionWitness, VecDequeWitness, VecWitness,
};

// ---- clone_type agrees with the underlying container's Clone ----

#[test]
fn test_clone_functor_matches_container_clone() {
    let o: Option<i32> = Some(3);
    assert_eq!(OptionWitness::clone_type(&o), o);
    let none: Option<i32> = None;
    assert_eq!(OptionWitness::clone_type(&none), none);

    let v = vec![1, 2, 3];
    assert_eq!(VecWitness::clone_type(&v), v);

    let b = Box::new(9);
    assert_eq!(BoxWitness::clone_type(&b), b);

    let l: LinkedList<i32> = [1, 2].into_iter().collect();
    assert_eq!(LinkedListWitness::clone_type(&l), l);

    let d: VecDeque<i32> = [4, 5].into_iter().collect();
    assert_eq!(VecDequeWitness::clone_type(&d), d);
}

// ---- Clone for Free (recursion through F::clone_type terminates) ----

#[test]
fn test_free_clone_equals_original() {
    let f_opt: Free<OptionWitness, i32> =
        Free::Suspend(Some(Box::new(Free::Suspend(Some(Box::new(Free::Pure(7)))))));
    assert_eq!(f_opt.clone(), f_opt);

    let f_vec: Free<VecWitness, i32> = Free::Suspend(vec![
        Box::new(Free::Pure(1)),
        Box::new(Free::Suspend(vec![Box::new(Free::Pure(2))])),
    ]);
    assert_eq!(f_vec.clone(), f_vec);

    // Pure-leaf branch on its own.
    let leaf: Free<OptionWitness, i32> = Free::Pure(42);
    assert_eq!(leaf.clone(), leaf);
}

// ---- Clone for Cofree ----

fn cofree_vec() -> Cofree<VecWitness, i32> {
    Cofree::new(
        1,
        vec![
            Box::new(Cofree::new(2, vec![])),
            Box::new(Cofree::new(3, vec![Box::new(Cofree::new(4, vec![]))])),
        ],
    )
}

#[test]
fn test_cofree_clone_equals_original() {
    let w = cofree_vec();
    assert_eq!(w.clone(), w);

    // Over a second witness, exercising the Option `clone_type` path in the recursion.
    let w_opt: Cofree<OptionWitness, i32> = Cofree::new(1, Some(Box::new(Cofree::new(2, None))));
    assert_eq!(w_opt.clone(), w_opt);
}

// ---- duplicate (D2): the inherent comonadic `duplicate` = `extend (|w| w.clone())` ----

#[test]
fn test_cofree_inherent_duplicate() {
    let w = cofree_vec();
    let dup = w.clone().duplicate(); // Cofree<VecWitness, Cofree<VecWitness, i32>>

    // The label at each position is the whole sub-tree focused there; the root's label is `w`.
    assert_eq!(dup.head(), &w);
    // Comonad `extract ∘ duplicate = id` (inherent `extract` reads the head).
    assert_eq!(dup.extract(), w);
}

// ---- CoMonad trait (D4): the by-reference comonad instance for CofreeWitness ----

#[test]
fn test_cofree_comonad_extract_reads_head() {
    let w = cofree_vec();
    assert_eq!(CofreeWitness::<VecWitness>::extract(&w), 1);
}

#[test]
fn test_cofree_comonad_left_identity() {
    // extend(w, extract) == w
    let w = cofree_vec();
    let extended = CofreeWitness::<VecWitness>::extend(&w, |c: &Cofree<VecWitness, i32>| *c.head());
    assert_eq!(extended, w);
}

#[test]
fn test_cofree_comonad_right_identity() {
    // extract(extend(w, f)) == f(w)
    let w = cofree_vec();
    let f = |c: &Cofree<VecWitness, i32>| *c.head() + 100;
    let extended = CofreeWitness::<VecWitness>::extend(&w, f);
    assert_eq!(CofreeWitness::<VecWitness>::extract(&extended), f(&w));
}

#[test]
fn test_cofree_comonad_associativity() {
    // extend(extend(w, f), g) == extend(w, |w'| g(&extend(w', f)))
    let w = cofree_vec();
    let f = |c: &Cofree<VecWitness, i32>| *c.head() + 1;
    let g = |c: &Cofree<VecWitness, i32>| *c.head() * 2;

    let lhs = CofreeWitness::<VecWitness>::extend(&CofreeWitness::<VecWitness>::extend(&w, f), g);
    let rhs = CofreeWitness::<VecWitness>::extend(&w, |wp: &Cofree<VecWitness, i32>| {
        g(&CofreeWitness::<VecWitness>::extend(wp, f))
    });
    assert_eq!(lhs, rhs);
}

#[test]
fn test_cofree_comonad_duplicate_default_matches_inherent() {
    // The CoMonad trait's default `duplicate` and the inherent `duplicate` agree: the root label is
    // the whole tree, and extracting it recovers the original.
    let w = cofree_vec();
    let dup = CofreeWitness::<VecWitness>::duplicate(&w); // Cofree<VecWitness, Cofree<VecWitness, i32>>
    assert_eq!(dup.head(), &w);
    assert_eq!(dup, w.clone().duplicate());
}

// ---- Functor trait for CofreeWitness matches the inherent `map` for a pure function ----

#[test]
fn test_cofree_functor_trait_matches_inherent_map() {
    let w = cofree_vec();
    let via_trait = CofreeWitness::<VecWitness>::fmap(w.clone(), |x| x * 10);
    let via_inherent = w.map(|x| x * 10);
    assert_eq!(via_trait, via_inherent);
}
