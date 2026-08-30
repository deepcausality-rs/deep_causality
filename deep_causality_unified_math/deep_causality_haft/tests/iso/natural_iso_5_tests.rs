/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier 3 `NaturalIso5<F, G>` round-trip tests (arity-5 HKT5Unbound).

use deep_causality_haft::{HKT5Unbound, NaturalIso5, NoConstraint, Satisfies};

// =============================================================================
// Fixtures
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Quintuple<V, S, C, E, L>(V, S, C, E, L);

#[derive(Debug, Clone, PartialEq, Eq)]
struct MyQuintuple<V, S, C, E, L> {
    v: V,
    s: S,
    c: C,
    e: E,
    l: L,
}

struct QuintupleWitness;

impl HKT5Unbound for QuintupleWitness {
    type Constraint = NoConstraint;
    type Type<V, S, C, E, L>
        = Quintuple<V, S, C, E, L>
    where
        V: Satisfies<NoConstraint>,
        S: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint>,
        L: Satisfies<NoConstraint>;
}

struct MyQuintupleWitness;

impl HKT5Unbound for MyQuintupleWitness {
    type Constraint = NoConstraint;
    type Type<V, S, C, E, L>
        = MyQuintuple<V, S, C, E, L>
    where
        V: Satisfies<NoConstraint>,
        S: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint>,
        L: Satisfies<NoConstraint>;
}

struct QuintupleMyQuintupleIso;

impl NaturalIso5<QuintupleWitness, MyQuintupleWitness> for QuintupleMyQuintupleIso {
    fn to_target<V, S, C, E, L>(fa: Quintuple<V, S, C, E, L>) -> MyQuintuple<V, S, C, E, L>
    where
        V: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        S: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        L: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        MyQuintuple {
            v: fa.0,
            s: fa.1,
            c: fa.2,
            e: fa.3,
            l: fa.4,
        }
    }

    fn to_source<V, S, C, E, L>(ga: MyQuintuple<V, S, C, E, L>) -> Quintuple<V, S, C, E, L>
    where
        V: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        S: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        E: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        L: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        Quintuple(ga.v, ga.s, ga.c, ga.e, ga.l)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn arity5_to_target_maps_fields_correctly() {
    let out: MyQuintuple<i32, &str, bool, f64, u8> =
        <QuintupleMyQuintupleIso as NaturalIso5<QuintupleWitness, MyQuintupleWitness>>::to_target(
            Quintuple(1, "two", true, 3.5, 9u8),
        );
    assert_eq!(
        out,
        MyQuintuple {
            v: 1,
            s: "two",
            c: true,
            e: 3.5,
            l: 9u8,
        }
    );
}

#[test]
fn arity5_to_source_maps_fields_correctly() {
    let out: Quintuple<i32, &str, bool, f64, u8> = <QuintupleMyQuintupleIso as NaturalIso5<
        QuintupleWitness,
        MyQuintupleWitness,
    >>::to_source(MyQuintuple {
        v: 1,
        s: "two",
        c: true,
        e: 3.5,
        l: 9u8,
    });
    assert_eq!(out, Quintuple(1, "two", true, 3.5, 9u8));
}

#[test]
fn arity5_round_trip_holds_for_canonical_iso() {
    let fa = Quintuple(1i32, "two", true, 3.5f64, 9u8);
    let target =
        <QuintupleMyQuintupleIso as NaturalIso5<QuintupleWitness, MyQuintupleWitness>>::to_target(
            fa.clone(),
        );
    let back =
        <QuintupleMyQuintupleIso as NaturalIso5<QuintupleWitness, MyQuintupleWitness>>::to_source(
            target,
        );
    assert_eq!(fa, back);

    let ga = MyQuintuple {
        v: 1i32,
        s: "two",
        c: true,
        e: 3.5f64,
        l: 9u8,
    };
    let source =
        <QuintupleMyQuintupleIso as NaturalIso5<QuintupleWitness, MyQuintupleWitness>>::to_source(
            ga.clone(),
        );
    let back2 =
        <QuintupleMyQuintupleIso as NaturalIso5<QuintupleWitness, MyQuintupleWitness>>::to_target(
            source,
        );
    assert_eq!(ga, back2);
}
