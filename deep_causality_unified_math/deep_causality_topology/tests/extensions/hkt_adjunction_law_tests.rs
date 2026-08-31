/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Adjunction laws for `ChainWitness<f64>`.
//!
//! The crate has two `Adjunction` implementations and 38 tests across them, every one of which
//! checks `unit`, `counit`, `left_adjunct` or `right_adjunct` in isolation on a single fixture.
//! What makes an adjunction an adjunction is that the two adjuncts are mutually inverse, and that
//! was asserted nowhere. These are those laws, over generated chains.

use deep_causality_haft::Adjunction;
use deep_causality_linear::CsrMatrix;
use deep_causality_topology::utils_tests::{LawRng, approx_eq, chain_cases, path_complex};
use deep_causality_topology::{Chain, ChainWitness, SimplicialComplex};
use std::sync::Arc;

type Ctx = (Arc<SimplicialComplex<f64>>, usize);
const TOL: f64 = 1e-9;
const SEED: u64 = 0xAD_0F1;

fn ctx_for(n: usize, grade: usize) -> Ctx {
    (Arc::new(path_complex::<f64>(n)), grade)
}

/// The first stored weight, which is the value `counit` and `right_adjunct` both select.
fn first_weight(c: &Chain<f64, f64>) -> f64 {
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
                let wrapped = <ChainWitness<f64> as Adjunction<
                    ChainWitness<f64>,
                    ChainWitness<f64>,
                    Ctx,
                >>::unit(&ctx, a);
                let back = <ChainWitness<f64> as Adjunction<
                    ChainWitness<f64>,
                    ChainWitness<f64>,
                    Ctx,
                >>::counit(&ctx, wrapped)
                .expect("unit builds a chain that stores a value, so counit finds one");
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
    //
    // What the round trip pins down is the first stored weight, and only that. `right_adjunct`
    // selects the first entry of the chain it is handed, and `left_adjunct` runs `f` over a chain
    // `unit` built from a single value, so `f` has to be a function of that one weight for the two
    // sides to be comparable at all. The sweep varies vertex count, grade and sparsity around that
    // entry: it shows the selection stays on the first weight as the shape changes, not that the
    // law holds for an `f` reading the rest of the chain. The context is the case's own complex
    // and grade, so the two adjuncts and the chain agree about which complex they are over.
    for case in chain_cases(SEED ^ 1) {
        let ctx: Ctx = (Arc::clone(case.value.complex()), case.value.grade());
        let f = |c: Chain<f64, f64>| first_weight(&c) * 2.0 + 1.0;

        let round_trip = <ChainWitness<f64> as Adjunction<
            ChainWitness<f64>,
            ChainWitness<f64>,
            Ctx,
        >>::right_adjunct(&ctx, case.value.clone(), |a: f64| {
            <ChainWitness<f64> as Adjunction<ChainWitness<f64>, ChainWitness<f64>, Ctx>>::left_adjunct(
                        &ctx, a, f,
                    )
        })
        .expect("the generated chain stores a value on both sides of the round trip");

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

            // A plain `A -> Chain<B, B>`: the one-entry chain holding `x * k`.
            let g = |x: f64| -> Chain<f64, f64> {
                let w = CsrMatrix::from_triplets(1, n, &[(0, 0, x * k)])
                    .expect("single-entry chain weights");
                Chain::new(complex.clone(), 0, w)
            };

            let via_round_trip = <ChainWitness<f64> as Adjunction<
                ChainWitness<f64>,
                ChainWitness<f64>,
                Ctx,
            >>::left_adjunct(&ctx, a, |la: Chain<f64, f64>| {
                <ChainWitness<f64> as Adjunction<ChainWitness<f64>, ChainWitness<f64>, Ctx>>::right_adjunct(
                            &ctx, la, g,
                        )
                        .expect("g builds a one-entry chain, so there is always a B")
            });

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
        let outer =
            <ChainWitness<f64> as Adjunction<ChainWitness<f64>, ChainWitness<f64>, Ctx>>::unit(
                &ctx, a,
            );

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

// ----------------------------------------------------------------------------
// Functor identity on a complex that carries geometry
// ----------------------------------------------------------------------------

/// A line complex built from real coordinates, so its Hodge ⋆ operators are available.
fn geometric_line() -> SimplicialComplex<f64> {
    use deep_causality_topology::{Simplex, Skeleton};
    let v = Skeleton::new(0, vec![Simplex::new(vec![0]), Simplex::new(vec![1])]);
    let e = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let d1 = CsrMatrix::from_triplets(2, 1, &[(0, 0, -1i8), (1, 0, 1i8)]).expect("boundary");
    SimplicialComplex::with_geometry(vec![v, e], vec![d1], vec![], vec![0.0, 0.0, 1.0, 0.0], 2)
}

#[test]
fn fmap_preserves_the_complex_geometry() {
    // `Chain` used to hold one parameter for both the complex's precision and the coefficient
    // group, so `fmap` had to rebuild the complex and dropped its Hodge ⋆ operators. With the two
    // separated the complex is carried across, and this is the regression guard for that.
    use deep_causality_haft::Functor;

    let source = geometric_line();
    assert!(
        source.hodge_star_operators().is_ok(),
        "fixture is wrong: the source complex must carry geometry, or this test proves nothing"
    );

    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1.5f64), (0, 1, 2.5)]).expect("weights");
    let chain: Chain<f64, f64> = Chain::new(Arc::new(source), 0, weights);

    let mapped = <ChainWitness<f64> as Functor<ChainWitness<f64>>>::fmap(chain, |x| x * 2.0);
    assert!(
        mapped.complex().hodge_star_operators().is_ok(),
        "fmap dropped the Hodge star operators"
    );
}

