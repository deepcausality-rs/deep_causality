/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/LaxMonoidal.lean` (Mac Lane CWM §XI.2;
//! McBride–Paterson 2008 §7).
//!
//! The Lean file transcribes the `Option` carrier. These tests exercise the same laws against a
//! real witness of the crate's traits. `OptionWitness` does not yet implement `Semigroupal`, so
//! the carrier here is a local witness over `Option` that does, transcribing the same φ and η.

use deep_causality_haft::{
    Convolutional, Functor, HKT, LaxMonoidal, MonoidalApplicative, NoConstraint, Satisfies,
    Semigroupal,
};

/// A local witness over `Option`, carrying the same φ and η the Lean file transcribes.
struct OptWitness;

impl HKT for OptWitness {
    type Constraint = NoConstraint;
    type Type<T> = Option<T>;
}

impl Functor<OptWitness> for OptWitness {
    fn fmap<A, B, Func>(fa: Option<A>, f: Func) -> Option<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        fa.map(f)
    }
}

impl Semigroupal<OptWitness> for OptWitness {
    fn zip_with<A, B, C, Func>(fa: Option<A>, fb: Option<B>, mut f: Func) -> Option<C>
    where
        Func: FnMut(A, B) -> C,
    {
        match (fa, fb) {
            (Some(a), Some(b)) => Some(f(a, b)),
            _ => None,
        }
    }
}

impl LaxMonoidal<OptWitness> for OptWitness {
    fn unit() -> Option<()> {
        Some(())
    }
}

impl Convolutional<OptWitness> for OptWitness {}
impl MonoidalApplicative<OptWitness> for OptWitness {}

/// Every inhabited/empty combination the carrier admits.
const CASES: [(Option<i32>, Option<char>); 4] = [
    (Some(3), Some('a')),
    (Some(3), None),
    (None, Some('a')),
    (None, None),
];

/// THEOREM_MAP: haft.lax_monoidal.naturality
#[test]
fn test_lax_monoidal_naturality() {
    let f = |x: i32| x * 2;
    let g = |c: char| c as u32;

    for (fa, fb) in CASES {
        let lhs = OptWitness::zip(OptWitness::fmap(fa, f), OptWitness::fmap(fb, g));
        let rhs = OptWitness::fmap(OptWitness::zip(fa, fb), |(a, b)| (f(a), g(b)));
        assert_eq!(lhs, rhs, "naturality failed on ({fa:?}, {fb:?})");
    }
}

/// THEOREM_MAP: haft.lax_monoidal.assoc
#[test]
fn test_lax_monoidal_assoc() {
    for (fa, fb) in CASES {
        for fc in [Some(true), None] {
            // zip(zip(fa, fb), fc) reassociated == zip(fa, zip(fb, fc))
            let lhs = OptWitness::fmap(
                OptWitness::zip(OptWitness::zip(fa, fb), fc),
                |((a, b), c)| (a, (b, c)),
            );
            let rhs = OptWitness::zip(fa, OptWitness::zip(fb, fc));
            assert_eq!(lhs, rhs, "assoc failed on ({fa:?}, {fb:?}, {fc:?})");
        }
    }

    // `zip` really is the derived form of `zip_with`, which is what lets it be a provided method.
    for (fa, fb) in CASES {
        assert_eq!(
            OptWitness::zip(fa, fb),
            OptWitness::zip_with(fa, fb, |a, b| (a, b))
        );
    }
}

/// THEOREM_MAP: haft.lax_monoidal.unit_laws
#[test]
fn test_lax_monoidal_unit_laws() {
    for fa in [Some(7), None] {
        let left = OptWitness::fmap(OptWitness::zip(OptWitness::unit(), fa), |((), a)| a);
        assert_eq!(left, fa, "left unit failed on {fa:?}");

        let right = OptWitness::fmap(OptWitness::zip(fa, OptWitness::unit()), |(a, ())| a);
        assert_eq!(right, fa, "right unit failed on {fa:?}");
    }
}

/// THEOREM_MAP: haft.lax_monoidal.apply_agreement
#[test]
fn test_lax_monoidal_apply_agreement() {
    let double: fn(i32) -> i32 = |x| x * 2;

    for ff in [Some(double), None] {
        for fa in [Some(21), None] {
            // The derived apply, against the hand-written form `f_ab.and_then(|f| f_a.map(f))`
            // that `OptionWitness::apply` uses and `Applicative.lean` transcribes as `optApply`.
            let derived = OptWitness::apply(ff, fa);
            let hand_written = ff.and_then(|f| fa.map(f));
            assert_eq!(derived, hand_written, "disagreement on ({ff:?}, {fa:?})");
        }
    }
}

/// The structure is genuinely Δ-free: `apply` runs over a payload that is neither `Clone` nor
/// `Copy`, which `Applicative::apply` could not accept because of its `A: Clone` bound.
#[test]
fn test_apply_is_diagonal_free() {
    #[derive(Debug, PartialEq)]
    struct Moved(u32);

    let f: fn(Moved) -> Moved = |m| Moved(m.0 + 1);
    assert_eq!(OptWitness::apply(Some(f), Some(Moved(41))), Some(Moved(42)));
}
