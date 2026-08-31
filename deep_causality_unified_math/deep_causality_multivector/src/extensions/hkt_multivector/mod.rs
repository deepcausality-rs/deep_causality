/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;

use crate::CausalMultiVector;
use crate::CausalMultiVectorError;
use deep_causality_haft::{
    Adjunction, Applicative, CoMonad, Foldable, Functor, HKT, NoConstraint, Pure, Satisfies,
};
use deep_causality_metric::Metric;

pub struct CausalMultiVectorWitness;

impl HKT for CausalMultiVectorWitness {
    type Constraint = NoConstraint;
    type Type<T> = CausalMultiVector<T>;
}

// ----------------------------------------------------------------------------
// Functor
// ----------------------------------------------------------------------------
impl Functor<CausalMultiVectorWitness> for CausalMultiVectorWitness {
    fn fmap<A, B, Func>(fa: CausalMultiVector<A>, f: Func) -> CausalMultiVector<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
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
    fn pure<T>(value: T) -> CausalMultiVector<T>
    where
        T: Satisfies<NoConstraint>,
    {
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
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint> + FnMut(A) -> B,
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
        A: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_iter().fold(init, f)
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
        A: Satisfies<NoConstraint> + Clone,
    {
        // Extract scalar part (index 0)
        fa.data.first().cloned().expect("Empty MultiVector")
    }

    fn extend<A, B, Func>(fa: &CausalMultiVector<A>, mut f: Func) -> CausalMultiVector<B>
    where
        Func: FnMut(&CausalMultiVector<A>) -> B,
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
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
impl Adjunction<CausalMultiVectorWitness, CausalMultiVectorWitness, Metric>
    for CausalMultiVectorWitness
{
    type Error = CausalMultiVectorError;

    fn unit<A>(ctx: &Metric, a: A) -> CausalMultiVector<CausalMultiVector<A>>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
    {
        // unit: a -> R(L(a))
        // Inner MV: the context algebra Cl(ctx), which admits exactly 2^dim coefficients, so 'a'
        // fills every blade. The scalar embedding, 'a' at index 0 and zeros elsewhere, would need a
        // `Zero` bound the trait signature does not offer, and a one-coefficient value carrying a
        // higher-dimensional metric is a value `CausalMultiVector::new` rejects.
        let inner_data = vec![a; 1 << ctx.dimension()];
        let inner_mv = CausalMultiVector {
            data: inner_data,
            metric: *ctx,
        };

        // Outer MV: Cl(0), whose single coefficient holds inner_mv.
        let outer_data = vec![inner_mv];
        CausalMultiVector {
            data: outer_data,
            metric: Metric::Euclidean(0),
        }
    }

    /// # Errors
    ///
    /// Returns [`CausalMultiVectorError::empty_multivector`] when the flattened multivector stores
    /// no coefficient, so there is no scalar part to take.
    fn counit<B>(
        _ctx: &Metric,
        lrb: CausalMultiVector<CausalMultiVector<B>>,
    ) -> Result<B, Self::Error>
    where
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
    {
        // counit: L(R(b)) -> b. Flatten the nesting, then take the scalar part.
        lrb.data
            .into_iter()
            .flat_map(|inner| inner.data.into_iter())
            .next()
            .ok_or_else(CausalMultiVectorError::empty_multivector)
    }

    fn left_adjunct<A, B, F>(ctx: &Metric, a: A, f: F) -> CausalMultiVector<B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        F: Fn(CausalMultiVector<A>) -> B,
    {
        // left: a -> f(unit(a))
        let unit_res = Self::unit(ctx, a);
        Self::fmap(unit_res, f)
    }

    /// # Errors
    ///
    /// Returns [`CausalMultiVectorError::empty_multivector`] when `la` stores no coefficient, so
    /// there is no `A` to apply `f` to, or when the multivector `f` returns stores none.
    fn right_adjunct<A, B, F>(
        _ctx: &Metric,
        la: CausalMultiVector<A>,
        f: F,
    ) -> Result<B, Self::Error>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        F: FnMut(A) -> CausalMultiVector<B>,
    {
        // right: (A -> R<B>) -> (L<A> -> B)
        // map la with f -> L<R<B>> (MV<MV<B>>), then extract by hand so this does not inherit
        // counit's `Clone` bound on B.
        Self::fmap(la, f)
            .data
            .into_iter()
            .next()
            .and_then(|inner_mv| inner_mv.data.into_iter().next())
            .ok_or_else(CausalMultiVectorError::empty_multivector)
    }
}
