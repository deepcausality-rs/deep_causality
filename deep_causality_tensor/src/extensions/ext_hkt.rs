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
    ///
    /// # Arguments
    ///
    /// *   `value`: The value to wrap in a `CausalTensor`.
    ///
    /// # Returns
    ///
    /// A new `CausalTensor` with a shape of `[]` containing the `value`.
    fn pure<T>(value: T) -> CausalTensor<T> {
        CausalTensor::new(vec![value], vec![]).expect("Scalar tensor creation should not fail")
    }

    /// Applies a function wrapped in a `CausalTensor` (`f_ab`) to a value wrapped in a `CausalTensor` (`f_a`).
    ///
    /// This implementation assumes `f_ab` is a scalar tensor containing a single function.
    /// It applies this function element-wise to all values in `f_a`.
    ///
    /// # Arguments
    ///
    /// *   `f_ab`: A `CausalTensor` containing a single function.
    /// *   `f_a`: A `CausalTensor` containing arguments.
    ///
    /// # Returns
    ///
    /// A `CausalTensor` containing the results of applying the function to each value.
    fn apply<A, B, Func>(f_ab: CausalTensor<Func>, f_a: CausalTensor<A>) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        if f_ab.shape().is_empty() && f_ab.len() == 1 {
            let shape = f_a.shape().to_vec(); // Extract shape before moving data
            let func = f_ab.data.into_iter().next().unwrap(); // Get the single function
            let new_data = f_a.data.into_iter().map(func).collect();
            CausalTensor::new(new_data, shape).expect("Shape should remain valid after apply")
        } else {
            // For now, return an empty tensor if f_ab is not a scalar function tensor.
            // A more complete implementation would involve broadcasting multiple functions to multiple values.
            // Or, consider returning a Result<CausalTensor<B>, Error>.
            CausalTensor::new(Vec::new(), vec![0])
                .expect("Creating an empty tensor should not fail")
        }
    }
}

// Implementation of Foldable for CausalTensorWitness
impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    /// Folds (reduces) a `CausalTensor` into a single value.
    ///
    /// Applies the function `f` cumulatively to the elements of the tensor,
    /// starting with an initial accumulator value.
    ///
    /// # Arguments
    ///
    /// *   `fa`: The `CausalTensor` to fold.
    /// *   `init`: The initial accumulator value.
    /// *   `f`: The folding function.
    ///
    /// # Returns
    ///
    /// The accumulated result.
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        fa.data.into_iter().fold(init, f)
    }
}

// Implementation of Functor for CausalTensorWitness
impl Functor<CausalTensorWitness> for CausalTensorWitness {
    /// Implements the `fmap` operation for `CausalTensor<T>`.
    ///
    /// Applies the function `f` to each element in the tensor, producing a new tensor.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The `CausalTensor` to map over.
    /// *   `f`: The function to apply to each element.
    ///
    /// # Returns
    ///
    /// A new `CausalTensor` with the function applied to each of its elements.
    fn fmap<A, B, Func>(m_a: CausalTensor<A>, f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec(); // Extract shape before moving data
        let new_data = m_a.data.into_iter().map(f).collect();
        CausalTensor::new(new_data, shape).expect("Shape should remain valid after fmap")
    }
}

// Implementation of Monad for CausalTensorWitness
impl Monad<CausalTensorWitness> for CausalTensorWitness {
    /// Implements the `bind` (or `flat_map`) operation for `CausalTensor<T>`.
    ///
    /// Applies the function `f` to each element in the tensor, where `f` itself
    /// returns a new `CausalTensor`. The data from all resulting tensors are then
    /// concatenated into a single 1-dimensional `CausalTensor`.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The initial `CausalTensor`.
    /// *   `f`: A function that takes an inner value and returns a new `CausalTensor`.
    ///
    /// # Returns
    ///
    /// A new 1-dimensional `CausalTensor` representing the chained and flattened computation.
    fn bind<A, B, Func>(m_a: CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(A) -> CausalTensor<B>,
    {
        let result_data: Vec<B> = m_a
            .data
            .into_iter()
            .flat_map(|val_a| f(val_a).data.into_iter())
            .collect();

        let result_len = result_data.len();
        CausalTensor::new(result_data, vec![result_len])
            .expect("Concatenated tensor creation should not fail")
    }
}

// Implementation of CoMonad for CausalTensorWitness
impl BoundedComonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Clone,
    {
        // 'extract' is typically defined for contexts with a single, clear focus.
        // For CausalTensor, this is most naturally a scalar tensor (0-dimensional).
        // If the tensor is a scalar and non-empty, its single value is the focus.
        if fa.num_dim() == 0 && !fa.is_empty() {
            fa.data[0].clone()
        } else if fa.is_empty() {
            // As CoMonad::extract must return an 'A' and cannot fail,
            // this indicates a conceptual mismatch between CoMonad and potentially
            // empty/multi-element CausalTensors.
            // Panicking here reflects that an 'A' cannot be extracted from an empty context.
            panic!("CoMonad::extract cannot be called on an empty CausalTensor.");
        } else {
            // For non-scalar tensors (e.g., vectors, matrices), a single 'focus'
            // is not inherently defined. Choosing the first element is an arbitrary
            // decision required to satisfy the CoMonad trait signature.
            // Users should ideally ensure that CausalTensors treated as CoMonads
            // are scalar for meaningful 'extract' operations.
            fa.data[0].clone()
        }
    }

    fn extend<A, B, Func>(fa: &CausalTensor<A>, mut f: Func) -> CausalTensor<B>
    where
        Func: FnMut(&CausalTensor<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        let new_data: Vec<B> = (0..fa.data.len())
            .map(|i| {
                // The Logic: Create the view, apply the function
                let focused_view = fa.shifted_view(i);
                f(&focused_view)
            })
            .collect();

        CausalTensor::new(new_data, fa.shape().to_vec()).expect("Shape mismatch")
    }
}

// Implementation of BoundedAdjunction for CausalTensorWitness
// Context is Vec<usize> (Shape), as we need it to construct new Tensors in 'unit'.
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
        // Create inner tensor
        if !ctx.is_empty() {
            panic!(
                "BoundedAdjunction::unit for CausalTensor requires an empty shape vector (Scalar). Provided shape: {:?}",
                ctx
            );
        }
        let inner = CausalTensor::new(vec![a], ctx.clone()).expect("Inner tensor creation failed");

        // Wrap in outer tensor (scalar wrapper)
        CausalTensor::new(vec![inner], vec![]).expect("Outer tensor creation failed")
    }

    fn counit<B>(_ctx: &Vec<usize>, lrb: CausalTensor<CausalTensor<B>>) -> B
    where
        B: Clone + Zero + Add<Output = B> + Mul<Output = B>,
    {
        // Flatten and Extract
        let flattened = <Self as Monad<Self>>::bind(lrb, |x| x);
        <Self as BoundedComonad<Self>>::extract(&flattened)
    }
}
