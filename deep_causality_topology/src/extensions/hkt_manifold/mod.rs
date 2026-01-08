/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Manifold;
use deep_causality_haft::{
    Applicative, CoMonad, Foldable, Functor, HKT, Monad, NoConstraint, Pure, Satisfies,
};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use std::marker::PhantomData;

// ============================================================================
// PART 1: Free (Unbounded) Witness - "ManifoldWitness"
// Use Case: General computation, chaining, dynamic pipelines.
// ============================================================================

pub struct ManifoldWitness<C>(PhantomData<C>);

impl<C> HKT for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint>,
{
    type Constraint = NoConstraint;
    type Type<T>
        = Manifold<C, T>
    where
        T: Satisfies<NoConstraint>;
}

impl<C> Functor<ManifoldWitness<C>> for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint> + Clone,
{
    fn fmap<A, B, Func>(m_a: Manifold<C, A>, f: Func) -> Manifold<C, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        // 1. Map Data
        // Capture f in a closure for data mapping
        let new_data_tensor = CausalTensorWitness::fmap(m_a.data, f);

        // 2. Complex and Metric are INVARIANT because they depend on C, not A or B.
        // We can just clone them.
        Manifold {
            complex: m_a.complex.clone(),
            data: new_data_tensor,
            metric: m_a.metric.clone(),
            cursor: m_a.cursor,
        }
    }
}

impl<C> Foldable<ManifoldWitness<C>> for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint>,
{
    fn fold<A, B, Func>(fa: Manifold<C, A>, init: B, f: Func) -> B
    where
        A: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_vec().into_iter().fold(init, f)
    }
}

impl<C> Pure<ManifoldWitness<C>> for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint> + Default,
{
    fn pure<T>(value: T) -> Manifold<C, T>
    where
        T: Satisfies<NoConstraint>,
    {
        let tensor = CausalTensor::from_vec(vec![value], &[1]);
        Manifold {
            complex: Default::default(),
            data: tensor,
            metric: None,
            cursor: 0,
        }
    }
}

impl<C> Monad<ManifoldWitness<C>> for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint> + Clone + Default,
{
    fn bind<A, B, Func>(m_a: Manifold<C, A>, mut f: Func) -> Manifold<C, B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> <Self as HKT>::Type<B>,
    {
        let mut result_data = Vec::with_capacity(m_a.data.len());

        for a in m_a.data.into_vec() {
            let mb = f(a);
            result_data.extend(mb.data.into_vec());
        }

        let new_len = result_data.len();
        let new_tensor = CausalTensor::from_vec(result_data, &[new_len]);

        // We clone the input structure.
        Manifold {
            complex: m_a.complex.clone(),
            data: new_tensor,
            metric: m_a.metric.clone(),
            cursor: 0,
        }
    }
}

impl<C> Applicative<ManifoldWitness<C>> for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint> + Clone + Default,
{
    fn apply<A, B, Func>(f_ab: Manifold<C, Func>, f_a: Manifold<C, A>) -> Manifold<C, B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint> + FnMut(A) -> B,
    {
        let shape = f_a.data.shape().to_vec();
        let funcs = f_ab.data.into_vec();
        let args = f_a.data.into_vec();

        let new_data: Vec<B> = if funcs.len() == 1 {
            let f = funcs.into_iter().next().unwrap();
            args.into_iter().map(f).collect()
        } else {
            funcs.into_iter().zip(args).map(|(mut f, a)| f(a)).collect()
        };

        let new_tensor = CausalTensor::from_vec(new_data, &shape);

        // Preserve topology from A
        Manifold {
            complex: f_a.complex.clone(),
            data: new_tensor,
            metric: f_a.metric.clone(),
            cursor: 0,
        }
    }
}

impl<C> CoMonad<ManifoldWitness<C>> for ManifoldWitness<C>
where
    C: Satisfies<NoConstraint> + Clone,
{
    fn extract<A>(fa: &Manifold<C, A>) -> A
    where
        A: Satisfies<NoConstraint> + Clone,
    {
        if fa.data.is_empty() {
            panic!("Cannot extract from empty Manifold");
        }
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor out of bounds")
    }

    fn extend<A, B, Func>(fa: &Manifold<C, A>, mut f: Func) -> Manifold<C, B>
    where
        Func: FnMut(&Manifold<C, A>) -> B,
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
    {
        let len = fa.data.len();
        let shape = fa.data.shape().to_vec();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let mut view = fa.clone_shallow();
                view.cursor = i;
                f(&view)
            })
            .collect();

        let new_tensor = CausalTensor::from_vec(new_data, &shape);

        // Preserve topology and metric from A!
        Manifold {
            complex: fa.complex.clone(),
            data: new_tensor,
            metric: fa.metric.clone(),
            cursor: fa.cursor, // Maintain orig cursor? Or reset?
        }
    }
}
