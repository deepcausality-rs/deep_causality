/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier 3 `NaturalIso3<F, G>` round-trip tests (arity-3 HKT3Unbound).

use deep_causality_haft::{HKT3Unbound, NaturalIso3, NoConstraint, Satisfies};

// =============================================================================
// Fixtures
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Triple<A, B, C>(A, B, C);

#[derive(Debug, Clone, PartialEq, Eq)]
struct MyTriple<A, B, C> {
    a: A,
    b: B,
    c: C,
}

struct TripleWitness;

impl HKT3Unbound for TripleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C>
        = Triple<A, B, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>;
}

struct MyTripleWitness;

impl HKT3Unbound for MyTripleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C>
        = MyTriple<A, B, C>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>;
}

struct TripleMyTripleIso;

impl NaturalIso3<TripleWitness, MyTripleWitness> for TripleMyTripleIso {
    fn to_target<A, B, C>(fa: Triple<A, B, C>) -> MyTriple<A, B, C>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        MyTriple {
            a: fa.0,
            b: fa.1,
            c: fa.2,
        }
    }

    fn to_source<A, B, C>(ga: MyTriple<A, B, C>) -> Triple<A, B, C>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        Triple(ga.a, ga.b, ga.c)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn arity3_to_target_maps_fields_correctly() {
    let out: MyTriple<i32, &str, bool> = <TripleMyTripleIso as NaturalIso3<
        TripleWitness,
        MyTripleWitness,
    >>::to_target(Triple(1, "two", true));
    assert_eq!(
        out,
        MyTriple {
            a: 1,
            b: "two",
            c: true
        }
    );
}

#[test]
fn arity3_to_source_maps_fields_correctly() {
    let out: Triple<i32, &str, bool> =
        <TripleMyTripleIso as NaturalIso3<TripleWitness, MyTripleWitness>>::to_source(MyTriple {
            a: 1,
            b: "two",
            c: true,
        });
    assert_eq!(out, Triple(1, "two", true));
}

#[test]
fn arity3_round_trip_holds_for_canonical_iso() {
    let fa = Triple(1i32, "two", true);
    let target =
        <TripleMyTripleIso as NaturalIso3<TripleWitness, MyTripleWitness>>::to_target(fa.clone());
    let back =
        <TripleMyTripleIso as NaturalIso3<TripleWitness, MyTripleWitness>>::to_source(target);
    assert_eq!(fa, back);

    let ga = MyTriple {
        a: 1i32,
        b: "two",
        c: true,
    };
    let source =
        <TripleMyTripleIso as NaturalIso3<TripleWitness, MyTripleWitness>>::to_source(ga.clone());
    let back2 =
        <TripleMyTripleIso as NaturalIso3<TripleWitness, MyTripleWitness>>::to_target(source);
    assert_eq!(ga, back2);
}
