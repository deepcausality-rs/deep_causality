/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;

use crate::CausalMultiVector;
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Pure, Traversable};
use deep_causality_metric::Metric;

pub struct CausalMultiVectorWitness;

impl HKT for CausalMultiVectorWitness {
    type Type<T> = CausalMultiVector<T>;
}

// ----------------------------------------------------------------------------
// Functor
// ----------------------------------------------------------------------------
impl Functor<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn fmap<A, B, Func>(fa: CausalMultiVector<A>, f: Func) -> CausalMultiVector<B>
    where
        Func: FnMut(A) -> B,
    {
        let metric = fa.metric;
        let data = fa.data.into_iter().map(f).collect();
        CausalMultiVector { data, metric }
    }
}

// ----------------------------------------------------------------------------
// Pure
// ----------------------------------------------------------------------------
impl Pure<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn pure<T>(value: T) -> CausalMultiVector<T> {
        let metric = Metric::Euclidean(0);
        let data = vec![value];
        CausalMultiVector { data, metric }
    }
}

// ----------------------------------------------------------------------------
// Applicative
// ----------------------------------------------------------------------------
impl Applicative<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn apply<A, B, Func>(
        f_ab: CausalMultiVector<Func>,
        f_a: CausalMultiVector<A>,
    ) -> CausalMultiVector<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let metric = f_a.metric;
        let funcs = f_ab.data;
        let args = f_a.data;

        let data = if funcs.len() == 1 {
            let f = funcs.into_iter().next().unwrap();
            args.into_iter().map(f).collect()
        } else if funcs.len() == args.len() {
            funcs.into_iter().zip(args).map(|(mut f, a)| f(a)).collect()
        } else {
            panic!(
                "Applicative::apply shape mismatch: {} funcs vs {} args",
                funcs.len(),
                args.len()
            );
        };

        CausalMultiVector { data, metric }
    }
}

// ----------------------------------------------------------------------------
// Foldable
// ----------------------------------------------------------------------------
impl Foldable<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn fold<A, B, Func>(fa: CausalMultiVector<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_iter().fold(init, f)
    }
}

// ----------------------------------------------------------------------------
// Traversable
// ----------------------------------------------------------------------------
impl Traversable<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    /// Flips `CausalMultiVector<M<A>>` into `M<CausalMultiVector<A>>`, folding an accumulator
    /// through `M` from left to right so the effects run in blade order and the result keeps that
    /// order.
    ///
    /// An element in a failing state collapses the whole traversal, and the first such element in
    /// blade order is the one reported.
    ///
    /// # The metric survives, and so does the length invariant
    ///
    /// A `CausalMultiVector` holds exactly `2^dim` coefficients, with `dim` coming from its
    /// `Metric`, so the metric is not decoration: it fixes the length. That is what denies this
    /// witness a [`Monad`](deep_causality_haft::Monad) — `bind`'s continuation may return a
    /// different metric than the input carries, and the two identity laws then want opposite
    /// choices.
    ///
    /// `sequence` never faces that choice. It is one-in-one-out by construction, so the input's
    /// metric is the only defensible answer and the coefficient count cannot change. The
    /// invariant `CausalMultiVector::new` enforces therefore holds without being re-checked, which
    /// is why the result is built from the struct fields directly, exactly as `fmap` does.
    ///
    /// The metric is read before `data` is consumed; the borrow checker enforces that ordering
    /// rather than leaving it to a convention.
    ///
    /// # Cost
    ///
    /// The accumulator is cloned once per step and holds `k` elements at step `k`, so the fold
    /// performs `n(n-1)/2` element clones for an `n`-coefficient multivector. The clone is forced
    /// by [`Applicative::apply`]'s `Func: FnMut` bound rather than by this trait's `A: Clone`: an
    /// `FnMut` may be invoked repeatedly, so the closure cannot move its captured accumulator out.
    /// `n` is `2^dim`, so this is bounded by the algebra rather than by the data.
    fn sequence<A, M>(fa: CausalMultiVector<M::Type<A>>) -> M::Type<CausalMultiVector<A>>
    where
        M: Applicative<M> + HKT,
        A: Clone,
    {
        let metric = fa.metric;
        let mut acc: M::Type<Vec<A>> = M::pure(Vec::new());
        for m_a in fa.data {
            acc = M::apply(
                M::fmap(acc, |v: Vec<A>| {
                    move |a: A| {
                        let mut v = v.clone();
                        v.push(a);
                        v
                    }
                }),
                m_a,
            );
        }
        M::fmap(acc, move |data| CausalMultiVector { data, metric })
    }
}

