/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Manifold;
use deep_causality_haft::{
    Applicative, CoMonad, Foldable, Functor, HKT, Monad, NoConstraint, Satisfies,
};
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;

// ============================================================================
// PART 1: Free (Unbounded) Witness - "ManifoldWitness"
// Use Case: General computation, chaining, dynamic pipelines.
// ============================================================================

pub struct ManifoldWitness;

impl HKT for ManifoldWitness {
    type Constraint = NoConstraint;
    type Type<T>
        = Manifold<T>
    where
        T: Satisfies<NoConstraint>;
}

// --- Algebraic Implementations (Free) ---

impl Functor<ManifoldWitness> for ManifoldWitness {
    fn fmap<A, B, Func>(m_a: Manifold<A>, f: Func) -> Manifold<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        let shape = m_a.data.shape().to_vec();
        // Convert to vec to iterate (BackendTensor !: IntoIterator)
        let new_data = m_a.data.into_vec().into_iter().map(f).collect::<Vec<B>>();
        let new_tensor = CausalTensor::from_vec(new_data, &shape);

        Manifold {
            complex: m_a.complex,
            data: new_tensor,
            metric: m_a.metric,
            cursor: m_a.cursor,
        }
    }
}

impl Foldable<ManifoldWitness> for ManifoldWitness {
    fn fold<A, B, Func>(fa: Manifold<A>, init: B, f: Func) -> B
    where
        A: Satisfies<NoConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_vec().into_iter().fold(init, f)
    }
}

impl Monad<ManifoldWitness> for ManifoldWitness {
    fn bind<A, B, Func>(m_a: Manifold<A>, mut f: Func) -> Manifold<B>
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

        Manifold {
            complex: m_a.complex,
            data: new_tensor,
            metric: m_a.metric,
            cursor: 0,
        }
    }
}

impl Applicative<ManifoldWitness> for ManifoldWitness {
    fn pure<T>(value: T) -> Manifold<T>
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

    fn apply<A, B, Func>(f_ab: Manifold<Func>, f_a: Manifold<A>) -> Manifold<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
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

        Manifold {
            complex: f_a.complex,
            data: new_tensor,
            metric: f_a.metric,
            cursor: 0,
        }
    }
}

impl CoMonad<ManifoldWitness> for ManifoldWitness {
    fn extract<A>(fa: &Manifold<A>) -> A
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

    fn extend<A, B, Func>(fa: &Manifold<A>, mut f: Func) -> Manifold<B>
    where
        Func: FnMut(&Manifold<A>) -> B,
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
    {
        let len = fa.data.len();
        let shape = fa.data.shape().to_vec();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let mut view = fa.clone();
                view.cursor = i;
                f(&view)
            })
            .collect();

        let new_tensor = CausalTensor::from_vec(new_data, &shape);
        Manifold {
            complex: fa.complex.clone(),
            data: new_tensor,
            metric: fa.metric.clone(),
            cursor: fa.cursor, // Maintain orig cursor? Or reset?
        }
    }
}

// ============================================================================
// PART 2: Strict (Bounded) Witness - "StrictManifoldWitness"
// Use Case: Verified Storage, optimized physics types.
// ============================================================================

#[allow(dead_code)]
pub struct StrictManifoldWitness;

pub struct ManifoldConstraint;

// Allowed Types for Strict Manifold
impl Satisfies<ManifoldConstraint> for f32 {}
impl Satisfies<ManifoldConstraint> for f64 {}
impl Satisfies<ManifoldConstraint> for Complex<f32> {}
impl Satisfies<ManifoldConstraint> for Complex<f64> {}
impl Satisfies<ManifoldConstraint> for i32 {}
impl Satisfies<ManifoldConstraint> for i64 {}
impl Satisfies<ManifoldConstraint> for usize {}
impl<T> Satisfies<ManifoldConstraint> for CausalTensor<T> {}

impl HKT for StrictManifoldWitness {
    type Constraint = ManifoldConstraint;
    type Type<T>
        = Manifold<T>
    where
        T: Satisfies<ManifoldConstraint>;
}

impl Functor<StrictManifoldWitness> for StrictManifoldWitness {
    fn fmap<A, B, Func>(m_a: Manifold<A>, f: Func) -> Manifold<B>
    where
        A: Satisfies<ManifoldConstraint>,
        B: Satisfies<ManifoldConstraint>,
        Func: FnMut(A) -> B,
    {
        let shape = m_a.data.shape().to_vec();
        let new_data = m_a.data.into_vec().into_iter().map(f).collect::<Vec<B>>();
        let new_tensor = CausalTensor::from_vec(new_data, &shape);

        Manifold {
            complex: m_a.complex,
            data: new_tensor,
            metric: m_a.metric,
            cursor: m_a.cursor,
        }
    }
}

impl Foldable<StrictManifoldWitness> for StrictManifoldWitness {
    fn fold<A, B, Func>(fa: Manifold<A>, init: B, f: Func) -> B
    where
        A: Satisfies<ManifoldConstraint>,
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_vec().into_iter().fold(init, f)
    }
}

// CoMonad omitted for Strict Mode due to trait bound limitations.
