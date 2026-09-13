/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Applicative, CloneFunctor, DebugFunctor, EqFunctor, Foldable, Functor, HKT, Monad, Pure,
    Traversable,
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
/// # Element bounds
///
/// `VecWitness` places no bound on `T`.
pub struct VecWitness;

impl HKT for VecWitness {
    /// Specifies that `VecWitness` represents the `Vec<T>` type constructor.
    type Type<T> = Vec<T>;
}

// Implementation of Pure for VecWitness
impl Pure<VecWitness> for VecWitness {
    /// Lifts a pure value into a `Vec` containing only that value.
    fn pure<T>(value: T) -> <VecWitness as HKT>::Type<T> {
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
        A: Clone,
        Func: FnMut(A) -> B,
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

// Implementation of Traversable for VecWitness
impl Traversable<VecWitness> for VecWitness {
    /// Flips `Vec<M<A>>` into `M<Vec<A>>` by folding an accumulator through `M` from left to
    /// right, so the effects run in index order and the result keeps that order.
    ///
    /// An element in a failing state collapses the whole traversal, and the first such element in
    /// index order is the one reported. The empty vector yields `M::pure(Vec::new())`.
    ///
    /// # Cost
    ///
    /// The accumulator is cloned once per step and holds `k` elements at step `k`, so the fold
    /// performs `n(n-1)/2` element clones — quadratic — for an `n`-element input. The clone is
    /// forced by [`Applicative::apply`]'s `Func: FnMut` bound, not by this trait's `A: Clone`: an
    /// `FnMut` may be invoked repeatedly, so the closure cannot move its captured accumulator out,
    /// and the cartesian carriers do invoke it once per element. An `A: Copy` bound would not help,
    /// because the cloned value is the accumulator `Vec`, which is never `Copy`.
    fn sequence<A, M>(fa: alloc::vec::Vec<M::Type<A>>) -> M::Type<alloc::vec::Vec<A>>
    where
        M: Applicative<M> + HKT,
        A: Clone,
    {
        let mut acc: M::Type<Vec<A>> = M::pure(Vec::new());
        for m_a in fa {
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
        acc
    }
}

// NOTE: `VecWitness` implements `Traversable` through the `apply`-based accumulator fold above.
// It did not always, and the reason it does now is a change of premise rather than of preference.
//
// The fold puts an anonymous closure inside `M`:
//
//     acc = M::apply(M::fmap(acc, |v| move |a| { v.push(a); v }), m_a)
//
// While `HKT` carried an associated `Constraint` and every method carried a
// `T: Satisfies<F::Constraint>` bound, `Applicative::apply` required that closure type to satisfy
// the witness constraint, `sequence` could not declare it, and an impl could not add it (E0276).
// That closed this route. `Satisfies` and the associated `Constraint` have since been removed —
// `HKT` is now `type Type<T>;` alone, and `lax_monoidal/mod.rs` records the removal — so the
// obstruction is gone and the fold compiles.
//
// The alternative route, a fold written against `Semigroupal::zip_with`, is still rejected, and
// the measurement that rejects it still stands: it needs `sequence`'s inner-`M` bound moved from
// `Applicative` to `Semigroupal + Pure`, and those are substitutive rather than comparable.
// Measured, that swap takes the witnesses admissible as the inner applicative from 19 down to 3,
// losing every effect monad in the workspace — `StudyEffectWitness`, `CdlEffectWitness`,
// `GraphGeneratableEffectWitness` and the `MyEffectHktWitness` family — along with `BoxWitness`,
// `LinkedListWitness`, `ManifoldWitness` and `CausalTensorWitness`. The `apply` route needs no
// such move, so the bound stays at `Applicative` and nothing is lost. See
// `openspec/notes/archive/hkt_gat/monoidal-applicative.md` §6 finding 5 for that measurement.
//
// One law is not tested here and cannot be: composition. It needs a `Compose<M, N>` applicative
// with `Type<T> = M<N<T>>`, whose `apply` would require `N::Type<A>: Clone` — a bound on a
// method-level parameter that neither the trait nor an impl can state. The defect class it would
// have caught is a traversal that visits elements in the wrong order while still returning the
// right result; `test_traversable_vec_effect_order_is_left_to_right` catches that instead, using a
// carrier that records the order in which `apply` runs.
