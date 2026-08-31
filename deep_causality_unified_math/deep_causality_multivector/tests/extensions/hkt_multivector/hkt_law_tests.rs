/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Categorical laws for `CausalMultiVectorWitness`, and the reason it has no `Monad`.
//!
//! A `CausalMultiVector` holds exactly `2^dim` coefficients, with `dim` coming from its `Metric`,
//! so the metric fixes the length. The witness used to implement `Monad`, and `bind(m, pure)` turned
//! `Minkowski(4)` into `Euclidean(0)` while keeping 16 coefficients, a value
//! `CausalMultiVector::new` rejects. These tests pin what the remaining instances guarantee.

use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Pure};
use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness as W};

/// A deterministic generator, so a counterexample is reproducible from its seed.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn scalar(&mut self, mag: f64) -> f64 {
        (self.next() % 1_000_001) as f64 / 1_000_000.0 * 2.0 * mag - mag
    }
}

/// The metrics under test, each paired with the coefficient count its algebra admits.
fn metrics() -> Vec<(Metric, usize)> {
    vec![
        (Metric::Euclidean(0), 1),
        (Metric::Euclidean(1), 2),
        (Metric::Euclidean(2), 4),
        (Metric::Euclidean(3), 8),
        (Metric::Minkowski(4), 16),
    ]
}

fn sample(metric: Metric, len: usize, rng: &mut Rng) -> CausalMultiVector<f64> {
    let data: Vec<f64> = (0..len).map(|_| rng.scalar(5.0)).collect();
    CausalMultiVector::new(data, metric).expect("generator must build a well-formed multivector")
}

fn close(a: &CausalMultiVector<f64>, b: &CausalMultiVector<f64>) -> bool {
    a.metric() == b.metric()
        && a.data().len() == b.data().len()
        && a.data()
            .iter()
            .zip(b.data())
            .all(|(x, y)| (x - y).abs() < 1e-9)
}

#[test]
fn functor_identity_preserves_the_metric() {
    let mut rng = Rng::new(0x11);
    for (metric, len) in metrics() {
        let v = sample(metric, len, &mut rng);
        let mapped = <W as Functor<W>>::fmap(v.clone(), |x| x);
        assert!(
            close(&mapped, &v),
            "fmap(id, v) != v for {metric:?}: metric {:?} -> {:?}",
            v.metric(),
            mapped.metric()
        );
    }
}

#[test]
fn functor_composition() {
    let mut rng = Rng::new(0x22);
    for (metric, len) in metrics() {
        let v = sample(metric, len, &mut rng);
        let (p, q) = (rng.scalar(3.0), rng.scalar(3.0));
        let f = move |x: f64| x * p;
        let g = move |x: f64| x + q;

        let lhs = <W as Functor<W>>::fmap(<W as Functor<W>>::fmap(v.clone(), f), g);
        let rhs = <W as Functor<W>>::fmap(v, move |x| g(f(x)));
        assert!(
            close(&lhs, &rhs),
            "fmap(g) . fmap(f) != fmap(g . f) for {metric:?}"
        );
    }
}

#[test]
fn fmap_result_is_a_well_formed_multivector() {
    // The invariant `CausalMultiVector::new` enforces: 2^dim coefficients. `bind` used to break it.
    let mut rng = Rng::new(0x33);
    for (metric, len) in metrics() {
        let v = sample(metric, len, &mut rng);
        let mapped = <W as Functor<W>>::fmap(v, |x| x * 2.0);
        // The metric must come from the INPUT. Re-validating the output against its own metric
        // asks whether the value is consistent with itself, which it is by construction: an
        // `fmap` that halved the coefficient count *and* rewrote the metric to match would pass.
        assert_eq!(
            mapped.metric(),
            metric,
            "fmap changed the algebra for {metric:?}"
        );
        assert_eq!(
            mapped.data().len(),
            len,
            "fmap changed the coefficient count for {metric:?}"
        );
        assert!(
            CausalMultiVector::new(mapped.data().clone(), metric).is_ok(),
            "fmap produced a multivector the constructor rejects for {metric:?}"
        );
    }
}

#[test]
fn pure_builds_the_scalar_algebra_and_is_well_formed() {
    // `pure` names the one metric it can without inventing geometry: Cl(0), whose algebra has a
    // single coefficient. That value is well formed, which is why `Pure` survives while `Monad`
    // does not.
    let p = <W as Pure<W>>::pure(7.0f64);
    assert_eq!(p.metric().dimension(), 0);
    assert_eq!(p.data().len(), 1);
    assert!(CausalMultiVector::new(p.data().clone(), p.metric()).is_ok());
}

#[test]
fn applicative_identity() {
    // apply(pure(id), v) == v. `apply` broadcasts a single function and takes the metric from the
    // argument, so the law holds even though `pure` carries Cl(0).
    let mut rng = Rng::new(0x44);
    for (metric, len) in metrics() {
        let v = sample(metric, len, &mut rng);
        let idf: fn(f64) -> f64 = |x| x;
        let out = <W as Applicative<W>>::apply(<W as Pure<W>>::pure(idf), v.clone());
        assert!(close(&out, &v), "apply(pure(id), v) != v for {metric:?}");
    }
}

#[test]
fn applicative_homomorphism() {
    // apply(pure(f), pure(x)) == pure(f(x))
    let mut rng = Rng::new(0x55);
    for _ in 0..32 {
        let x = rng.scalar(9.0);
        let f: fn(f64) -> f64 = |v| v * 3.0 - 1.0;
        let lhs = <W as Applicative<W>>::apply(<W as Pure<W>>::pure(f), <W as Pure<W>>::pure(x));
        let rhs = <W as Pure<W>>::pure(f(x));
        assert!(
            close(&lhs, &rhs),
            "apply(pure(f), pure(x)) != pure(f(x)) for x={x}"
        );
    }
}

#[test]
fn comonad_left_identity() {
    // extend(w, extract) == w
    let mut rng = Rng::new(0x66);
    for (metric, len) in metrics() {
        let v = sample(metric, len, &mut rng);
        let out = <W as CoMonad<W>>::extend(&v, |w: &CausalMultiVector<f64>| {
            <W as CoMonad<W>>::extract(w)
        });
        assert!(close(&out, &v), "extend(w, extract) != w for {metric:?}");
    }
}

#[test]
fn foldable_visits_every_coefficient() {
    let mut rng = Rng::new(0x77);
    for (metric, len) in metrics() {
        let v = sample(metric, len, &mut rng);
        // A sum is invariant under reordering and under visiting a coefficient twice while
        // skipping another of equal value. Record the traversal instead.
        let expected: Vec<f64> = v.data().clone();
        let visited = <W as Foldable<W>>::fold(v, Vec::new(), |mut acc, x| {
            acc.push(x);
            acc
        });
        assert_eq!(
            visited, expected,
            "fold visited the coefficients out of order for {metric:?}"
        );
    }
}
