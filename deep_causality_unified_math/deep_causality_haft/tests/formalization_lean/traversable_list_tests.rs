/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/TraversableList.lean` (McBride–Paterson 2008 §3;
//! Jaskelioff–Rypacek 2012).
//!
//! `traversable_tests.rs` witnesses the laws for `OptionWitness::sequence`, whose carrier holds at
//! most one element. This file witnesses them for the *sequential* carrier `VecWitness`, whose
//! `sequence` is a left-to-right accumulator fold — a different proof in Lean (an induction with a
//! generalised accumulator) and so a separate witness here.
//!
//! `DenseVectorWitness` and `CausalTensorWitness` run the same fold and are witnessed in their own
//! crates; the Lean file models the shared kernel, and the three differ only in how the result is
//! rebuilt, which is a bijection on the element list.

use deep_causality_haft::{
    Applicative, Functor, HKT, OptionWitness, Pure, Traversable, VecWitness,
};

// The Identity applicative — required by the ACCEPTED identity law (the trait docstring's own
// version is vacuous; deviation D5), mirroring the fixture in `traversable_tests.rs`.
#[derive(Debug, PartialEq, Clone)]
struct Ident<T>(T);
struct IdentWitness;
impl HKT for IdentWitness {
    type Type<T> = Ident<T>;
}
impl Functor<IdentWitness> for IdentWitness {
    fn fmap<A, B, Func>(m_a: Ident<A>, mut f: Func) -> Ident<B>
    where
        Func: FnMut(A) -> B,
    {
        Ident(f(m_a.0))
    }
}
impl Pure<IdentWitness> for IdentWitness {
    fn pure<T>(value: T) -> Ident<T> {
        Ident(value)
    }
}
impl Applicative<IdentWitness> for IdentWitness {
    fn apply<A, B, Func>(f_ab: Ident<Func>, f_a: Ident<A>) -> Ident<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let mut f = f_ab.0;
        Ident(f(f_a.0))
    }
}

/// THEOREM_MAP: haft.traversable.list.identity
#[test]
fn test_traversable_list_identity() {
    // `sequence` at the Identity applicative is the identity. Exercised at three distinct
    // elements, so a permutation or dropped-element defect cannot pass, and at the empty list.
    assert_eq!(
        VecWitness::sequence::<i32, IdentWitness>(vec![Ident(1), Ident(2), Ident(3)]),
        Ident(vec![1, 2, 3])
    );
    assert_eq!(
        VecWitness::sequence::<i32, IdentWitness>(Vec::new()),
        Ident(Vec::new())
    );
}

/// THEOREM_MAP: haft.traversable.list.naturality
#[test]
fn test_traversable_list_naturality() {
    // φ : Ident → Option, φ(Ident(a)) = Some(a) — an applicative morphism.
    // Naturality: φ (sequence_Ident xs) = sequence_Option (fmap φ xs).
    fn phi<T>(i: Ident<T>) -> Option<T> {
        Some(i.0)
    }
    for xs in [
        vec![Ident(1), Ident(2), Ident(3)],
        vec![Ident(42)],
        Vec::<Ident<i32>>::new(),
    ] {
        let lhs: Option<Vec<i32>> = phi(VecWitness::sequence::<i32, IdentWitness>(xs.clone()));
        let rhs: Option<Vec<i32>> =
            VecWitness::sequence::<i32, OptionWitness>(VecWitness::fmap(xs, phi));
        assert_eq!(lhs, rhs);
    }
}

/// THEOREM_MAP: haft.traversable.list.length_preserved
#[test]
fn test_traversable_list_length_preserved() {
    // The element count is preserved. This is the formal content of the shape claim
    // `CausalTensorWitness::sequence` makes: a tensor re-wears the input's shape on the output,
    // which is sound exactly because the count cannot change.
    for n in [0usize, 1, 2, 5, 17] {
        let xs: Vec<Ident<i32>> = (0..n as i32).map(Ident).collect();
        let out = VecWitness::sequence::<i32, IdentWitness>(xs);
        assert_eq!(out.0.len(), n, "length preserved at n = {n}");
    }
}
