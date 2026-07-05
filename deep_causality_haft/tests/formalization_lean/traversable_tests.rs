/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/Traversable.lean` (McBride–Paterson 2008 §3;
//! Jaskelioff–Rypacek 2012).

use deep_causality_haft::{
    Applicative, Functor, HKT, NoConstraint, OptionWitness, Pure, Satisfies, Traversable,
};

// The Identity applicative — required by the ACCEPTED identity law (the docstring's own
// version is vacuous; deviation D5).
#[derive(Debug, PartialEq, Clone)]
struct Ident<T>(T);
struct IdentWitness;
impl HKT for IdentWitness {
    type Constraint = NoConstraint;
    type Type<T> = Ident<T>;
}
impl Functor<IdentWitness> for IdentWitness {
    fn fmap<A, B, Func>(m_a: Ident<A>, mut f: Func) -> Ident<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        Ident(f(m_a.0))
    }
}
impl Pure<IdentWitness> for IdentWitness {
    fn pure<T>(value: T) -> Ident<T>
    where
        T: Satisfies<NoConstraint>,
    {
        Ident(value)
    }
}
impl Applicative<IdentWitness> for IdentWitness {
    fn apply<A, B, Func>(f_ab: Ident<Func>, f_a: Ident<A>) -> Ident<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint> + FnMut(A) -> B,
    {
        let mut f = f_ab.0;
        Ident(f(f_a.0))
    }
}

/// THEOREM_MAP: haft.traversable.identity
#[test]
fn test_traversable_identity() {
    // sequence at the Identity applicative is the identity (up to the Ident wrapper).
    assert_eq!(
        OptionWitness::sequence::<i32, IdentWitness>(Some(Ident(3))),
        Ident(Some(3))
    );
    assert_eq!(
        OptionWitness::sequence::<i32, IdentWitness>(None),
        Ident(None)
    );
}

/// THEOREM_MAP: haft.traversable.naturality
#[test]
fn test_traversable_naturality() {
    // φ : Ident → Option, φ(Ident(a)) = Some(a) — an applicative morphism.
    // Naturality: φ (sequence_Ident x) = sequence_Option (fmap φ x).
    fn phi<T>(i: Ident<T>) -> Option<T> {
        Some(i.0)
    }
    for x in [Some(Ident(3)), None::<Ident<i32>>] {
        let lhs: Option<Option<i32>> = phi(OptionWitness::sequence::<i32, IdentWitness>(x.clone()));
        let rhs: Option<Option<i32>> =
            OptionWitness::sequence::<i32, OptionWitness>(OptionWitness::fmap(x, phi));
        assert_eq!(lhs, rhs);
    }
}
