/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, Tensor};
use deep_causality_haft::{Applicative, BoundedComonad, Foldable, Functor, HKT, Monad};
use deep_causality_num::Zero;

/// `CausalTensorWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `CausalTensor<T>` type constructor. It allows `CausalTensor` to be used with generic
/// functional programming traits like `Functor`, `Applicative`, `Foldable`, and `Monad`
/// from the `deep_causality_haft` crate.
///
/// By implementing `HKT` for `CausalTensorWitness`, we can write generic functions that operate
/// on any type that has the "shape" of `CausalTensor`, without knowing the inner type `T`.
pub struct CausalTensorWitness;

impl HKT for CausalTensorWitness {
    /// Specifies that `CausalTensorWitness` represents the `CausalTensor<T>` type constructor.
    type Type<T> = CausalTensor<T>;
}

// Implementation of Applicative for CausalTensorWitness
impl Applicative<CausalTensorWitness> for CausalTensorWitness {
    /// Lifts a pure value into a scalar `CausalTensor`.
    fn pure<T>(value: T) -> CausalTensor<T> {
        CausalTensor::from_vec(vec![value], &[])
    }

    /// Applies a function wrapped in a `CausalTensor` (`f_ab`) to a value wrapped in a `CausalTensor` (`f_a`).
    ///
    /// This implementation uses a Zip strategy (element-wise application) to avoid cloning arguments.
    /// Broadcast behavior is not supported for non-Clone types.
    fn apply<A, B, Func>(f_ab: CausalTensor<Func>, f_a: CausalTensor<A>) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        // Zip strategy: match elements. Broadcast is not supported without Clone.
        // Assuming shapes match or broadcasting logic handled externally.
        let shape_a = f_a.shape().to_vec();

        let mut result_data = Vec::with_capacity(f_a.len());
        let funcs = f_ab.into_vec();
        let args = f_a.into_vec();

        // Use zip.
        for (mut f, a) in funcs.into_iter().zip(args.into_iter()) {
            result_data.push(f(a));
        }

        CausalTensor::from_vec(result_data, &shape_a)
    }
}

// Implementation of Foldable for CausalTensorWitness
impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    /// Folds (reduces) a `CausalTensor` into a single value.
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.into_vec().into_iter().fold(init, f)
    }
}

// Implementation of Functor for CausalTensorWitness
impl Functor<CausalTensorWitness> for CausalTensorWitness {
    /// Implements the `fmap` operation for `CausalTensor<T>`.
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec(); // Clone shape before moving data
        let new_data = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

// Implementation of Monad for CausalTensorWitness
impl Monad<CausalTensorWitness> for CausalTensorWitness {
    /// Implements the `bind` (or `flat_map`) operation for `CausalTensor<T>`.
    fn bind<A, B, Func>(m_a: CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> CausalTensor<B>,
    {
        let mut result_data = Vec::new();
        // Bind consumes elements and flattens results.

        for a in m_a.into_vec() {
            let mb = f(a);
            result_data.extend(mb.into_vec());
        }

        let len = result_data.len();
        CausalTensor::from_vec(result_data, &[len])
    }
}

// Implementation of CoMonad for CausalTensorWitness
impl BoundedComonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Clone,
    {
        if fa.ndim() == 0 && !fa.is_empty() {
            let v = fa.to_vec();
            v.into_iter().next().unwrap()
        } else if fa.is_empty() {
            panic!("CoMonad::extract cannot be called on an empty CausalTensor.");
        } else {
            let v = fa.to_vec();
            v.into_iter().next().unwrap()
        }
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(&CausalTensor<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        let len = fa.len();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let focused_view = fa.shifted_view(i);
                f(&focused_view)
            })
            .collect();

        CausalTensor::from_slice(&new_data, fa.shape())
    }
}

// Implementation of BoundedAdjunction for CausalTensorWitness
use deep_causality_haft::BoundedAdjunction;
use std::ops::{Add, Mul};

impl BoundedAdjunction<CausalTensorWitness, CausalTensorWitness, Vec<usize>>
    for CausalTensorWitness
{
    fn left_adjunct<A, B, F>(ctx: &Vec<usize>, a: A, f: F) -> CausalTensor<B>
    where
        F: Fn(CausalTensor<A>) -> B,
        A: Clone + Zero + Copy + PartialEq,
        B: Clone,
    {
        // 1. Create unit: A -> Tensor<Tensor<A>>
        let t_t_a = Self::unit(ctx, a);

        // 2. Map f: Tensor<A> -> B over Tensor<Tensor<A>> to get Tensor<B>
        <Self as Functor<Self>>::fmap(t_t_a, f)
    }

    fn right_adjunct<A, B, F>(ctx: &Vec<usize>, la: CausalTensor<A>, f: F) -> B
    where
        F: FnMut(A) -> CausalTensor<B>,
        A: Clone + Zero,
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>,
    {
        let mapped = <Self as Functor<Self>>::fmap(la, f);
        Self::counit(ctx, mapped)
    }

    fn unit<A>(ctx: &Vec<usize>, a: A) -> CausalTensor<CausalTensor<A>>
    where
        A: Clone + Zero + Copy + PartialEq,
    {
        if !ctx.is_empty() {
            panic!(
                "BoundedAdjunction::unit requires empty shape (Scalar). Provided: {:?}",
                ctx
            );
        }
        let inner = CausalTensor::from_vec(vec![a], ctx);
        CausalTensor::from_vec(vec![inner], &[])
    }

    fn counit<B>(_ctx: &Vec<usize>, lrb: CausalTensor<CausalTensor<B>>) -> B
    where
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>,
    {
        // Flatten and Extract
        // lrb is Tensor<Tensor<B>>.
        // bind flattens.
        let flattened = <Self as Monad<Self>>::bind(lrb, |x| x);
        <Self as BoundedComonad<Self>>::extract(&flattened)
    }
}
