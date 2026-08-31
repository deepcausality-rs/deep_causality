/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;

use crate::CausalTensor;
use crate::traits::tensor::Tensor;
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, HKT, Monad, Pure};

// ============================================================================
// HKT Witness Implementation
// ============================================================================

/// HKT witness for [`CausalTensor`].
///
/// # Why `NoConstraint`
///
/// `CausalTensor<T>` declares no bound on `T`, and the categorical operations implemented here
/// do no arithmetic: `fmap`, `fold`, `pure`, `bind`, `extend` and `apply` move elements without
/// computing on them. `fmap` maps `CausalTensor<A>` to `CausalTensor<B>` for unrelated `A` and
/// `B`, so a tensor of labels maps as readily as a tensor of `f64`. `NoConstraint` states that
/// accurately; it is not a placeholder for a bound that belongs here.
///
/// The tensor operations that *do* compute carry their bounds on the impls that need them:
/// `ConjugateScalar` for complex-aware algebra, `RealField + Zero + One + Sum + FromPrimitive`
/// for statistics and reductions, `Clone` for reshaping. Those name real traits, so the compiler
/// enforces them and no downstream crate can satisfy them by declaration.
///
/// See `openspec/notes/archive/hkt_gat/hkt_CausalTensor.md` for the measurement behind this.
pub struct CausalTensorWitness;

impl HKT for CausalTensorWitness {
    type Type<T> = CausalTensor<T>;
}

// ============================================================================
// Algebraic Implementations
// ============================================================================

impl Functor<CausalTensorWitness> for CausalTensorWitness {
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec();
        let new_data: Vec<B> = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_vec().into_iter().fold(init, f)
    }
}

impl Pure<CausalTensorWitness> for CausalTensorWitness {
    fn pure<T>(value: T) -> CausalTensor<T> {
        CausalTensor::from_vec(vec![value], &[])
    }
}

impl Monad<CausalTensorWitness> for CausalTensorWitness {
    fn bind<A, B, Func>(m_a: CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> <Self as HKT>::Type<B>,
    {
        let mut result_data = Vec::with_capacity(m_a.len());
        for a in m_a.into_vec() {
            let mb = f(a);
            result_data.extend(mb.into_vec());
        }
        let len = result_data.len();
        CausalTensor::from_vec(result_data, &[len])
    }
}

impl CoMonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Clone,
    {
        fa.as_slice()
            .first()
            .cloned()
            .expect("CoMonad::extract cannot be called on an empty CausalTensor.")
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(&CausalTensor<A>) -> B,
        A: Clone,
    {
        let len = fa.len();
        let shape = fa.shape().to_vec();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let view = fa.shifted_view(i);
                f(&view)
            })
            .collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

impl Applicative<CausalTensorWitness> for CausalTensorWitness {
    fn apply<A, B, Func>(f_ab: CausalTensor<Func>, f_a: CausalTensor<A>) -> CausalTensor<B>
    where
        A: Clone,
        Func: FnMut(A) -> B,
    {
        let shape = f_a.shape().to_vec();
        let funcs = f_ab.into_vec();
        let args = f_a.into_vec();

        if funcs.len() == args.len() {
            let data: Vec<B> = funcs.into_iter().zip(args).map(|(mut f, a)| f(a)).collect();
            CausalTensor::from_vec(data, &shape)
        } else if funcs.len() == 1 {
            let f = funcs.into_iter().next().unwrap();
            let data: Vec<B> = args.into_iter().map(f).collect();
            CausalTensor::from_vec(data, &shape)
        } else {
            // Return empty tensor on mismatch, as expected by tests
            CausalTensor::from_vec(vec![], &[0])
        }
    }
}
