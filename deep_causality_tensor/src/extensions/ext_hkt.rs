/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::constraint::TensorDataConstraint;
use crate::{CausalTensor, Tensor};
use deep_causality_haft::{CoMonad, Foldable, Functor, HKT, Satisfies};

/// `CausalTensorWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `CausalTensor<T>` type constructor. It allows `CausalTensor` to be used with generic
/// functional programming traits like `Functor`, `Foldable`, and `CoMonad`
/// from the `deep_causality_haft` crate.
///
/// Note: `Applicative` and `Monad` are NOT implemented because `TensorDataConstraint`
/// strictly restricts inner types to numeric types (implementing `TensorData`).
/// `Applicative` requires `Type<Func>`, and `Monad` requires nested `Type<Type<A>>`,
/// neither of which satisfy the numeric constraint.
pub struct CausalTensorWitness;

impl HKT for CausalTensorWitness {
    // Strictly enforce TensorData constraint as per specification.
    // This allows CausalTensor to participate in the "Restricted Monad" category
    type Constraint = TensorDataConstraint;

    /// Specifies that `CausalTensorWitness` represents the `CausalTensor<T>` type constructor.
    type Type<T> = CausalTensor<T>;
}

// Implementation of Foldable for CausalTensorWitness
impl Foldable<CausalTensorWitness> for CausalTensorWitness {
    /// Folds (reduces) a `CausalTensor` into a single value.
    fn fold<A, B, Func>(fa: CausalTensor<A>, init: B, f: Func) -> B
    where
        A: Satisfies<TensorDataConstraint>,
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
        A: Satisfies<TensorDataConstraint>,
        B: Satisfies<TensorDataConstraint>,
        Func: FnMut(A) -> B,
    {
        let shape = m_a.shape().to_vec(); // Clone shape before moving data
        let new_data = m_a.into_vec().into_iter().map(f).collect();
        CausalTensor::from_vec(new_data, &shape)
    }
}

// Implementation of CoMonad for CausalTensorWitness
impl CoMonad<CausalTensorWitness> for CausalTensorWitness {
    fn extract<A>(fa: &CausalTensor<A>) -> A
    where
        A: Satisfies<TensorDataConstraint> + Clone,
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
        A: Satisfies<TensorDataConstraint> + Clone,
        B: Satisfies<TensorDataConstraint>,
    {
        let len = fa.len();
        let new_data: Vec<B> = (0..len)
            .map(|i| {
                let focused_view = fa.shifted_view(i);
                f(&focused_view)
            })
            .collect();

        CausalTensor::from_vec(new_data, fa.shape())
    }
}
