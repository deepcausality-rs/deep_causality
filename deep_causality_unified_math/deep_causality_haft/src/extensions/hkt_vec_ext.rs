/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Applicative, CloneFunctor, DebugFunctor, EqFunctor, Foldable, Functor, HKT, Monad,
    NoConstraint, Pure, Satisfies,
};
use alloc::vec;
use alloc::vec::Vec;

/// `VecWitness` is a zero-sized type that acts as a Higher-Kinded Type (HKT) witness
/// for the `Vec<T>` type constructor. It allows `Vec` to be used with generic
/// functional programming traits like `Functor`, `Applicative`, `Foldable`, and `Monad`.
///
/// By implementing `HKT` for `VecWitness`, we can write generic functions that operate
/// on any type that has the "shape" of `Vec`, without knowing the inner type `T`.
///
/// # Constraint
///
/// `VecWitness` uses `NoConstraint`, meaning it works with any type `T`.
pub struct VecWitness;

impl HKT for VecWitness {
    type Constraint = NoConstraint;

    /// Specifies that `VecWitness` represents the `Vec<T>` type constructor.
    type Type<T> = Vec<T>;
}

// Implementation of Pure for VecWitness
impl Pure<VecWitness> for VecWitness {
    /// Lifts a pure value into a `Vec` containing only that value.
    fn pure<T>(value: T) -> <VecWitness as HKT>::Type<T>
    where
        T: Satisfies<NoConstraint>,
    {
        vec![value]
    }
}

// Implementation of Applicative for VecWitness
impl Applicative<VecWitness> for VecWitness {
    /// Applies a vector of functions (`f_ab`) to a vector of values (`f_a`).
    ///
    /// Each function in `f_ab` is applied to each value in `f_a`, producing a new vector
    /// containing all possible combinations of applications.
    fn apply<A, B, Func>(
        f_ab: <VecWitness as HKT>::Type<Func>,
        f_a: <VecWitness as HKT>::Type<A>,
    ) -> <VecWitness as HKT>::Type<B>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        Func: Satisfies<NoConstraint> + FnMut(A) -> B,
    {
        f_ab.into_iter()
            .flat_map(|mut f_val| {
                f_a.iter()
                    .map(move |a_val| f_val(a_val.clone()))
                    .collect::<Vec<B>>()
            })
            .collect()
    }
}

// Implementation of Functor for VecWitness
impl Functor<VecWitness> for VecWitness {
    /// Implements the `fmap` operation for `Vec<T>`.
    ///
    /// Applies the function `f` to each element in the vector, producing a new vector.
    fn fmap<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        m_a.into_iter().map(f).collect()
    }
}

// Implementation of Foldable for VecWitness
impl Foldable<VecWitness> for VecWitness {
    /// Folds (reduces) a `Vec` into a single value.
    fn fold<A, B, Func>(fa: <VecWitness as HKT>::Type<A>, init: B, f: Func) -> B
    where
        <VecWitness as HKT>::Type<A>: IntoIterator<Item = A>,
        A: Satisfies<NoConstraint>,
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
    fn bind<A, B, Func>(m_a: <VecWitness as HKT>::Type<A>, f: Func) -> <VecWitness as HKT>::Type<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> <VecWitness as HKT>::Type<B>,
    {
        m_a.into_iter().flat_map(f).collect()
    }
}

// Implementation of EqFunctor for VecWitness (element-wise structural equality of `Vec<T>`).
impl EqFunctor for VecWitness {
    fn eq_type<T: PartialEq>(a: &Vec<T>, b: &Vec<T>) -> bool {
        a == b
    }
}

// Implementation of DebugFunctor for VecWitness (delegates to `Vec`'s own `Debug`).
impl DebugFunctor for VecWitness {
    fn fmt_type<T: core::fmt::Debug>(
        fa: &Vec<T>,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        core::fmt::Debug::fmt(fa, f)
    }
}

// Implementation of CloneFunctor for VecWitness (delegates to `Vec`'s own `Clone`).
impl CloneFunctor for VecWitness {
    fn clone_type<T: Clone>(fa: &Vec<T>) -> Vec<T> {
        fa.clone()
    }
}

// NOTE: `Traversable` is deliberately not implemented for `VecWitness`, and the reason is a
// signature one rather than a mathematical one.
//
// The usual `sequence` for a list folds an accumulator through the inner applicative:
//
//     acc = M::apply(M::fmap(acc, |v| move |a| { v.push(a); v }), m_a)
//
// That puts a *function* inside `M`, so `Applicative::apply` requires the anonymous closure type
// to satisfy `M::Constraint`:
//
//     error[E0277]: the trait bound `{closure@...}: Satisfies<<M as HKT>::Constraint>`
//                   is not satisfied
//
// `sequence` cannot declare that, and an impl may not add the bound itself (E0276). The same fold
// written against a `zip_with`-style structure map compiles and passes, because the combining
// function never enters `M`:
//
//     acc = M::zip_with(acc, m_a, |mut v, a| { v.push(a); v });
//
// The `zip_with` structure map now exists (`crate::Semigroupal`), and a `sequence` written
// against it does compile and pass. It is still not implemented here, and that is a decision
// rather than an omission: `sequence`'s inner-`M` bound would have to move from `Applicative` to
// `Semigroupal + Pure`, and those two are substitutive rather than comparable. Measured, that
// swap takes the witnesses admissible as the inner applicative from 19 down to 3, losing every
// effect monad in the workspace — `StudyEffectWitness`, `CdlEffectWitness`,
// `GraphGeneratableEffectWitness` and the `MyEffectHktWitness` family — along with `BoxWitness`,
// `LinkedListWitness`, `ManifoldWitness`, `CausalTensorWitness` and `VecWitness` itself. One
// carrier gained is not worth sixteen lost.
//
// Revisit only as part of a change that first adopts `Semigroupal` across those witnesses, so the
// bound can move without narrowing the trait's contract. See
// `openspec/notes/archive/hkt_gat/monoidal-applicative.md` §6 finding 5 for the measurement. Until then,
// `OptionWitness` and `ResultWitness` are the only two `Traversable` carriers.
