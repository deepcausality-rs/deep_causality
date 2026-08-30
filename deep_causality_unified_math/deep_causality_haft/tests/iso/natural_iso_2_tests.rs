/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier 3 `NaturalIso2<F, G>` round-trip tests (arity-2 HKT2Unbound).

use deep_causality_haft::{HKT2Unbound, NaturalIso2, NoConstraint, Satisfies};

// =============================================================================
// Fixtures
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Pair<A, B>(A, B);

#[derive(Debug, Clone, PartialEq, Eq)]
struct MyPair<A, B> {
    a: A,
    b: B,
}

struct PairWitness;

impl HKT2Unbound for PairWitness {
    type Constraint = NoConstraint;
    type Type<A, B>
        = Pair<A, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>;
}

struct MyPairWitness;

impl HKT2Unbound for MyPairWitness {
    type Constraint = NoConstraint;
    type Type<A, B>
        = MyPair<A, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>;
}

struct PairMyPairIso;

impl NaturalIso2<PairWitness, MyPairWitness> for PairMyPairIso {
    fn to_target<A, B>(fa: Pair<A, B>) -> MyPair<A, B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        MyPair { a: fa.0, b: fa.1 }
    }

    fn to_source<A, B>(ga: MyPair<A, B>) -> Pair<A, B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        Pair(ga.a, ga.b)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn arity2_to_target_maps_fields_correctly() {
    let out: MyPair<i32, &str> =
        <PairMyPairIso as NaturalIso2<PairWitness, MyPairWitness>>::to_target(Pair(7, "hi"));
    assert_eq!(out, MyPair { a: 7, b: "hi" });
}

#[test]
fn arity2_to_source_maps_fields_correctly() {
    let out: Pair<i32, &str> =
        <PairMyPairIso as NaturalIso2<PairWitness, MyPairWitness>>::to_source(MyPair {
            a: 7,
            b: "hi",
        });
    assert_eq!(out, Pair(7, "hi"));
}

#[test]
fn arity2_round_trip_holds_for_canonical_iso() {
    let fa = Pair(42i32, true);
    let target = <PairMyPairIso as NaturalIso2<PairWitness, MyPairWitness>>::to_target(fa.clone());
    let back = <PairMyPairIso as NaturalIso2<PairWitness, MyPairWitness>>::to_source(target);
    assert_eq!(fa, back);

    let ga = MyPair { a: 42i32, b: true };
    let source = <PairMyPairIso as NaturalIso2<PairWitness, MyPairWitness>>::to_source(ga.clone());
    let back2 = <PairMyPairIso as NaturalIso2<PairWitness, MyPairWitness>>::to_target(source);
    assert_eq!(ga, back2);
}
