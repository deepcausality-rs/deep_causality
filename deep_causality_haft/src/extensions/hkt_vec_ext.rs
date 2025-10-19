/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Applicative, Foldable, Functor, HKT, Monad};

/// `VecWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `Vec<T>` type constructor. It allows `Vec` to be used with generic
/// functional programming traits like `Functor`, `Applicative`, `Foldable`, and `Monad`.
///
/// By implementing `HKT` for `VecWitness`, we can write generic functions that operate
/// on any type that has the "shape" of `Vec`, without knowing the inner type `T`.
pub struct VecWitness;

impl HKT for VecWitness {
    /// Specifies that `VecWitness` represents the `Vec<T>` type constructor.
    type Type<T> = Vec<T>;
}

// Implementation of Functor for VecWitness
impl Functor<VecWitness> for VecWitness {
    /// Implements the `fmap` operation for `Vec<T>`.
    ///
    /// Applies the function `f` to each element in the vector, producing a new vector.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The `Vec` to map over.
    /// *   `f`: The function to apply to each element.
    ///
    /// # Returns
    ///
    /// A new `Vec` with the function applied to each of its elements.
    fn fmap<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
    {
        m_a.into_iter().map(f).collect()
    }
}

// Implementation of Applicative for VecWitness
impl Applicative<VecWitness> for VecWitness {
    /// Lifts a pure value into a `Vec` containing only that value.
    ///
    /// # Arguments
    ///
    /// *   `value`: The value to wrap in a `Vec`.
    ///
    /// # Returns
    ///
    /// A new `Vec` containing `value`.
    fn pure<T>(value: T) -> <VecWitness as HKT>::Type<T> {
        vec![value]
    }

    /// Applies a vector of functions (`f_ab`) to a vector of values (`f_a`).
    ///
    /// Each function in `f_ab` is applied to each value in `f_a`, producing a new vector
    /// containing all possible combinations of applications.
    ///
    /// # Arguments
    ///
    /// *   `f_ab`: A `Vec` containing functions.
    /// *   `f_a`: A `Vec` containing arguments.
    ///
    /// # Returns
    ///
    /// A new `Vec` with the results of applying each function to each value.
    fn apply<A, B, Func>(
        f_ab: <VecWitness as HKT>::Type<Func>,
        f_a: <VecWitness as HKT>::Type<A>,
    ) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> B,
        A: Clone,
    {
        f_ab.into_iter()
            .flat_map(|mut f_val| {
                f_a.iter()
                    .map(move |a_val| f_val(a_val.clone()))
                    .collect::<Vec<B>>()
            }) // Clone a_val for FnMut
            .collect()
    }
}

// Implementation of Foldable for VecWitness
impl Foldable<VecWitness> for VecWitness {
    /// Folds (reduces) a `Vec` into a single value.
    ///
    /// Applies the function `f` cumulatively to the elements of the vector,
    /// starting with an initial accumulator value.
    ///
    /// # Arguments
    ///
    /// *   `fa`: The `Vec` to fold.
    /// *   `init`: The initial accumulator value.
    /// *   `f`: The folding function.
    ///
    /// # Returns
    ///
    /// The accumulated result.
    fn fold<A, B, Func>(fa: <VecWitness as HKT>::Type<A>, init: B, f: Func) -> B
    where
        <VecWitness as HKT>::Type<A>: IntoIterator<Item = A>,
        Func: FnMut(B, A) -> B,
    {
        fa.into_iter().fold(init, f)
    }
}

// Implementation of Monad for VecWitness
impl Monad<VecWitness> for VecWitness {
    /// Implements the `bind` (or `flat_map`) operation for `Vec<T>`.
    ///
    /// Applies the function `f` to each element in the vector, where `f` itself
    /// returns a new vector. All the resulting vectors are then concatenated into a single `Vec`.
    /// This is useful for chaining computations that produce multiple results.
    ///
    /// # Arguments
    ///
    /// *   `m_a`: The initial `Vec`.
    /// *   `f`: A function that takes an inner value and returns a new `Vec`.
    ///
    /// # Returns
    ///
    /// A new `Vec` representing the chained and flattened computation.
    fn bind<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        Func: FnMut(A) -> <VecWitness as HKT>::Type<B>,
    {
        m_a.into_iter().flat_map(f).collect()
    }
}