#[test]
fn functor_identity_holds_on_a_geometric_complex() {
    // The law the geometry drop broke: `fmap(id, c) == c`, on a complex carrying a metric.
    use deep_causality_haft::Functor;

    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 3.0f64), (0, 1, 4.0)]).expect("weights");
    let chain: Chain<f64, f64> = Chain::new(Arc::new(geometric_line()), 0, weights);

    let mapped = <ChainWitness<f64> as Functor<ChainWitness<f64>>>::fmap(chain.clone(), |x| x);
    assert_eq!(
        mapped, chain,
        "fmap(id, c) != c on a geometry-carrying complex"
    );
}

#[test]
fn mapping_coefficients_leaves_the_precision_alone() {
    // The type-level statement of the same fact: mapping f64 coefficients to i32 yields
    // `Chain<f64, i32>`, not `Chain<i32, i32>`. The complex keeps its precision.
    use deep_causality_haft::Functor;

    let weights = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1.9f64), (0, 1, 2.9)]).expect("weights");
    let chain: Chain<f64, f64> = Chain::new(Arc::new(geometric_line()), 0, weights);

    let ints: Chain<f64, i32> =
        <ChainWitness<f64> as Functor<ChainWitness<f64>>>::fmap(chain, |x| x as i32);
    assert_eq!(ints.weights().values(), &vec![1, 2]);
    assert!(
        ints.complex().hodge_star_operators().is_ok(),
        "changing the coefficient type must not disturb the complex"
    );
}

// ---------------------------------------------------------------------------
// The partial operations report rather than panic
// ---------------------------------------------------------------------------

/// An empty chain is reachable input, not a corner case: CSR drops explicit zeros, so a chain
/// whose weights are all zero stores nothing. Before `Adjunction::Error` existed, both of these
/// panicked.
#[test]
fn right_adjunct_reports_an_empty_input_chain() {
    let complex = Arc::new(path_complex::<f64>(3));
    let ctx: Ctx = (complex.clone(), 0);

    // All-zero triplets: CSR stores no explicit entry, so the chain is empty.
    let empty = Chain::new(
        complex.clone(),
        0,
        CsrMatrix::<f64>::from_triplets(1, 3, &[(0, 0, 0.0)]).expect("zero-weight chain"),
    );

    let result =
        <ChainWitness<f64> as Adjunction<ChainWitness<f64>, ChainWitness<f64>, Ctx>>::right_adjunct(
            &ctx,
            empty,
            |a: f64| {
                let w = CsrMatrix::from_triplets(1, 3, &[(0, 0, a)]).expect("one-entry chain");
                Chain::new(complex.clone(), 0, w)
            },
        );

    let err = result.expect_err("an empty input chain has no A to apply f to");
    assert!(
        err.to_string().contains("no A to apply f to"),
        "the error should name what was missing, got: {err}"
    );
}

/// The other half: the input is fine, but `f` returns a chain that stores nothing.
#[test]
fn right_adjunct_reports_an_empty_output_chain() {
    let complex = Arc::new(path_complex::<f64>(3));
    let ctx: Ctx = (complex.clone(), 0);

    let la = Chain::new(
        complex.clone(),
        0,
        CsrMatrix::from_triplets(1, 3, &[(0, 0, 2.5)]).expect("one-entry chain"),
    );

    let result =
        <ChainWitness<f64> as Adjunction<ChainWitness<f64>, ChainWitness<f64>, Ctx>>::right_adjunct(
            &ctx,
            la,
            |_a: f64| Chain::new(complex.clone(), 0, CsrMatrix::<f64>::new()),
        );

    let err = result.expect_err("an empty output chain has no B to return");
    assert!(
        err.to_string().contains("no B to return"),
        "the error should name what was missing, got: {err}"
    );
}

/// `counit` is partial for the same reason and now reports the same way.
#[test]
fn counit_reports_an_empty_outer_chain() {
    let complex = Arc::new(path_complex::<f64>(3));
    let ctx: Ctx = (complex.clone(), 0);

    let empty_outer: Chain<f64, Chain<f64, f64>> = Chain::new(complex.clone(), 0, CsrMatrix::new());

    let result =
        <ChainWitness<f64> as Adjunction<ChainWitness<f64>, ChainWitness<f64>, Ctx>>::counit(
            &ctx,
            empty_outer,
        );

    let err = result.expect_err("an empty outer chain has no inner chain to descend into");
    assert!(
        err.to_string().contains("no inner chain"),
        "the error should name what was missing, got: {err}"
    );
}
