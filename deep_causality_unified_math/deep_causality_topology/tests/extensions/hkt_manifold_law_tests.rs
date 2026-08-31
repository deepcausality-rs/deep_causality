/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Categorical laws for `ManifoldWitness`, the crate's only `Monad` and `Applicative`.
//!
//! Every case is generated. `Manifold` derives `PartialEq`, so the laws are asserted against the
//! whole structure rather than against the data tensor alone: a law that holds for the payload and
//! drops the complex or the metric is not a law that holds. The focus is part of that structure
//! everywhere except in `applicative_identity`, which says at the assertion why it compares the
//! focus separately.

use deep_causality_haft::{Applicative, CoMonad, Functor, Monad, Pure};
use deep_causality_topology::utils_tests::{LawRng, approx_eq_slice, manifold_cases};
use deep_causality_topology::{Manifold, ManifoldWitness, SimplicialComplex};

type M = Manifold<SimplicialComplex<f64>, f64>;
type W = ManifoldWitness<f64>;

const TOL: f64 = 1e-9;
const SEED: u64 = 0x5EED_1A77;

#[test]
fn functor_identity() {
    for case in manifold_cases(SEED) {
        let mapped = <W as Functor<W>>::fmap(case.value.clone(), |x| x);
        assert_eq!(
            mapped, case.value,
            "fmap(id, m) != m for {}: the functor identity law",
            case.label
        );
    }
}

#[test]
fn functor_composition() {
    // `well_scaled`, not `scalar`: the comparison below is numeric, and `scalar` injects `1e12`
    // and subnormals that would make it measure float conditioning instead of the composition.
    let mut rng = LawRng::new(SEED ^ 1);
    for case in manifold_cases(SEED ^ 1) {
        let (p, q) = (rng.well_scaled(4.0), rng.well_scaled(4.0));
        let f = move |x: f64| x * p;
        let g = move |x: f64| x + q;

        let lhs = <W as Functor<W>>::fmap(<W as Functor<W>>::fmap(case.value.clone(), f), g);
        let rhs = <W as Functor<W>>::fmap(case.value.clone(), move |x| g(f(x)));

        assert!(
            approx_eq_slice(lhs.data().as_slice(), rhs.data().as_slice(), TOL),
            "fmap(g) . fmap(f) != fmap(g . f) for {} with p={p} q={q}",
            case.label
        );
        assert_eq!(
            lhs.cursor(),
            rhs.cursor(),
            "composition changed the focus for {}",
            case.label
        );
    }
}

#[test]
fn monad_left_identity() {
    let mut rng = LawRng::new(SEED ^ 2);
    for _ in 0..64 {
        let a = rng.scalar(9.0);
        let k = rng.scalar(3.0);
        let f = move |x: f64| <W as Pure<W>>::pure(x * k);

        let lhs = <W as Monad<W>>::bind(<W as Pure<W>>::pure(a), f);
        let rhs = f(a);

        assert!(
            approx_eq_slice(lhs.data().as_slice(), rhs.data().as_slice(), TOL),
            "bind(pure(a), f) != f(a) for a={a} k={k}"
        );
    }
}

#[test]
fn monad_right_identity() {
    // bind(m, pure) == m. Swept over every legal focus, because the focus is part of `m`.
    for case in manifold_cases(SEED ^ 3) {
        let rt = <W as Monad<W>>::bind(case.value.clone(), <W as Pure<W>>::pure);
        assert_eq!(
            rt, case.value,
            "bind(m, pure) != m for {}: the monad right identity law",
            case.label
        );
    }
}

#[test]
fn monad_associativity() {
    // Numeric law, so the coefficients come from `well_scaled` for the reason given above.
    let mut rng = LawRng::new(SEED ^ 4);
    for case in manifold_cases(SEED ^ 4) {
        let (p, q) = (rng.well_scaled(3.0), rng.well_scaled(3.0));
        let f = move |x: f64| <W as Pure<W>>::pure(x * p);
        let g = move |x: f64| <W as Pure<W>>::pure(x + q);

        let lhs = <W as Monad<W>>::bind(<W as Monad<W>>::bind(case.value.clone(), f), g);
        let rhs =
            <W as Monad<W>>::bind(case.value.clone(), move |x| <W as Monad<W>>::bind(f(x), g));

        assert!(
            approx_eq_slice(lhs.data().as_slice(), rhs.data().as_slice(), TOL),
            "bind(bind(m,f),g) != bind(m, |x| bind(f(x),g)) for {}",
            case.label
        );
    }
}