// ----------------------------------------------------------------------------
// Monad: deliberately not implemented
// ----------------------------------------------------------------------------
//
// A `CausalMultiVector` holds exactly `2^dim` coefficients, where `dim` comes from its `Metric`, so
// the metric is not decoration: it fixes the length. `Pure::pure` receives one value and no metric,
// and the only metric it can name without inventing geometry is `Euclidean(0)`, whose algebra
// `Cl(0)` has exactly one coefficient. That value is well formed, which is why `Pure` stays.
//
// `bind` is what cannot be written. It receives a metric from its input and another from every
// `f(a)`, and the two identity laws demand opposite choices:
//
//   left  identity, `bind(pure(a), f) == f(a)`, needs the metric taken from `f`'s result,
//                   because `pure(a)` carries only `Euclidean(0)`;
//   right identity, `bind(m, pure) == m`,       needs the metric taken from the input,
//                   because `pure` supplies only `Euclidean(0)`.
//
// ----------------------------------------------------------------------------
// CoMonad
// ----------------------------------------------------------------------------
impl CoMonad<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn extract<A>(fa: &CausalMultiVector<A>) -> A
    where
        A: Clone,
    {
        // Extract scalar part (index 0)
        fa.data.first().cloned().expect("Empty MultiVector")
    }

    fn extend<A, B, Func>(fa: &CausalMultiVector<A>, mut f: Func) -> CausalMultiVector<B>
    where
        Func: FnMut(&CausalMultiVector<A>) -> B,
        A: Clone,
    {
        // Extend with cyclic rotation.
        // For each position i, construct a view where i is the new origin (0).
        // For a MultiVector, this is "Basis Shifting".
        // view[k] = fa[(i + k) % N]
        let n = fa.data.len();
        let mut result_data = Vec::with_capacity(n);

        for i in 0..n {
            // Create rotated view
            let mut rotated_data = Vec::with_capacity(n);
            for k in 0..n {
                let idx = (i + k) % n;
                rotated_data.push(fa.data[idx].clone());
            }
            let view = CausalMultiVector {
                data: rotated_data,
                metric: fa.metric,
            };

            result_data.push(f(&view));
        }

        CausalMultiVector {
            data: result_data,
            metric: fa.metric,
        }
    }
}

// ----------------------------------------------------------------------------
// Adjunction
// ----------------------------------------------------------------------------
// Context: Metric (defining the space we are adjoint to)
// `Adjunction` is deliberately absent for this witness.
//
// It used to claim `CausalMultiVector` is adjoint to itself, with `unit` broadcasting one value
// across every blade of the context algebra and `counit` taking the first coefficient of the
// first inner multivector. That pair cannot satisfy the defining bijection: `right_adjunct` after
// `left_adjunct` reconstructs a *constant* multivector, so it is the identity only when the input
// already holds the same value in every blade. With `ctx = Euclidean(1)` and `la = [3, 4]` the
// round trip yields 3 where the law requires 4.
//
// No `unit` can do better, because it is handed a single `A` and must fill `2^dim` coefficients,
// and a container functor is not self-adjoint in general. The impl had no production callers and
// no documentation. Removing it follows `7ec185d49`, which deleted the laws-free `GaugeField`
// witness rather than keep a structure whose laws fail.
