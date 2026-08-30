/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tier 3 `NaturalIso4<F, G>` round-trip tests (arity-4 HKT4Unbound).

use deep_causality_haft::{HKT4Unbound, NaturalIso4, NoConstraint, Satisfies};

// =============================================================================
// Fixtures
// =============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct Quadruple<A, B, C, D>(A, B, C, D);

#[derive(Debug, Clone, PartialEq, Eq)]
struct MyQuadruple<A, B, C, D> {
    a: A,
    b: B,
    c: C,
    d: D,
}

struct QuadrupleWitness;

impl HKT4Unbound for QuadrupleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C, D>
        = Quadruple<A, B, C, D>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>;
}

struct MyQuadrupleWitness;

impl HKT4Unbound for MyQuadrupleWitness {
    type Constraint = NoConstraint;
    type Type<A, B, C, D>
        = MyQuadruple<A, B, C, D>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint>;
}

struct QuadrupleMyQuadrupleIso;

impl NaturalIso4<QuadrupleWitness, MyQuadrupleWitness> for QuadrupleMyQuadrupleIso {
    fn to_target<A, B, C, D>(fa: Quadruple<A, B, C, D>) -> MyQuadruple<A, B, C, D>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        MyQuadruple {
            a: fa.0,
            b: fa.1,
            c: fa.2,
            d: fa.3,
        }
    }

    fn to_source<A, B, C, D>(ga: MyQuadruple<A, B, C, D>) -> Quadruple<A, B, C, D>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        C: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        D: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
    {
        Quadruple(ga.a, ga.b, ga.c, ga.d)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[test]
fn arity4_to_target_maps_fields_correctly() {
    let out: MyQuadruple<i32, &str, bool, f64> = <QuadrupleMyQuadrupleIso as NaturalIso4<
        QuadrupleWitness,
        MyQuadrupleWitness,
    >>::to_target(Quadruple(1, "two", true, 3.5));
    assert_eq!(
        out,
        MyQuadruple {
            a: 1,
            b: "two",
            c: true,
            d: 3.5,
        }
    );
}

#[test]
fn arity4_to_source_maps_fields_correctly() {
    let out: Quadruple<i32, &str, bool, f64> = <QuadrupleMyQuadrupleIso as NaturalIso4<
        QuadrupleWitness,
        MyQuadrupleWitness,
    >>::to_source(MyQuadruple {
        a: 1,
        b: "two",
        c: true,
        d: 3.5,
    });
    assert_eq!(out, Quadruple(1, "two", true, 3.5));
}

#[test]
fn arity4_round_trip_holds_for_canonical_iso() {
    let fa = Quadruple(1i32, "two", true, 3.5f64);
    let target =
        <QuadrupleMyQuadrupleIso as NaturalIso4<QuadrupleWitness, MyQuadrupleWitness>>::to_target(
            fa.clone(),
        );
    let back =
        <QuadrupleMyQuadrupleIso as NaturalIso4<QuadrupleWitness, MyQuadrupleWitness>>::to_source(
            target,
        );
    assert_eq!(fa, back);

    let ga = MyQuadruple {
        a: 1i32,
        b: "two",
        c: true,
        d: 3.5f64,
    };
    let source =
        <QuadrupleMyQuadrupleIso as NaturalIso4<QuadrupleWitness, MyQuadrupleWitness>>::to_source(
            ga.clone(),
        );
    let back2 =
        <QuadrupleMyQuadrupleIso as NaturalIso4<QuadrupleWitness, MyQuadrupleWitness>>::to_target(
            source,
        );
    assert_eq!(ga, back2);
}
