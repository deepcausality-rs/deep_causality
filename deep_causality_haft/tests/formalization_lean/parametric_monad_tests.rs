/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Haft/ParametricMonad.lean` (Atkey 2009).

use deep_causality_haft::{HKT3Unbound, NoConstraint, ParametricMonad, Satisfies};

// Phantom-indexed carrier, mirroring the crate's canonical parametric-monad test.
// The Lean model proves the laws for the full indexed-state monad; this carrier is its
// value-projection, so the value-level equalities below are the checkable fragment.
#[derive(Debug, Clone, PartialEq)]
struct IxVal<SIn, SOut, A> {
    value: A,
    _in: std::marker::PhantomData<SIn>,
    _out: std::marker::PhantomData<SOut>,
}

fn ixval<SIn, SOut, A>(value: A) -> IxVal<SIn, SOut, A> {
    IxVal {
        value,
        _in: std::marker::PhantomData,
        _out: std::marker::PhantomData,
    }
}

struct IxValWitness;
impl HKT3Unbound for IxValWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C> = IxVal<A, B, C>;
}

impl ParametricMonad<IxValWitness> for IxValWitness {
    fn pure<S, A>(value: A) -> IxVal<S, S, A>
    where
        S: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
    {
        ixval(value)
    }

    fn ibind<S1, S2, S3, A, B, F>(m: IxVal<S1, S2, A>, mut f: F) -> IxVal<S1, S3, B>
    where
        S1: Satisfies<NoConstraint>,
        S2: Satisfies<NoConstraint>,
        S3: Satisfies<NoConstraint>,
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        F: FnMut(A) -> IxVal<S2, S3, B>,
    {
        ixval(f(m.value).value)
    }
}

/// THEOREM_MAP: haft.parametric_monad.laws
#[test]
fn test_parametric_monad_laws() {
    // States are types: u8 → u16 → u32 → u64 — ibind composes the transitions.
    let f = |a: i32| -> IxVal<u16, u32, i32> { ixval(a + 1) };
    let g = |b: i32| -> IxVal<u32, u64, i32> { ixval(b * 2) };

    // Indexed left identity: ibind (pure a) f = f a
    let lhs: IxVal<u16, u32, i32> = IxValWitness::ibind(IxValWitness::pure(5), f);
    assert_eq!(lhs, f(5));

    // Indexed right identity: ibind m pure = m
    let m: IxVal<u8, u16, i32> = ixval(7);
    let rhs: IxVal<u8, u16, i32> = IxValWitness::ibind(m.clone(), IxValWitness::pure);
    assert_eq!(rhs, m);

    // Indexed associativity across u8 → u16 → u32 → u64
    let m: IxVal<u8, u16, i32> = ixval(3);
    let lhs: IxVal<u8, u64, i32> = IxValWitness::ibind(IxValWitness::ibind(m.clone(), f), g);
    let rhs: IxVal<u8, u64, i32> = IxValWitness::ibind(m, |a| IxValWitness::ibind(f(a), g));
    assert_eq!(lhs, rhs);
}