#[test]
fn applicative_identity() {
    // apply(pure(id), v) == v. Asserted on the whole manifold, so an `apply` that rebuilt the
    // complex or dropped the metric fails here rather than passing a payload-only comparison.
    //
    // The focus is the one field that does not round-trip, and the exclusion is pinned rather
    // than ignored. `apply` reads the complex and the metric from the argument but always builds
    // its result at cursor 0, where `bind` and `extend` thread the input's focus through. Taking
    // the focus from `out` would make this test blind to that field entirely; asserting it equals
    // 0 means a change to `apply`'s focus handling fails here and has to be argued for.
    for case in manifold_cases(SEED ^ 5) {
        let idf: fn(f64) -> f64 = |x| x;
        let out = <W as Applicative<W>>::apply(<W as Pure<W>>::pure(idf), case.value.clone());

        assert_eq!(
            out.cursor(),
            0,
            "apply builds at cursor 0, unlike bind and extend, for {}",
            case.label
        );

        let expected: M = Manifold::new(case.value.complex().clone(), case.value.data().clone(), 0)
            .expect("a generated case rebuilt at cursor 0 is still well formed");

        assert_eq!(out, expected, "apply(pure(id), v) != v for {}", case.label);
    }
}

#[test]
fn applicative_homomorphism() {
    // apply(pure(f), pure(x)) == pure(f(x))
    let mut rng = LawRng::new(SEED ^ 6);
    for _ in 0..64 {
        let x = rng.scalar(9.0);
        let f: fn(f64) -> f64 = |v| v * 2.0 + 1.0;

        let lhs = <W as Applicative<W>>::apply(<W as Pure<W>>::pure(f), <W as Pure<W>>::pure(x));
        let rhs = <W as Pure<W>>::pure(f(x));

        assert!(
            approx_eq_slice(lhs.data().as_slice(), rhs.data().as_slice(), TOL),
            "apply(pure(f), pure(x)) != pure(f(x)) for x={x}"
        );
    }
}

#[test]
fn comonad_left_identity() {
    // extend(w, extract) == w
    for case in manifold_cases(SEED ^ 7) {
        let out = <W as CoMonad<W>>::extend(&case.value, |v: &M| <W as CoMonad<W>>::extract(v));
        assert_eq!(
            out, case.value,
            "extend(w, extract) != w for {}: the comonad left identity law",
            case.label
        );
    }
}

#[test]
fn comonad_right_identity() {
    // extract(extend(w, f)) == f(w), which only has content when the focus is preserved.
    for case in manifold_cases(SEED ^ 8) {
        let f = |v: &M| v.data().as_slice()[v.cursor()] * 3.0 + 1.0;
        let extended = <W as CoMonad<W>>::extend(&case.value, f);
        let lhs = <W as CoMonad<W>>::extract(&extended);
        let rhs = f(&case.value);
        assert!(
            approx_eq_slice(&[lhs], &[rhs], TOL),
            "extract(extend(w, f)) != f(w) for {}: got {lhs}, want {rhs}",
            case.label
        );
    }
}

#[test]
fn comonad_associativity() {
    for case in manifold_cases(SEED ^ 9) {
        let g = |v: &M| v.data().as_slice()[v.cursor()] + 1.0;
        let f = |v: &M| v.data().as_slice()[v.cursor()] * 10.0;

        let lhs = <W as CoMonad<W>>::extend(&<W as CoMonad<W>>::extend(&case.value, g), f);
        let rhs =
            <W as CoMonad<W>>::extend(&case.value, |vp: &M| f(&<W as CoMonad<W>>::extend(vp, g)));

        assert!(
            approx_eq_slice(lhs.data().as_slice(), rhs.data().as_slice(), TOL),
            "extend(extend(w,g),f) != extend(w, |w'| f(extend(w',g))) for {}",
            case.label
        );
    }
}
