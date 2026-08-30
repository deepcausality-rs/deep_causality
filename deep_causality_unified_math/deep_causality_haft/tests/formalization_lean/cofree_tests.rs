/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the cofree-comonad laws.
//!
//! Mirrors `lean/DeepCausalityFormal/Haft/Cofree.lean` (the dual of `FreeMonad.lean`, reusing
//! `Comonad.lean`'s three coKleisli laws). `Cofree<F, A>` has `PartialEq`/`Debug` here via
//! `VecWitness: EqFunctor + DebugFunctor`, so trees are compared directly with `assert_eq!` — no
//! fold oracle is needed. `Cofree` is instantiated over `VecWitness` (a multi-hole functor that
//! bottoms out on the empty `Vec`, so the trees are finite). One `#[test]` per THEOREM_MAP id, plus
//! inherent-surface behaviour.
//!
//! The inherent `extend` consumes `self`; the associativity witness uses a test-local deep clone
//! (`clone_tree`) to model the value-consuming semantics of the Lean proof, where the law re-observes
//! a sub-context.

use deep_causality_haft::{Cofree, VecWitness};

// A finite labelled tree, fresh each call (`Cofree` is not `Clone`).
fn build() -> Cofree<VecWitness, i32> {
    Cofree::new(
        1,
        vec![
            Box::new(Cofree::new(2, vec![Box::new(Cofree::new(4, vec![]))])),
            Box::new(Cofree::new(3, vec![])),
        ],
    )
}

// Test-local deep clone via the public accessors (models value consumption in the by-value API).
fn clone_tree(w: &Cofree<VecWitness, i32>) -> Cofree<VecWitness, i32> {
    let kids = w
        .tail()
        .iter()
        .map(|b| Box::new(clone_tree(b)))
        .collect::<Vec<_>>();
    Cofree::new(*w.head(), kids)
}

// ---- haft.cofree.comonad_laws : left/right identity + associativity (Uustalu–Vene 2008) ----

/// THEOREM_MAP: haft.cofree.comonad_laws
#[test]
fn test_cofree_comonad_left_identity() {
    // extend extract = id
    let got = build().extend(&|w: &Cofree<VecWitness, i32>| w.extract());
    assert_eq!(got, build());
}

/// THEOREM_MAP: haft.cofree.comonad_laws
#[test]
fn test_cofree_comonad_right_identity() {
    // extract ∘ extend f = f
    let w = build();
    let f = |c: &Cofree<VecWitness, i32>| c.extract() * 2 + c.tail().len() as i32;
    let expected = f(&w);
    let got = w.extend(&f).extract();
    assert_eq!(got, expected);
}

/// THEOREM_MAP: haft.cofree.comonad_laws
#[test]
fn test_cofree_comonad_associativity() {
    // extend g (extend f w) = extend (|w'| g (extend f w')) w
    let f = |c: &Cofree<VecWitness, i32>| c.extract() + c.tail().len() as i32;
    let g = |c: &Cofree<VecWitness, i32>| c.extract() * 10;

    let lhs = clone_tree(&build()).extend(&f).extend(&g);
    let rhs =
        build().extend(&|w_prime: &Cofree<VecWitness, i32>| g(&clone_tree(w_prime).extend(&f)));
    assert_eq!(lhs, rhs);
}

// ---- haft.cofree.unfold : the anamorphism (dual of Free::fold) ----

/// THEOREM_MAP: haft.cofree.unfold
#[test]
fn test_cofree_unfold_builds_finite_tree() {
    // seed = remaining depth; each node is labelled by its depth with two depth-1 children until 0.
    let tree = Cofree::<VecWitness, i32>::unfold(2, &|d: i32| {
        let kids = if d > 0 { vec![d - 1, d - 1] } else { vec![] };
        (d, kids)
    });
    let leaf = || Box::new(Cofree::<VecWitness, i32>::new(0, vec![]));
    let mid = || Box::new(Cofree::<VecWitness, i32>::new(1, vec![leaf(), leaf()]));
    let expected = Cofree::new(2, vec![mid(), mid()]);
    assert_eq!(tree, expected);
}

/// THEOREM_MAP: haft.cofree.unfold
#[test]
fn test_cofree_unfold_extract_is_coalgebra_head() {
    // extract (unfold c s) = (c s).0
    let root = Cofree::<VecWitness, i32>::unfold(5, &|d: i32| {
        (d * 10, if d > 0 { vec![d - 1] } else { vec![] })
    });
    assert_eq!(root.extract(), 50);
}

// ---- inherent-surface behaviour (extract / map) ----

#[test]
fn test_cofree_extract_reads_head() {
    assert_eq!(build().extract(), 1);
}

// The "Cofree is the dual product of Free" scenario: new(head, tail) is inspected by
// head()/tail() and decomposed by into_parts() into the same (head, tail).
#[test]
fn test_cofree_new_accessors_and_into_parts() {
    let child = Box::new(Cofree::<VecWitness, i32>::new(8, vec![]));
    let w = Cofree::<VecWitness, i32>::new(7, vec![child]);

    // inspection
    assert_eq!(*w.head(), 7);
    assert_eq!(w.tail().len(), 1);
    assert_eq!(*w.tail()[0].head(), 8);

    // decomposition returns (head, tail) in that order — reconstructing via new round-trips.
    let (head, tail) = w.into_parts();
    assert_eq!(head, 7);
    assert_eq!(tail.len(), 1);
    let rebuilt = Cofree::<VecWitness, i32>::new(head, tail);
    assert_eq!(
        rebuilt,
        Cofree::new(7, vec![Box::new(Cofree::new(8, vec![]))])
    );
}

#[test]
fn test_cofree_map_relabels_preserving_shape() {
    let mapped = build().map(|x| x + 100);
    let expected = Cofree::new(
        101,
        vec![
            Box::new(Cofree::new(102, vec![Box::new(Cofree::new(104, vec![]))])),
            Box::new(Cofree::new(103, vec![])),
        ],
    );
    assert_eq!(mapped, expected);
}
