/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Adjunction laws for `ChainWitness`.
//!
//! The crate has two `Adjunction` implementations and 38 tests across them, every one of which
//! checks `unit`, `counit`, `left_adjunct` or `right_adjunct` in isolation on a single fixture.
//! What makes an adjunction an adjunction is that the two adjuncts are mutually inverse, and that
//! was asserted nowhere. These are those laws, over generated chains.

use deep_causality_haft::Adjunction;
use deep_causality_topology::utils_tests::{approx_eq, chain_cases, path_complex, LawRng};
use deep_causality_linear::CsrMatrix;
use deep_causality_topology::{Chain, ChainWitness, SimplicialComplex};
use std::sync::Arc;

type Ctx = (Arc<SimplicialComplex<f64>>, usize);
const TOL: f64 = 1e-9;
const SEED: u64 = 0xAD_0F1;

fn ctx_for(n: usize, grade: usize) -> Ctx {
    (Arc::new(path_complex::<f64>(n)), grade)
}

/// The first stored weight, which is the value `counit` and `right_adjunct` both select.
fn first_weight(c: &Chain<f64>) -> f64 {
    *c.weights()
        .values()
        .first()
        .expect("generated chains are non-empty")
}

#[test]
fn counit_after_unit_is_the_identity() {
    // counit(unit(a)) == a. The unit wraps a value twice; the counit unwraps it twice.
    let mut rng = LawRng::new(SEED);
    for n in [2usize, 3, 4] {
        for grade in 0..2usize {
            let ctx = ctx_for(n, grade);
            for _ in 0..8 {
                let a = rng.well_scaled(9.0);
                let wrapped = <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::unit(
                    &ctx, a,
                );
                let back =
                    <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::counit(
                        &ctx, wrapped,
                    );
                assert!(
                    approx_eq(back, a, TOL),
                    "counit(unit(a)) != a for n={n} grade={grade}: got {back}, want {a}"
                );
            }
        }
    }
}

#[test]
fn right_adjunct_inverts_left_adjunct() {
    // right_adjunct(la, |a| left_adjunct(a, f)) == f(la).
    // This is the adjunction round-trip, and it is what the per-method tests cannot see.
    for case in chain_cases(SEED ^ 1) {
        let ctx = ctx_for(4, case.value.grade());
        let f = |c: Chain<f64>| first_weight(&c) * 2.0 + 1.0;

        let round_trip =
            <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::right_adjunct(
                &ctx,
                case.value.clone(),
                |a: f64| {
                    <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::left_adjunct(
                        &ctx, a, f,
                    )
                },
            );

        let direct = f(case.value.clone());
        assert!(
            approx_eq(round_trip, direct, TOL),
            "right_adjunct(la, left_adjunct(., f)) != f(la) for {}: got {round_trip}, want {direct}",
            case.label
        );
    }
}

#[test]
fn left_adjunct_inverts_right_adjunct() {
    // left_adjunct(a, |la| right_adjunct(la, g)) == g(a), compared on the stored weight.
    let mut rng = LawRng::new(SEED ^ 2);
    for n in [2usize, 3, 4] {
        let complex = Arc::new(path_complex::<f64>(n));
        let ctx: Ctx = (complex.clone(), 0);

        for _ in 0..8 {
            let a = rng.well_scaled(5.0);
            let k = rng.well_scaled(3.0);

            // A plain `A -> Chain<B>`: the one-entry chain holding `x * k`.
            let g = |x: f64| -> Chain<f64> {
                let w = CsrMatrix::from_triplets(1, n, &[(0, 0, x * k)])
                    .expect("single-entry chain weights");
                Chain::new(complex.clone(), 0, w)
            };

            let via_round_trip =
                <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::left_adjunct(
                    &ctx,
                    a,
                    |la: Chain<f64>| {
                        <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::right_adjunct(
                            &ctx, la, g,
                        )
                    },
                );

            let direct = g(a);
            assert!(
                approx_eq(first_weight(&via_round_trip), first_weight(&direct), TOL),
                "left_adjunct(a, right_adjunct(., g)) != g(a) for n={n}, a={a}, k={k}"
            );
        }
    }
}

#[test]
fn unit_places_the_value_where_counit_looks_for_it() {
    // The two halves have to agree about which position carries the value. Asserting each in
    // isolation, which is what the existing suite does, cannot catch a disagreement.
    let mut rng = LawRng::new(SEED ^ 3);
    for n in [2usize, 5] {
        let ctx = ctx_for(n, 0);
        let a = rng.well_scaled(7.0);
        let outer = <ChainWitness as Adjunction<ChainWitness, ChainWitness, Ctx>>::unit(&ctx, a);

        assert_eq!(
            outer.weights().values().len(),
            1,
            "unit should store exactly one outer entry"
        );
        let inner = &outer.weights().values()[0];
        assert_eq!(
            inner.weights().values().len(),
            1,
            "unit should store exactly one inner entry"
        );
        assert!(
            approx_eq(inner.weights().values()[0], a, TOL),
            "unit lost the value it was given"
        );
    }
}
